#!/usr/bin/env node
/**
 * Auto-detect GPU capabilities and set appropriate features
 * Used by npm scripts to automatically enable hardware acceleration
 */

const { execSync } = require('child_process');
const os = require('os');

function commandExists(cmd) {
  try {
    execSync(`${os.platform() === 'win32' ? 'where' : 'which'} ${cmd}`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function detectGPU() {
  const platform = os.platform();

  // macOS: Metal is always available, check for Apple Silicon for CoreML
  if (platform === 'darwin') {
    const arch = os.arch();
    if (arch === 'arm64') {
      console.log('🍎 Apple Silicon detected - using Metal + CoreML');
      return 'coreml'; // CoreML includes Metal
    } else {
      console.log('🍎 macOS Intel detected - using Metal');
      return 'metal';
    }
  }

  // Windows/Linux: Check for GPUs
  if (platform === 'win32' || platform === 'linux') {
    // Check for NVIDIA GPU
    if (commandExists('nvidia-smi')) {
      const cudaPath = process.env.CUDA_PATH;
      if (cudaPath || commandExists('nvcc')) {
        console.log('🟢 NVIDIA GPU detected with CUDA - using CUDA acceleration');
        return 'cuda';
      } else {
        throw new Error('NVIDIA GPU detected, and ASR activation requires the CUDA Toolkit (CUDA_PATH or nvcc).');
      }
    }

    // Check for AMD GPU (Linux only)
    if (platform === 'linux' && commandExists('rocm-smi')) {
      const rocmPath = process.env.ROCM_PATH;
      if (rocmPath || commandExists('hipcc')) {
        console.log('🔴 AMD GPU detected with ROCm - using HIPBlas acceleration');
        return 'hipblas';
      } else {
        throw new Error('AMD GPU detected, and HIP ASR activation requires ROCm (ROCM_PATH or hipcc).');
      }
    }

    // Check for Vulkan
    if (commandExists('vulkaninfo') || (platform === 'win32' && require('fs').existsSync('C:\\VulkanSDK'))) {
      const vulkanSdk = process.env.VULKAN_SDK;
      if (vulkanSdk) {
        console.log('🔵 Vulkan detected with all dependencies - using Vulkan acceleration');
        return 'vulkan';
      } else {
        throw new Error('Vulkan GPU detected, and ASR activation requires VULKAN_SDK.');
      }
    }
  }

  throw new Error('No supported GPU backend detected. Meetily ASR requires CUDA, Vulkan, HIP, or Metal.');
}

// Redirect console.log to stderr so only the feature goes to stdout
const originalLog = console.log;
console.log = (...args) => {
  process.stderr.write(args.join(' ') + '\n');
};

// Detect and output the feature
const feature = detectGPU();

// Restore console.log
console.log = originalLog;

// Only write the feature to stdout (no newline, no extra text)
if (feature) {
  process.stdout.write(feature);
}
