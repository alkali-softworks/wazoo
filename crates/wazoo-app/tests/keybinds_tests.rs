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
    // Ordinary keys when Alt is pressed should NOT match
    assert_eq!(
        find_key_action(&kb, &Key::Character("n".into()), true),
        None
    );
}

#[test]
fn test_find_key_action_custom() {
    let kb = KeybindSettings {
        add_player: "p".to_string(),
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
}
