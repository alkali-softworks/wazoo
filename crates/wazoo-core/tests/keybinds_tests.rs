use wazoo_core::keybinds::KeybindSettings;
use wazoo_core::models::WazooSettings;

#[test]
fn test_menu_hint() {
    let kb = KeybindSettings::default();
    assert_eq!(kb.menu_hint(&kb.add_player), "N");
    assert_eq!(kb.menu_hint(&kb.search_videos), "J");
    assert_eq!(kb.menu_hint(&kb.toggle_layout), "L");
    assert_eq!(kb.menu_hint(&kb.toggle_file_picker), "H");
    assert_eq!(kb.menu_hint(&kb.toggle_transcript), "V");
    assert_eq!(kb.menu_hint(&kb.toggle_bookmarks), "B");
    assert_eq!(kb.menu_hint(&kb.toggle_history), "Y");
    assert_eq!(kb.menu_hint(&kb.close_app), "Alt+X");
    assert_eq!(kb.menu_hint(&kb.toggle_pin), "P");
}

#[test]
fn test_help_shortcuts_generation() {
    let kb = KeybindSettings::default();
    let shortcuts = kb.help_shortcuts_with_descriptions(|key| key.to_string());
    assert_eq!(shortcuts.len(), 23);
    assert_eq!(
        shortcuts[0],
        (
            "j OR /".to_string(),
            "help.shortcuts.search_videos".to_string()
        )
    );
    assert_eq!(
        shortcuts[7],
        ("n".to_string(), "help.shortcuts.add_player".to_string())
    );
    assert_eq!(
        shortcuts[12],
        ("y".to_string(), "help.shortcuts.toggle_history".to_string())
    );
    assert_eq!(
        shortcuts[13],
        (
            "+ / =".to_string(),
            "bookmarks.bookmark_current".to_string()
        )
    );
    assert_eq!(
        shortcuts[20],
        (
            "P / Alt+P".to_string(),
            "help.shortcuts.toggle_pin".to_string()
        )
    );
    assert_eq!(
        shortcuts[21],
        (
            "Alt + X".to_string(),
            "help.shortcuts.close_app".to_string()
        )
    );
}

#[test]
fn test_help_categories_generation() {
    let kb = KeybindSettings::default();
    let categories = kb.help_categories(|key| key.to_string());
    assert_eq!(categories.len(), 5);
    assert_eq!(categories[0].title, "help.categories.playback");
    assert_eq!(categories[1].title, "help.categories.drawers");
    assert_eq!(categories[2].title, "help.categories.layout");
    assert_eq!(categories[3].title, "help.categories.audio");
    assert_eq!(categories[4].title, "help.categories.system");

    // Drawer category contains both H and F for file drawer
    let file_drawer_item = &categories[1].shortcuts[0];
    assert_eq!(file_drawer_item.description, "help.shortcuts.toggle_file_picker");
    match &file_drawer_item.key {
        wazoo_core::KeyDisplay::Alternatives(keys) => {
            assert_eq!(keys, &vec!["H".to_string(), "F".to_string()]);
        }
        other => panic!("Expected Alternatives, got {:?}", other),
    }
}

#[test]
fn test_is_complete_json_and_reconcile_with_defaults() {
    // Incomplete json missing most keys
    let incomplete_json: serde_json::Value = serde_json::json!({
        "keybinds": {
            "add_player": "p"
        }
    });
    assert!(!KeybindSettings::is_complete_json(&incomplete_json));

    // Incomplete json with blank value
    let mut full_map = serde_json::Map::new();
    for &k in KeybindSettings::ALL_KEYS {
        full_map.insert(k.to_string(), serde_json::Value::String("x".to_string()));
    }
    full_map.insert(
        "add_player".to_string(),
        serde_json::Value::String("   ".to_string()),
    );
    let blank_json = serde_json::json!({ "keybinds": full_map });
    assert!(!KeybindSettings::is_complete_json(&blank_json));

    // Reconcile blank value
    let mut kb = KeybindSettings {
        add_player: "  ".to_string(),
        ..Default::default()
    };
    kb.reconcile_with_defaults();
    assert_eq!(kb.add_player, "n");

    // Complete json with non-empty values
    let complete_json = serde_json::to_value(WazooSettings::default()).unwrap();
    assert!(KeybindSettings::is_complete_json(&complete_json));
}
