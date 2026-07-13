#!/usr/bin/env node
/**
 * Auto-detect GPU and run Tauri with appropriate features
 */

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Get the command (dev or build)
const command = process.argv[2];
if (!command || !['dev', 'build'].includes(command)) {
  console.error('Usage: node tauri-auto.js [dev|build]');
  process.exit(1);
}

// Detect GPU feature
let feature = '';
const GPU_FEATURES = new Set(['cuda', 'vulkan', 'hipblas', 'metal', 'coreml']);

// Check for environment variable override first
if (process.env.TAURI_GPU_FEATURE) {
  feature = process.env.TAURI_GPU_FEATURE;
  console.log(`🔧 Using forced GPU feature from environment: ${feature}`);
} else {
  try {
    const result = execSync('node scripts/auto-detect-gpu.js', {
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'inherit']
    });
    feature = result.trim();
  } catch (err) {
    console.error('Meetily ASR requires a GPU backend; automatic GPU detection did not produce one.');
    process.exit(err.status || 1);
  }
}

if (!GPU_FEATURES.has(feature)) {
  console.error(`Meetily ASR requires one GPU feature (${[...GPU_FEATURES].join(', ')}); received: ${feature || 'none'}.`);
  process.exit(1);
}

console.log(''); // Empty line for spacing

// Platform-specific environment variables
const platform = os.platform();
const env = { ...process.env };

if (platform === 'linux' && feature === 'cuda') {
  console.log('🐧 Linux/CUDA detected: Setting CMAKE flags for NVIDIA GPU');
  env.CMAKE_CUDA_ARCHITECTURES = process.env.CMAKE_CUDA_ARCHITECTURES || '75;86;89';
  env.CMAKE_CUDA_FLAGS = process.env.CMAKE_CUDA_FLAGS || '-Xcompiler=-fPIC';
  env.CMAKE_CUDA_STANDARD = '17';
  env.CMAKE_POSITION_INDEPENDENT_CODE = 'ON';
}

// Build the tauri command
let tauriCmd = `tauri ${command}`;
tauriCmd += ` -- --features ${feature}`;
console.log(`🚀 Running: tauri ${command} with GPU feature: ${feature}`);
console.log('');

// Execute the command
try {
  execSync(tauriCmd, { stdio: 'inherit', env });
} catch (err) {
  process.exit(err.status || 1);
}
