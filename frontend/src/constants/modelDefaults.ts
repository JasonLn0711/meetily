/**
 * Default model names for transcription engines.
 * IMPORTANT: Keep in sync with Rust constants in src-tauri/src/config.rs
 */

/**
 * Default Whisper model for transcription when no preference is configured.
 * This is the Taiwan-focused Breeze ASR model used by localWhisper.
 */
export const DEFAULT_WHISPER_MODEL = 'breeze-asr-26';

/**
 * Default Parakeet model for transcription when no preference is configured.
 * This is the quantized version optimized for speed.
 */
export const DEFAULT_PARAKEET_MODEL = 'parakeet-tdt-0.6b-v3-int8';

/**
 * Model defaults by provider type
 */
export const MODEL_DEFAULTS = {
  whisper: DEFAULT_WHISPER_MODEL,
  localWhisper: DEFAULT_WHISPER_MODEL,
  parakeet: DEFAULT_PARAKEET_MODEL,
} as const;
