use anyhow::{anyhow, Result};
use log::{error, info};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::thread::JoinHandle;
use tokio::sync::{mpsc, oneshot};

use super::devices::AudioDevice;
use super::recording_state::RecordingState;
use super::stream::AudioStreamManager;

const COMMAND_CAPACITY: usize = 8;

enum AudioCommand {
    Start {
        state: Arc<RecordingState>,
        microphone: Option<Arc<AudioDevice>>,
        system: Option<Arc<AudioDevice>>,
        reply: oneshot::Sender<Result<()>>,
    },
    Stop {
        reply: oneshot::Sender<Result<()>>,
    },
    Shutdown,
}

/// Owns every native audio stream on one dedicated thread.
///
/// CPAL streams never cross this boundary: creation, playback, pause, drop,
/// and reconnect all execute inside the owner thread's local Tokio runtime.
pub struct AudioRuntime {
    sender: Option<mpsc::Sender<AudioCommand>>,
    active_streams: Arc<AtomicUsize>,
    owner_thread: Option<JoinHandle<()>>,
}

impl AudioRuntime {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(COMMAND_CAPACITY);
        let active_streams = Arc::new(AtomicUsize::new(0));
        let owner_active_streams = active_streams.clone();

        let owner_thread = std::thread::Builder::new()
            .name("meetily-audio-owner".to_string())
            .spawn(move || run_owner(receiver, owner_active_streams))
            .expect("failed to start audio owner thread");

        Self {
            sender: Some(sender),
            active_streams,
            owner_thread: Some(owner_thread),
        }
    }

    pub async fn start_streams(
        &self,
        state: Arc<RecordingState>,
        microphone: Option<Arc<AudioDevice>>,
        system: Option<Arc<AudioDevice>>,
    ) -> Result<()> {
        let (reply, response) = oneshot::channel();
        self.send(AudioCommand::Start {
            state,
            microphone,
            system,
            reply,
        })
        .await?;
        response
            .await
            .map_err(|_| anyhow!("audio owner thread stopped before start completed"))?
    }

    pub async fn stop_streams(&self) -> Result<()> {
        let (reply, response) = oneshot::channel();
        self.send(AudioCommand::Stop { reply }).await?;
        response
            .await
            .map_err(|_| anyhow!("audio owner thread stopped before stop completed"))?
    }

    pub fn active_stream_count(&self) -> usize {
        self.active_streams.load(Ordering::Acquire)
    }

    async fn send(&self, command: AudioCommand) -> Result<()> {
        self.sender
            .as_ref()
            .ok_or_else(|| anyhow!("audio owner thread is shut down"))?
            .send(command)
            .await
            .map_err(|_| anyhow!("audio owner thread is unavailable"))
    }
}

impl Default for AudioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioRuntime {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(AudioCommand::Shutdown);
            drop(sender);
        }

        if let Some(owner_thread) = self.owner_thread.take() {
            if owner_thread.join().is_err() {
                error!("audio owner thread panicked during shutdown");
            }
        }
    }
}

fn run_owner(receiver: mpsc::Receiver<AudioCommand>, active_streams: Arc<AtomicUsize>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build audio owner runtime");
    let local = tokio::task::LocalSet::new();

    local.block_on(&runtime, owner_loop(receiver, active_streams));
}

async fn owner_loop(mut receiver: mpsc::Receiver<AudioCommand>, active_streams: Arc<AtomicUsize>) {
    let mut manager: Option<AudioStreamManager> = None;

    while let Some(command) = receiver.recv().await {
        match command {
            AudioCommand::Start {
                state,
                microphone,
                system,
                reply,
            } => {
                if let Err(error) = stop_manager(&mut manager, &active_streams) {
                    let _ = reply.send(Err(error));
                    continue;
                }

                let mut next_manager = AudioStreamManager::new(state);
                let result = next_manager.start_streams(microphone, system).await;
                if result.is_ok() {
                    active_streams.store(next_manager.active_stream_count(), Ordering::Release);
                    manager = Some(next_manager);
                }
                let _ = reply.send(result);
            }
            AudioCommand::Stop { reply } => {
                let result = stop_manager(&mut manager, &active_streams);
                let _ = reply.send(result);
            }
            AudioCommand::Shutdown => break,
        }
    }

    if let Err(error) = stop_manager(&mut manager, &active_streams) {
        error!("failed to stop streams during audio owner shutdown: {error}");
    }
    info!("audio owner thread stopped");
}

fn stop_manager(
    manager: &mut Option<AudioStreamManager>,
    active_streams: &AtomicUsize,
) -> Result<()> {
    let result = manager
        .take()
        .map(|mut manager| manager.stop_streams())
        .unwrap_or(Ok(()));
    active_streams.store(0, Ordering::Release);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::devices::default_input_device;

    #[tokio::test]
    async fn owner_starts_and_stops_without_native_streams() {
        let runtime = AudioRuntime::new();
        runtime.stop_streams().await.unwrap();
        assert_eq!(runtime.active_stream_count(), 0);
    }

    #[tokio::test]
    #[ignore = "requires a live default microphone"]
    async fn live_default_microphone_survives_repeated_owner_lifecycles() {
        let state = RecordingState::new();
        let runtime = AudioRuntime::new();

        for cycle in 1..=25 {
            let (sender, mut receiver) = mpsc::unbounded_channel();
            state.start_recording().unwrap();
            state.set_audio_sender(sender);

            runtime
                .start_streams(
                    state.clone(),
                    Some(Arc::new(default_input_device().unwrap())),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(runtime.active_stream_count(), 1, "cycle {cycle}");

            tokio::time::timeout(std::time::Duration::from_secs(3), receiver.recv())
                .await
                .unwrap_or_else(|_| panic!("cycle {cycle} produced no callback within three seconds"))
                .unwrap_or_else(|| panic!("cycle {cycle} closed before the first callback"));

            runtime.stop_streams().await.unwrap();
            state.stop_recording();
            assert_eq!(runtime.active_stream_count(), 0, "cycle {cycle}");
        }
    }
}
