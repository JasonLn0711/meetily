#!/usr/bin/env node
/**
 * Auto-detect GPU and run Tauri with appropriate features
 */

const { execSync, spawnSync } = require('child_process');
const os = require('os');

const LINUX_AUDIO_DEVELOPMENT_PACKAGES = [
  ['alsa', 'libasound2-dev'],
  ['libpipewire-0.3', 'libpipewire-0.3-dev'],
  ['libpulse', 'libpulse-dev'],
];

function findMissingLinuxAudioPackages(
  probe = (name) => spawnSync('pkg-config', ['--exists', name], { stdio: 'ignore' }).status === 0
) {
  return LINUX_AUDIO_DEVELOPMENT_PACKAGES
    .filter(([name]) => !probe(name))
    .map(([, packageName]) => packageName);
}

function main() {
// Get the command (dev or build)
const command = process.argv[2];
if (!command || !['dev', 'build'].includes(command)) {
  console.error('Usage: node tauri-auto.js [dev|build] [cuda|vulkan|hipblas|metal|coreml]');
  process.exit(1);
}

if (os.platform() === 'linux') {
  const missingPackages = findMissingLinuxAudioPackages();
  if (missingPackages.length > 0) {
    console.error(`Meetily Linux audio build prerequisites need activation: ${missingPackages.join(', ')}`);
    console.error(`Ubuntu/Debian: sudo apt install ${missingPackages.join(' ')}`);
    console.error('Then rerun this command.');
    process.exit(1);
  }
  console.log('✅ Linux audio development packages ready');
}

// Detect GPU feature
let feature = '';
const GPU_FEATURES = new Set(['cuda', 'vulkan', 'hipblas', 'metal', 'coreml']);
const requestedFeature = process.argv[3] || process.env.TAURI_GPU_FEATURE;

// Check for environment variable override first
if (requestedFeature) {
  feature = requestedFeature;
  console.log(`🔧 Using requested GPU feature: ${feature}`);
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
}

module.exports = { findMissingLinuxAudioPackages };

if (require.main === module) {
  main();
}
