# GPU-only ASR Activation

Meetily speech recognition runs through a GPU backend. The application treats
GPU readiness as an activation contract rather than a performance preference.

## Whisper backend matrix

- **Metal/CoreML:** macOS
- **CUDA:** NVIDIA on Windows or Linux
- **Vulkan:** NVIDIA, AMD, or Intel on Windows or Linux
- **HIP BLAS:** AMD ROCm on Linux

Whisper model loading requires both a compiled backend and a compatible runtime
GPU detection result. The context is always created with `use_gpu=true`.

## Parakeet backend

Parakeet uses the CUDA ONNX Runtime execution provider. Session creation:

- registers CUDA with `error_on_failure`;
- sets `session.disable_cpu_ep_fallback=1`;
- returns a CUDA activation error in builds without the `cuda` feature.

This gives Parakeet a single auditable inference path.

## Automatic detection

```bash
cd frontend
node scripts/auto-detect-gpu.js
```

The command prints one feature name on standard output: `cuda`, `vulkan`,
`hipblas`, `metal`, or `coreml`. Missing drivers or SDKs produce an activation
error and a nonzero exit status.

You can choose a known backend through `TAURI_GPU_FEATURE`:

```bash
TAURI_GPU_FEATURE=cuda npm run tauri:dev
TAURI_GPU_FEATURE=vulkan npm run tauri:build
```

The Tauri wrapper validates the value against the GPU feature allowlist.

## Readiness checks

### CUDA

```bash
nvidia-smi
nvcc --version
```

### Vulkan

```bash
vulkaninfo --summary
test -n "$VULKAN_SDK"
```

### HIP

```bash
rocm-smi
hipcc --version
```

### Metal

Metal is provided by the supported macOS toolchain and selected by the
target-specific dependency configuration.

## Evidence gate

A GPU-enabled compile confirms packaging. A release-facing inference claim also
requires a real audio run on target hardware with:

- generated transcript output;
- compiled backend recorded;
- real inference timestamps;
- GPU telemetry during inference;
- visible error records for failed runs.

This separates build readiness from hardware-backed runtime evidence.

Complete implementation and live-evidence trail: [2026-07-14 Audio owner thread and GPU-only ASR audit](audit-events/2026-07-14-audio-owner-gpu-asr-hardening/audit-event.md).
