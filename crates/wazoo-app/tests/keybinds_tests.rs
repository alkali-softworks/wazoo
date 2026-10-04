use iced::keyboard::{Key, key::Named};
use wazoo_app::keybinds::find_key_action;
use wazoo_core::{KeyAction, KeybindSettings};

#[test]
fn test_find_key_action_defaults() {
    let kb = KeybindSettings::default();

    assert_eq!(
        find_key_action(&kb, &Key::Character("n".into()), false),
        Some(KeyAction::AddNewPlayer)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("N".into()), false),
        Some(KeyAction::AddNewPlayer)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("j".into()), false),
        Some(KeyAction::SearchVideos)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("/".into()), false),
        Some(KeyAction::SearchVideos)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Named(Named::Space), false),
        Some(KeyAction::TogglePlayPause)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Named(Named::ArrowUp), false),
        Some(KeyAction::NextVideo)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Named(Named::ArrowDown), false),
        Some(KeyAction::PrevVideo)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("x".into()), true),
        Some(KeyAction::CloseApp)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("y".into()), false),
        Some(KeyAction::ToggleHistory)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("h".into()), false),
        Some(KeyAction::ToggleFilePicker)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("H".into()), false),
        Some(KeyAction::ToggleFilePicker)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("f".into()), false),
        Some(KeyAction::ToggleFilePicker)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("F".into()), false),
        Some(KeyAction::ToggleFilePicker)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character(",".into()), false),
        Some(KeyAction::PrevFrame)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("<".into()), false),
        Some(KeyAction::PrevFrame)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character(".".into()), false),
        Some(KeyAction::NextFrame)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character(">".into()), false),
        Some(KeyAction::NextFrame)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("i".into()), false),
        Some(KeyAction::MarkIn)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("I".into()), false),
        Some(KeyAction::MarkIn)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("o".into()), false),
        Some(KeyAction::MarkOut)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("O".into()), false),
        Some(KeyAction::MarkOut)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("i".into()), true),
        Some(KeyAction::ClearMarkIn)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("I".into()), true),
        Some(KeyAction::ClearMarkIn)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("o".into()), true),
        Some(KeyAction::ClearMarkOut)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("O".into()), true),
        Some(KeyAction::ClearMarkOut)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Named(Named::F1), false),
        Some(KeyAction::OpenHelp)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Named(Named::F2), false),
        Some(KeyAction::OpenSettings)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("7".into()), false),
        Some(KeyAction::ToggleFilters)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("8".into()), false),
        Some(KeyAction::ToggleCube)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("9".into()), false),
        Some(KeyAction::ToggleCubeOverlay)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Named(Named::F11), false),
        Some(KeyAction::ToggleFullscreen)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("w".into()), false),
        Some(KeyAction::FitWindow)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("W".into()), false),
        Some(KeyAction::FitWindow)
    );
    assert_eq!(find_key_action(&kb, &Key::Named(Named::F7), false), None);
    // Ordinary keys when Alt is pressed should NOT match
    assert_eq!(
        find_key_action(&kb, &Key::Character("n".into()), true),
        None
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("w".into()), true),
        None
    );
}

#[test]
fn test_find_key_action_custom() {
    let kb = KeybindSettings {
        add_player: "p".to_string(),
        fit_window: "k".to_string(),
        ..Default::default()
    };

    assert_eq!(
        find_key_action(&kb, &Key::Character("n".into()), false),
        None
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("p".into()), false),
        Some(KeyAction::AddNewPlayer)
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("w".into()), false),
        None
    );
    assert_eq!(
        find_key_action(&kb, &Key::Character("k".into()), false),
        Some(KeyAction::FitWindow)
    );
}

#[test]
fn test_help_modal_search_flow() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // 1. Open help modal
    let _ = app.update(wazoo_app::message::Message::OpenHelpModal);
    assert!(app.modals.help);
    assert!(app.modals.help_search.is_empty());

    // 2. Type search query
    let _ = app.update(wazoo_app::message::Message::HelpSearchChanged("volume".to_string()));
    assert_eq!(app.modals.help_search, "volume");

    // 3. Clear search query
    let _ = app.update(wazoo_app::message::Message::ClearHelpSearch);
    assert!(app.modals.help_search.is_empty());

    // 4. Type another search query and close modal
    let _ = app.update(wazoo_app::message::Message::HelpSearchChanged("mute".to_string()));
    assert_eq!(app.modals.help_search, "mute");

    let _ = app.update(wazoo_app::message::Message::CloseHelpModal);
    assert!(!app.modals.help);
    assert!(
        app.modals.help_search.is_empty(),
        "Closing help modal must clear search query"
    );
}

#[test]
fn test_toggle_subtitles_toast() {
    let (mut app, _) = wazoo_app::app::new_test_app();

    // Default subtitles_enabled is true
    assert!(app.subtitles_enabled);

    // Toggle off -> Subtitles disabled
    let _ = app.update(wazoo_app::message::Message::ToggleSubtitles);
    assert!(!app.subtitles_enabled);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Subtitles disabled")
    );

    // Toggle back on -> Subtitles enabled
    let _ = app.update(wazoo_app::message::Message::ToggleSubtitles);
    assert!(app.subtitles_enabled);
    assert_eq!(
        app.overlay.toast_message.as_deref(),
        Some("Subtitles enabled")
    );
}
