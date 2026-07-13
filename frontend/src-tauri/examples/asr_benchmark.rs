//! Minimal live adapter for reference-backed ASR comparisons.

use app_lib::audio::decoder::decode_audio_file;
use app_lib::whisper_engine::WhisperEngine;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let model_path = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: asr_benchmark MODEL_PATH AUDIO_FILE..."))?;
    let audio_paths: Vec<PathBuf> = args.map(PathBuf::from).collect();
    if audio_paths.is_empty() {
        anyhow::bail!("at least one audio file is required");
    }

    let model_dir = model_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("model path must have a parent directory"))?;
    let model_name = model_name_from_path(&model_path)?;
    let engine = WhisperEngine::new_with_models_dir(Some(model_dir.to_path_buf()))?;
    engine.discover_models().await?;

    let load_started = Instant::now();
    engine.load_model(&model_name).await?;
    let load_seconds = load_started.elapsed().as_secs_f64();

    for audio_path in audio_paths {
        let decoded = decode_audio_file(&audio_path)?;
        let duration_seconds = decoded.duration_seconds;
        let audio = decoded.to_whisper_format();
        let started = Instant::now();
        let transcript = engine
            .transcribe_audio(audio, Some("zh".to_string()))
            .await?;
        let runtime_seconds = started.elapsed().as_secs_f64();

        println!(
            "{}",
            json!({
                "runtime": "meetily_whisper_rs",
                "model": model_name,
                "model_path": model_path,
                "audio_path": audio_path,
                "audio_seconds": duration_seconds,
                "model_load_seconds": load_seconds,
                "runtime_seconds": runtime_seconds,
                "real_time_factor": runtime_seconds / duration_seconds,
                "transcript": transcript,
            })
        );
    }

    Ok(())
}

fn model_name_from_path(path: &Path) -> anyhow::Result<String> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("model path must be valid UTF-8"))?;
    filename
        .strip_prefix("ggml-")
        .and_then(|name| name.strip_suffix(".bin"))
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("expected a ggml-MODEL.bin filename"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_catalog_name_from_model_path() {
        assert_eq!(
            model_name_from_path(Path::new("/models/ggml-breeze-asr-26.bin")).unwrap(),
            "breeze-asr-26"
        );
    }
}
