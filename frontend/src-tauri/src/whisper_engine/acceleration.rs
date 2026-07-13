use crate::audio::{GpuType, PerformanceTier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhisperCompiledBackend {
    Metal,
    Cuda,
    Vulkan,
    HipBlas,
    Cpu,
}

impl WhisperCompiledBackend {
    pub fn current() -> Self {
        if cfg!(feature = "cuda") {
            Self::Cuda
        } else if cfg!(feature = "vulkan") {
            Self::Vulkan
        } else if cfg!(feature = "hipblas") {
            Self::HipBlas
        } else if cfg!(target_os = "macos") || cfg!(feature = "metal") {
            Self::Metal
        } else {
            Self::Cpu
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Metal => "Metal",
            Self::Cuda => "Cuda",
            Self::Vulkan => "Vulkan",
            Self::HipBlas => "HipBlas",
            Self::Cpu => "Cpu",
        }
    }

    pub fn activation_hint(self) -> &'static str {
        match self {
            Self::Metal => "run on a Metal-capable macOS device",
            Self::Cuda => "install a working NVIDIA CUDA runtime",
            Self::Vulkan => "install a working Vulkan GPU runtime",
            Self::HipBlas => "install a working ROCm/HIP runtime",
            Self::Cpu => {
                "rebuild Meetily with --features cuda, --features vulkan, or --features hipblas"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhisperContextAcceleration {
    pub compiled_backend: WhisperCompiledBackend,
    pub runtime_detected_gpu: GpuType,
    pub use_gpu: bool,
    pub flash_attn: bool,
    pub gpu_device: i32,
}

impl WhisperContextAcceleration {
    pub fn status_label(self) -> &'static str {
        match (self.compiled_backend, self.flash_attn) {
            (WhisperCompiledBackend::Metal, true) => "Metal GPU with Flash Attention (Ultra-Fast)",
            (WhisperCompiledBackend::Metal, false) => "Metal GPU acceleration",
            (WhisperCompiledBackend::Cuda, true) => "CUDA GPU with Flash Attention (Ultra-Fast)",
            (WhisperCompiledBackend::Cuda, false) => "CUDA GPU acceleration",
            (WhisperCompiledBackend::Vulkan, _) => "Vulkan GPU acceleration",
            (WhisperCompiledBackend::HipBlas, _) => "HIP BLAS GPU acceleration",
            (WhisperCompiledBackend::Cpu, _) => "GPU backend activation required",
        }
    }
}

pub fn whisper_context_acceleration_for(
    compiled_backend: WhisperCompiledBackend,
    runtime_detected_gpu: GpuType,
    performance_tier: PerformanceTier,
) -> Result<WhisperContextAcceleration, String> {
    let runtime_matches_backend = match compiled_backend {
        WhisperCompiledBackend::Metal => runtime_detected_gpu == GpuType::Metal,
        WhisperCompiledBackend::Cuda => runtime_detected_gpu == GpuType::Cuda,
        WhisperCompiledBackend::Vulkan | WhisperCompiledBackend::HipBlas => {
            runtime_detected_gpu != GpuType::None
        }
        WhisperCompiledBackend::Cpu => false,
    };
    if !runtime_matches_backend {
        return Err(format!(
            "Meetily ASR requires an active GPU backend; compiled_backend={} runtime_detected_gpu={runtime_detected_gpu:?}. Next action: {}.",
            compiled_backend.as_str(),
            compiled_backend.activation_hint(),
        ));
    }

    let fast_tier = matches!(
        performance_tier,
        PerformanceTier::High | PerformanceTier::Ultra
    );
    let flash_attn = match compiled_backend {
        WhisperCompiledBackend::Metal | WhisperCompiledBackend::Cuda => fast_tier,
        WhisperCompiledBackend::Vulkan
        | WhisperCompiledBackend::HipBlas
        | WhisperCompiledBackend::Cpu => false,
    };

    Ok(WhisperContextAcceleration {
        compiled_backend,
        runtime_detected_gpu,
        use_gpu: true,
        flash_attn,
        gpu_device: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceleration_vulkan_backend_ignores_runtime_cuda_flash_attention() {
        let params = whisper_context_acceleration_for(
            WhisperCompiledBackend::Vulkan,
            GpuType::Cuda,
            PerformanceTier::High,
        )
        .unwrap();

        assert_eq!(params.compiled_backend, WhisperCompiledBackend::Vulkan);
        assert_eq!(params.runtime_detected_gpu, GpuType::Cuda);
        assert!(params.use_gpu);
        assert!(!params.flash_attn);
    }

    #[test]
    fn acceleration_vulkan_backend_requires_runtime_gpu_detection() {
        let error = whisper_context_acceleration_for(
            WhisperCompiledBackend::Vulkan,
            GpuType::None,
            PerformanceTier::Low,
        )
        .unwrap_err();

        assert!(error.contains("requires an active GPU backend"));
        assert!(error.contains("Vulkan"));
    }

    #[test]
    fn acceleration_cuda_backend_enables_flash_attention_for_fast_tiers() {
        let high = whisper_context_acceleration_for(
            WhisperCompiledBackend::Cuda,
            GpuType::Cuda,
            PerformanceTier::High,
        )
        .unwrap();
        let ultra = whisper_context_acceleration_for(
            WhisperCompiledBackend::Cuda,
            GpuType::Cuda,
            PerformanceTier::Ultra,
        )
        .unwrap();

        assert!(high.use_gpu);
        assert!(high.flash_attn);
        assert!(ultra.use_gpu);
        assert!(ultra.flash_attn);
    }

    #[test]
    fn acceleration_cpu_backend_is_rejected_for_every_runtime_detection() {
        for runtime_gpu in [GpuType::None, GpuType::Cuda, GpuType::Vulkan] {
            let error = whisper_context_acceleration_for(
                WhisperCompiledBackend::Cpu,
                runtime_gpu,
                PerformanceTier::Ultra,
            )
            .unwrap_err();

            assert!(error.contains("requires an active GPU backend"));
            assert!(error.contains("rebuild Meetily"));
        }
    }

    #[test]
    fn acceleration_cuda_backend_rejects_non_cuda_runtime() {
        let error = whisper_context_acceleration_for(
            WhisperCompiledBackend::Cuda,
            GpuType::Vulkan,
            PerformanceTier::High,
        )
        .unwrap_err();

        assert!(error.contains("install a working NVIDIA CUDA runtime"));
    }
}
