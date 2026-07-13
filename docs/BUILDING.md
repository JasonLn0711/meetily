# Building Meetily

Meetily is a Tauri desktop application with a Next.js frontend and a Rust core.
Its ASR runtime is GPU-only: a build can prepare the UI and model files on any
supported development machine, while transcription activates only when a
compatible GPU backend is compiled and detected.

## Prerequisites

- Node.js 18 or newer
- pnpm or npm
- Rust 1.85 or newer
- platform-specific Tauri system dependencies
- one supported ASR GPU toolchain from the table below

| Platform | GPU toolchain | Feature |
| --- | --- | --- |
| macOS | Metal/CoreML | target default |
| Windows/Linux NVIDIA | CUDA Toolkit | `cuda` |
| Windows/Linux AMD or Intel | Vulkan SDK | `vulkan` |
| Linux AMD | ROCm/HIP | `hipblas` |

Parakeet specifically uses the CUDA ONNX Runtime provider. Whisper supports
Metal, CUDA, Vulkan, and HIP.

## Install and run

```bash
cd frontend
pnpm install
pnpm run tauri:dev
```

`tauri:dev` and `tauri:build` call the GPU detector. The detector selects a
supported backend or returns an activation error with the missing toolchain.

For a known target, select the backend explicitly:

```bash
# NVIDIA CUDA
pnpm run tauri:dev:cuda
pnpm run tauri:build:cuda

# Vulkan
pnpm run tauri:dev:vulkan
pnpm run tauri:build:vulkan

# Linux ROCm/HIP
pnpm run tauri:dev:hipblas
pnpm run tauri:build:hipblas
```

On macOS, the target-specific Rust dependency enables Metal/CoreML.

## Linux dependencies

Ubuntu/Debian requires the Tauri, CPAL, and Vulkan development packages used by
the release workflows. The workflow files under `.github/workflows/` are the
canonical package list for supported Ubuntu images. In particular, keep these
audio packages available:

```bash
sudo apt install libasound2-dev libpipewire-0.3-dev libpulse-dev
```

For Vulkan builds, install the LunarG Vulkan SDK and set `VULKAN_SDK`. For CUDA
builds, install the CUDA Toolkit so `nvcc` or `CUDA_HOME` is available. For HIP
builds, install ROCm so `hipcc` or `ROCM_PATH` is available.

## Windows dependencies

Install Visual Studio 2022 Build Tools, the Windows SDK, LLVM, and either:

- CUDA Toolkit for `cuda`; or
- Vulkan SDK for `vulkan`.

The PowerShell helper scripts use Vulkan explicitly. The batch helper uses the
same GPU detector as the npm scripts.

## Runtime contract

Whisper validates the compiled backend against runtime GPU detection before
creating a model context. Parakeet registers CUDA with
`error_on_failure` and disables ONNX Runtime CPU execution-provider fallback.
An unmet GPU condition preserves model files and returns the activation action;
it never starts ASR inference on CPU.

## Validation

Before publishing a build:

```bash
cd frontend/src-tauri
cargo check --features vulkan
cargo test --features vulkan whisper_engine::acceleration
```

For an NVIDIA release candidate, use `--features cuda`. Hardware qualification
then runs a real reference-backed audio sample on the target GPU and records the
compiled backend, transcript, timing, and GPU telemetry.

See [GPU acceleration](GPU_ACCELERATION.md) for backend-specific checks.
