use std::collections::HashMap;
use wazoo_core::i18n::{Language, get_dictionary, t, t_with};

#[test]
fn test_all_23_languages_parsed() {
    let dict: &'static HashMap<&'static str, HashMap<String, String>> = get_dictionary();
    assert_eq!(dict.len(), 23);
    for lang in Language::ALL {
        assert!(
            dict.contains_key(lang.code),
            "Missing locale for {}",
            lang.code
        );
        let map = dict.get(lang.code).unwrap();
        assert!(!map.is_empty(), "Locale map empty for {}", lang.code);
        assert!(
            map.contains_key("common.settings"),
            "Missing common.settings in {}",
            lang.code
        );
        assert!(
            map.contains_key("common.or"),
            "Missing common.or in {}",
            lang.code
        );
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
    assert_eq!(t("pl", "common.settings"), "Ustawienia");
    assert_eq!(t("tr", "common.settings"), "Ayarlar");
    assert_eq!(t("nl", "common.settings"), "Instellingen");
    assert_eq!(t("sv", "common.settings"), "Inställningar");
    assert_eq!(t("cs", "common.settings"), "Nastavení");
    assert_eq!(t("uk", "common.settings"), "Налаштування");
    assert_eq!(t("fa", "common.settings"), "تنظیمات");
    assert_eq!(t("vi", "common.settings"), "Cài đặt");

    // Non-existent key falls back to key itself
    assert_eq!(t("es", "nonexistent.key.test"), "nonexistent.key.test");
}

#[test]
fn test_t_with_interpolation() {
    let formatted = t_with(
        "en",
        "wazoo.loading_progress",
        &[("percent", "50"), ("total", "100")],
    );
    assert_eq!(formatted, "Loading... 50% (100 files)");

    let formatted_es = t_with(
        "es",
        "wazoo.loading_progress",
        &[("percent", "50"), ("total", "100")],
    );
    assert_eq!(formatted_es, "Cargando... 50% (100 archivos)");
}

#[test]
fn test_rtl_detection() {
    assert!(Language::is_rtl_code("ar"));
    assert!(Language::is_rtl_code("he"));
    assert!(Language::is_rtl_code("fa"));
    assert!(!Language::is_rtl_code("en"));
    assert!(!Language::is_rtl_code("es"));
    assert!(!Language::is_rtl_code("ja"));
    assert!(!Language::is_rtl_code("nl"));
    assert!(!Language::is_rtl_code("vi"));
}

#[test]
fn test_all_locales_have_full_key_parity() {
    let dict = get_dictionary();
    let en_keys: std::collections::HashSet<_> =
        dict.get("en").expect("en locale exists").keys().collect();

    for lang in Language::ALL {
        let lang_keys: std::collections::HashSet<_> =
            dict.get(lang.code).expect("locale exists").keys().collect();
        let missing: Vec<_> = en_keys.difference(&lang_keys).collect();
        let extra: Vec<_> = lang_keys.difference(&en_keys).collect();
        assert!(
            missing.is_empty(),
            "Locale '{}' is missing keys: {:?}",
            lang.code,
            missing
        );
        assert!(
            extra.is_empty(),
            "Locale '{}' has extra keys: {:?}",
            lang.code,
            extra
        );
    }
}
