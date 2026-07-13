use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};

use crate::audio::devices::configuration::{AudioDevice, DeviceType};

/// Configure Linux audio devices using ALSA/PulseAudio
pub fn configure_linux_audio(host: &cpal::Host) -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    // Enumerate each direction once. Re-opening the ALSA host and then calling
    // host.devices() caused three full native scans on every monitor poll.
    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            devices.push(AudioDevice::new(name.clone(), DeviceType::Input));

            if name.contains("monitor") {
                devices.push(AudioDevice::new(
                    format!("{} (System Audio)", name),
                    DeviceType::Output
                ));
            }
        }
    }

    for device in host.output_devices()? {
        if let Ok(name) = device.name() {
            if !devices.iter().any(|candidate| candidate.name == name) {
                devices.push(AudioDevice::new(name, DeviceType::Output));
            }
        }
    }

    Ok(devices)
}
