use std::collections::HashMap;
use wazoo_core::i18n::{get_dictionary, t, t_with, Language};

#[test]
fn test_all_16_languages_parsed() {
    let dict: &'static HashMap<&'static str, HashMap<String, String>> = get_dictionary();
    assert_eq!(dict.len(), 16);
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
    assert!(!Language::is_rtl_code("en"));
    assert!(!Language::is_rtl_code("es"));
    assert!(!Language::is_rtl_code("ja"));
}
