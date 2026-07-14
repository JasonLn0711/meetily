import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { describe, test } from 'node:test';

const require = createRequire(import.meta.url);
const { findMissingLinuxAudioPackages } = require('../../scripts/tauri-auto.js');

describe('Tauri Linux audio prerequisite preflight', () => {
  test('passes when every pkg-config dependency is available', () => {
    assert.deepEqual(findMissingLinuxAudioPackages(() => true), []);
  });

  test('returns actionable Ubuntu packages for missing dependencies', () => {
    const available = new Set(['alsa']);

    assert.deepEqual(
      findMissingLinuxAudioPackages((name) => available.has(name)),
      ['libpipewire-0.3-dev', 'libpulse-dev'],
    );
  });
});
