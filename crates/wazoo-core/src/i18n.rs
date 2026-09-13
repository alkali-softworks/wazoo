/*!
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Internationalization & Localization (i18n) Engine
 * 
 * Provides translation lookup, placeholder interpolation, and language metadata across
 * 16 supported languages matching wazoo-desktop. Translation files are embedded at compile time.
 */

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    pub code: &'static str,
    pub display_name: &'static str,
    pub is_rtl: bool,
}

impl Language {
    pub const ALL: &'static [Language] = &[
        Language { code: "en", display_name: "English", is_rtl: false },
        Language { code: "es", display_name: "Español", is_rtl: false },
        Language { code: "fr", display_name: "Français", is_rtl: false },
        Language { code: "de", display_name: "Deutsch", is_rtl: false },
        Language { code: "it", display_name: "Italiano", is_rtl: false },
        Language { code: "pt", display_name: "Português", is_rtl: false },
        Language { code: "ru", display_name: "Русский (Russian)", is_rtl: false },
        Language { code: "zh", display_name: "简体中文 (Chinese)", is_rtl: false },
        Language { code: "ja", display_name: "日本語 (Japanese)", is_rtl: false },
        Language { code: "ko", display_name: "한국어 (Korean)", is_rtl: false },
        Language { code: "th", display_name: "ไทย (Thai)", is_rtl: false },
        Language { code: "he", display_name: "עברית (Hebrew)", is_rtl: true },
        Language { code: "ar", display_name: "العربية (Arabic)", is_rtl: true },
        Language { code: "hi", display_name: "हिन्दी (Hindi)", is_rtl: false },
        Language { code: "bn", display_name: "বাংলা (Bengali)", is_rtl: false },
        Language { code: "id", display_name: "Bahasa Indonesia", is_rtl: false },
    ];

    pub fn from_code(code: &str) -> Language {
        Self::ALL
            .iter()
            .copied()
            .find(|l| l.code.eq_ignore_ascii_case(code))
            .unwrap_or(Self::ALL[0])
    }

    pub fn is_rtl_code(code: &str) -> bool {
        Self::from_code(code).is_rtl
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

// Compile-time embedded JSON files
const RAW_LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.json")),
    ("es", include_str!("../locales/es.json")),
    ("fr", include_str!("../locales/fr.json")),
    ("de", include_str!("../locales/de.json")),
    ("it", include_str!("../locales/it.json")),
    ("pt", include_str!("../locales/pt.json")),
    ("ru", include_str!("../locales/ru.json")),
    ("zh", include_str!("../locales/zh.json")),
    ("ja", include_str!("../locales/ja.json")),
    ("ko", include_str!("../locales/ko.json")),
    ("th", include_str!("../locales/th.json")),
    ("he", include_str!("../locales/he.json")),
    ("ar", include_str!("../locales/ar.json")),
    ("hi", include_str!("../locales/hi.json")),
    ("bn", include_str!("../locales/bn.json")),
    ("id", include_str!("../locales/id.json")),
];

static DICTIONARY: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();

fn flatten_json(prefix: &str, value: &serde_json::Value, map: &mut HashMap<String, String>) {
    match value {
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_json(&new_key, v, map);
            }
        }
        serde_json::Value::String(s) => {
            map.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}

fn get_dictionary() -> &'static HashMap<&'static str, HashMap<String, String>> {
    DICTIONARY.get_or_init(|| {
        let mut dict = HashMap::new();
        for &(code, raw_json) in RAW_LOCALES {
            let mut lang_map = HashMap::new();
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw_json) {
                flatten_json("", &value, &mut lang_map);
            }
            dict.insert(code, lang_map);
        }
        dict
    })
}

/// Translate a key into the given language code, falling back to English ("en"), then the key itself.
pub fn t(lang: &str, key: &str) -> String {
    let dict = get_dictionary();
    let lang_code = lang.to_lowercase();

    // 1. Try target language
    if let Some(map) = dict.get(lang_code.as_str()) {
        if let Some(text) = map.get(key) {
            return text.clone();
        }
    }

    // 2. Fallback to English
    if lang_code != "en" {
        if let Some(map) = dict.get("en") {
            if let Some(text) = map.get(key) {
                return text.clone();
            }
        }
    }

    // 3. Fallback to raw key
    key.to_string()
}

/// Translate a key and interpolate placeholder parameters `{param}` with provided values.
pub fn t_with(lang: &str, key: &str, args: &[(&str, &str)]) -> String {
    let mut text = t(lang, key);
    for &(param, val) in args {
        let placeholder = format!("{{{param}}}");
        text = text.replace(&placeholder, val);
    }
    text
}

/// Returns true if the given folder name represents the "All" media folders option in any language or default sentinel.
pub fn is_all_folder(folder: &str) -> bool {
    let trimmed = folder.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return true;
    }
    for lang in Language::ALL {
        if trimmed == t(lang.code, "common.all") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_16_languages_parsed() {
        let dict = get_dictionary();
        assert_eq!(dict.len(), 16);
        for lang in Language::ALL {
            assert!(dict.contains_key(lang.code), "Missing locale for {}", lang.code);
            let map = dict.get(lang.code).unwrap();
            assert!(!map.is_empty(), "Locale map empty for {}", lang.code);
            assert!(map.contains_key("common.settings"), "Missing common.settings in {}", lang.code);
        }
    }

    #[test]
    fn test_translations_and_fallbacks() {
        assert_eq!(t("en", "common.settings"), "Settings");
        assert_eq!(t("es", "common.settings"), "Configuración");
        assert_eq!(t("fr", "common.settings"), "Paramètres");
        assert_eq!(t("ja", "common.settings"), "設定");
        assert_eq!(t("zh", "common.settings"), "设置");
        assert_eq!(t("de", "common.settings"), "Einstellungen");
        assert_eq!(t("ar", "common.settings"), "الإعدادات");

        // Non-existent key falls back to key itself
        assert_eq!(t("es", "nonexistent.key.test"), "nonexistent.key.test");
    }

    #[test]
    fn test_t_with_interpolation() {
        let formatted = t_with("en", "wazoo.loading_progress", &[("percent", "50"), ("total", "100")]);
        assert_eq!(formatted, "Loading... 50% (100 files)");

        let formatted_es = t_with("es", "wazoo.loading_progress", &[("percent", "50"), ("total", "100")]);
        assert_eq!(formatted_es, "Cargando... 50% (100 archivos)");
    }

    #[test]
    fn test_rtl_detection() {
        assert!(Language::is_rtl_code("ar"));
        assert!(Language::is_rtl_code("he"));
        assert!(!Language::is_rtl_code("en"));
        assert!(!Language::is_rtl_code("es"));
        assert!(!Language::is_rtl_code("ja"));
    }

    #[test]
    fn test_transcript_and_bookmarks_translations() {
        assert_eq!(t("en", "transcript.title"), "Transcript");
        assert_eq!(t("es", "transcript.title"), "Transcripción");
        assert_eq!(t("ja", "transcript.title"), "トランスクリプト");
        assert_eq!(t("zh", "transcript.title"), "字幕记录");

        assert_eq!(t("en", "transcript.loading_subtitles"), "Loading embedded subtitles");
        assert_eq!(t("es", "transcript.loading_subtitles"), "Cargando subtítulos integrados");

        assert_eq!(t("en", "bookmarks.title"), "Bookmarks");
        assert_eq!(t("es", "bookmarks.title"), "Marcadores");
        assert_eq!(t("ja", "bookmarks.title"), "ブックマーク");

        assert_eq!(t("en", "bookmarks.not_found"), "No bookmark found for current video");
        assert_eq!(t("es", "bookmarks.not_found"), "No se encontró ningún marcador para el video actual");
        assert_eq!(t_with("en", "bookmarks.added", &[("name", "Test")]), "Added bookmark: Test");
        assert_eq!(t_with("es", "bookmarks.added", &[("name", "Test")]), "Marcador añadido: Test");
    }

    #[test]
    fn test_is_all_folder() {
        assert!(is_all_folder(""));
        assert!(is_all_folder("All"));
        assert!(is_all_folder("all"));
        assert!(is_all_folder("ALL"));
        assert!(is_all_folder("Todo")); // Spanish
        assert!(is_all_folder("Alle")); // German
        assert!(is_all_folder("Tout")); // French
        assert!(is_all_folder("すべて")); // Japanese
        assert!(!is_all_folder("anime"));
        assert!(!is_all_folder("/media/movies"));
    }
}
