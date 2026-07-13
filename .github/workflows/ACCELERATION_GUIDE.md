# Meetily GPU-only ASR Build Guide

Meetily activates speech recognition through a GPU backend. This keeps the
runtime contract simple: transcription starts after a supported GPU backend is
compiled and detected, and the application returns an actionable activation
error when that contract is not met.

## Supported paths

| Platform | Whisper backend | Parakeet backend | Build feature |
| --- | --- | --- | --- |
| macOS | Metal | Separate CUDA-capable deployment path | automatic Metal/CoreML |
| Windows + NVIDIA | CUDA | CUDA ONNX Runtime | `cuda` |
| Windows + AMD/Intel | Vulkan | Separate CUDA-capable deployment path | `vulkan` |
| Linux + NVIDIA | CUDA | CUDA ONNX Runtime | `cuda` |
| Linux + AMD | HIP or Vulkan | Separate CUDA-capable deployment path | `hipblas` or `vulkan` |
| Linux + Intel | Vulkan | Separate CUDA-capable deployment path | `vulkan` |

Parakeet uses CUDA ONNX Runtime with CPU execution-provider fallback disabled.
Whisper supports the broader GPU matrix shown above.

## Local commands

The automatic scripts detect the available GPU toolchain and stop with an
activation message when no supported backend is ready:

```bash
cd frontend
npm run tauri:dev
npm run tauri:build
```

Explicit builds remain available when the target hardware is known:

```bash
# NVIDIA CUDA: Whisper + Parakeet
npm run tauri:dev:cuda
npm run tauri:build:cuda

# Vulkan: Whisper
npm run tauri:dev:vulkan
npm run tauri:build:vulkan

# Linux AMD ROCm: Whisper
npm run tauri:dev:hipblas
npm run tauri:build:hipblas
```

macOS uses the target-specific Metal/CoreML dependency configuration.

## Runtime enforcement

Whisper model loading validates both layers before inference:

1. the binary contains a GPU backend;
2. runtime hardware detection sees a compatible GPU runtime.

Parakeet session creation registers the CUDA execution provider with
`error_on_failure` and sets `session.disable_cpu_ep_fallback=1`. A missing CUDA
provider therefore produces a visible activation error instead of CPU
inference.

## Release workflow contract

Windows release workflows build Whisper with Vulkan. Linux release workflows
also build with Vulkan and install the matching SDK. macOS builds retain the
target-specific Metal/CoreML path.

CI runners validate that the GPU-enabled binaries compile. Hardware-backed
release qualification then runs a real audio inference on target GPU hardware
and records the compiled backend, transcript, timing, and device telemetry.

## Troubleshooting

### CUDA activation

- Confirm `nvidia-smi` sees the target GPU.
- Confirm `nvcc` or `CUDA_PATH`/`CUDA_HOME` exposes the CUDA Toolkit.
- Build with `--features cuda`.
- For Parakeet, confirm the ONNX Runtime CUDA provider can load its CUDA and
  cuDNN dependencies.

### Vulkan activation

- Confirm the GPU driver exposes Vulkan.
- Install the Vulkan SDK and set `VULKAN_SDK`.
- Build with `--features vulkan`.

### HIP activation

- Confirm `rocm-smi` sees the target GPU.
- Confirm `hipcc` or `ROCM_PATH` exposes ROCm.
- Build with `--features hipblas`.

The activation error is the authoritative next-action message. Meetily keeps
model files and user data intact while the required GPU layer is prepared.
