//! Product-facing capability contract for the bundled Parakeet models.

/// ISO 639-1 codes supported by Parakeet TDT 0.6B v3.
pub const PARAKEET_V3_LANGUAGE_CODES: &[&str] = &[
    "bg", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "de", "el", "hu", "it", "lv", "lt", "mt",
    "pl", "pt", "ro", "sk", "sl", "es", "sv", "ru", "uk",
];

/// Validate a requested language before loading or running a Parakeet model.
///
/// `auto` is accepted because v3 detects languages within its supported set.
/// Translation is intentionally rejected: Parakeet returns transcription in
/// the detected source language.
pub fn validate_language(model: &str, language: Option<&str>) -> Result<(), String> {
    let language = language.unwrap_or("auto").trim().to_ascii_lowercase();

    if language.is_empty() || language == "auto" {
        return Ok(());
    }

    if language == "auto-translate" {
        return Err(
            "Parakeet transcribes supported source languages in place. Choose Local Whisper for translation to English."
                .to_string(),
        );
    }

    let code = language
        .split(['-', '_'])
        .next()
        .unwrap_or(language.as_str());

    if model.contains("-v2-") {
        return (code == "en").then_some(()).ok_or_else(|| {
            format!(
                "Parakeet v2 supports English transcription. Choose Parakeet v3 for its 25 European languages or Local Whisper for '{}'.",
                language
            )
        });
    }

    PARAKEET_V3_LANGUAGE_CODES
        .contains(&code)
        .then_some(())
        .ok_or_else(|| {
            format!(
                "Parakeet v3 supports automatic transcription across 25 European languages. Choose Local Whisper for '{}'.",
                language
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const V3: &str = "parakeet-tdt-0.6b-v3-int8";
    const V2: &str = "parakeet-tdt-0.6b-v2-int8";

    #[test]
    fn v3_accepts_auto_and_supported_locale() {
        assert!(validate_language(V3, Some("auto")).is_ok());
        assert!(validate_language(V3, Some("pt-BR")).is_ok());
    }

    #[test]
    fn v3_routes_taiwan_mandarin_to_local_whisper() {
        let error = validate_language(V3, Some("zh-TW")).unwrap_err();
        assert!(error.contains("Local Whisper"));
    }

    #[test]
    fn parakeet_does_not_claim_translation() {
        assert!(validate_language(V3, Some("auto-translate")).is_err());
    }

    #[test]
    fn v2_is_english_only() {
        assert!(validate_language(V2, Some("en")).is_ok());
        assert!(validate_language(V2, Some("es")).is_err());
    }
}
