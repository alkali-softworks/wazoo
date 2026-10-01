/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Internationalization & Localization (i18n) Engine
 *
 * Provides translation lookup, placeholder interpolation, and language metadata across
 * 23 supported languages. Translation files are embedded at compile time.
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
        Language {
            code: "en",
            display_name: "English",
            is_rtl: false,
        },
        Language {
            code: "nl",
            display_name: "Nederlands",
            is_rtl: false,
        },
        Language {
            code: "es",
            display_name: "Español",
            is_rtl: false,
        },
        Language {
            code: "fr",
            display_name: "Français",
            is_rtl: false,
        },
        Language {
            code: "it",
            display_name: "Italiano",
            is_rtl: false,
        },
        Language {
            code: "pt",
            display_name: "Português",
            is_rtl: false,
        },
        Language {
            code: "de",
            display_name: "Deutsch",
            is_rtl: false,
        },
        Language {
            code: "cs",
            display_name: "Čeština",
            is_rtl: false,
        },
        Language {
            code: "pl",
            display_name: "Polski",
            is_rtl: false,
        },
        Language {
            code: "uk",
            display_name: "Українська",
            is_rtl: false,
        },
        Language {
            code: "ru",
            display_name: "Русский (Russian)",
            is_rtl: false,
        },
        Language {
            code: "sv",
            display_name: "Svenska",
            is_rtl: false,
        },
        Language {
            code: "tr",
            display_name: "Türkçe",
            is_rtl: false,
        },
        Language {
            code: "ar",
            display_name: "العربية (Arabic)",
            is_rtl: true,
        },
        Language {
            code: "he",
            display_name: "עברית (Hebrew)",
            is_rtl: true,
        },
        Language {
            code: "fa",
            display_name: "فارسی (Persian)",
            is_rtl: true,
        },
        Language {
            code: "hi",
            display_name: "हिन्दी (Hindi)",
            is_rtl: false,
        },
        Language {
            code: "bn",
            display_name: "বাংলা (Bengali)",
            is_rtl: false,
        },
        Language {
            code: "th",
            display_name: "ไทย (Thai)",
            is_rtl: false,
        },
        Language {
            code: "vi",
            display_name: "Tiếng Việt",
            is_rtl: false,
        },
        Language {
            code: "zh",
            display_name: "简体中文 (Chinese)",
            is_rtl: false,
        },
        Language {
            code: "ja",
            display_name: "日本語 (Japanese)",
            is_rtl: false,
        },
        Language {
            code: "ko",
            display_name: "한국어 (Korean)",
            is_rtl: false,
        },
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
    ("nl", include_str!("../locales/nl.json")),
    ("es", include_str!("../locales/es.json")),
    ("fr", include_str!("../locales/fr.json")),
    ("it", include_str!("../locales/it.json")),
    ("pt", include_str!("../locales/pt.json")),
    ("de", include_str!("../locales/de.json")),
    ("cs", include_str!("../locales/cs.json")),
    ("pl", include_str!("../locales/pl.json")),
    ("uk", include_str!("../locales/uk.json")),
    ("ru", include_str!("../locales/ru.json")),
    ("sv", include_str!("../locales/sv.json")),
    ("tr", include_str!("../locales/tr.json")),
    ("ar", include_str!("../locales/ar.json")),
    ("he", include_str!("../locales/he.json")),
    ("fa", include_str!("../locales/fa.json")),
    ("hi", include_str!("../locales/hi.json")),
    ("bn", include_str!("../locales/bn.json")),
    ("th", include_str!("../locales/th.json")),
    ("vi", include_str!("../locales/vi.json")),
    ("zh", include_str!("../locales/zh.json")),
    ("ja", include_str!("../locales/ja.json")),
    ("ko", include_str!("../locales/ko.json")),
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

pub fn get_dictionary() -> &'static HashMap<&'static str, HashMap<String, String>> {
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
