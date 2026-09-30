use wazoo_media::ScrollEngine;

#[test]
fn test_scroll_engine() {
    let mut engine = ScrollEngine::new(1000.0);
    engine.scroll_mode_muted = false;

    engine.add_item(1, 0.0, 500.0);
    assert_eq!(engine.calculate_player_volume(1), 1.0);

    // Player partially off top: y = -250, height = 500 -> 250 visible -> 50%
    engine.add_item(2, -250.0, 500.0);
    assert!((engine.calculate_player_volume(2) - 0.5).abs() < 0.01);

    // Needs new player because content ends at 500 while window is 1000
    assert_eq!(engine.needs_new_player(), Some(500.0));
}

#[test]
fn test_scroll_engine_init_stack() {
    let mut engine = ScrollEngine::new(1000.0);
    engine.init_stack(&[10, 20]);

    assert_eq!(engine.items.len(), 2);
    assert_eq!(engine.items.get(&10).unwrap().y_pos, 0.0);
    assert_eq!(engine.items.get(&10).unwrap().height, 500.0);
    assert_eq!(engine.items.get(&20).unwrap().y_pos, 500.0);
    assert_eq!(engine.items.get(&20).unwrap().height, 500.0);

    // Content ends at 1000, so window is completely full
    assert_eq!(engine.needs_new_player(), None);
}

#[test]
fn test_scroll_engine_recalculate_positions() {
    let mut engine = ScrollEngine::new(1000.0);
    // Add two items with gaps or misplaced offsets
    engine.add_item(1, 50.0, 400.0);
    engine.add_item(2, 600.0, 400.0);

    engine.recalculate_positions();

    // Total height = 800 < 1000, so top should be 1000 - 800 = 200
    assert_eq!(engine.items.get(&1).unwrap().y_pos, 200.0);
    assert_eq!(engine.items.get(&2).unwrap().y_pos, 600.0);
}

#[test]
fn test_scroll_engine_tick_despawn() {
    let mut engine = ScrollEngine::new(1000.0);
    engine.scroll_speed = 5.0;
    engine.add_item(1, -490.0, 500.0); // y becomes -495, bottom is 5.0 > 0.0
    engine.add_item(2, 10.0, 500.0);

    let offscreen = engine.tick();
    assert!(offscreen.is_empty());

    let mut engine2 = ScrollEngine::new(1000.0);
    engine2.scroll_speed = 10.0;
    engine2.add_item(1, -495.0, 500.0); // y will become -505, -505 + 500 = -5 < 0

    let offscreen2 = engine2.tick();
    assert_eq!(offscreen2, vec![1]);
    assert!(!engine2.items.contains_key(&1));
}

#[test]
fn test_scroll_engine_empty_spawn() {
    let engine = ScrollEngine::new(1000.0);
    assert_eq!(engine.needs_new_player(), Some(0.0));
}

#[test]
fn test_scroll_engine_lookahead_margin() {
    let mut engine = ScrollEngine::new(1000.0);
    engine.add_item(1, 0.0, 1050.0); // bottom at 1050 > 1000

    // Without margin: bottom is 1050 >= 1000, so None
    assert_eq!(engine.needs_new_player(), None);

    // With 100px margin: threshold is 1100 > 1050, so returns Some(1050.0)
    assert_eq!(engine.needs_new_player_with_margin(100.0), Some(1050.0));
}

#[test]
fn test_scroll_engine_real_aspect_ratio_heights() {
    let mut engine = ScrollEngine::with_window_size(1920.0, 1080.0);

    // 16:9 widescreen video at 1920 width -> real height is 1080.0
    let h_16_9 = engine.item_height_for_aspect_ratio(16.0 / 9.0);
    assert!((h_16_9 - 1080.0).abs() < 1.0);

    // 4:3 standard video at 1920 width -> real height is 1440.0
    let h_4_3 = engine.item_height_for_aspect_ratio(4.0 / 3.0);
    assert!((h_4_3 - 1440.0).abs() < 1.0);

    // 9:16 vertical video at 1920 width -> real height is 3413.33
    let h_9_16 = engine.item_height_for_aspect_ratio(9.0 / 16.0);
    assert!((h_9_16 - 3413.33).abs() < 1.0);

    // Initial stack with real heights
    engine.init_stack_with_heights(&[(1, h_16_9), (2, h_4_3)]);
    assert_eq!(engine.items.get(&1).unwrap().y_pos, 0.0);
    assert_eq!(engine.items.get(&1).unwrap().height, h_16_9);
    assert_eq!(engine.items.get(&2).unwrap().y_pos, h_16_9);
    assert_eq!(engine.items.get(&2).unwrap().height, h_4_3);

    // Updating height of item 1 updates position of item 2 gaplessly
    let new_h1 = 800.0;
    let changed = engine.update_height(1, new_h1);
    assert!(changed);
    engine.recalculate_positions();
    assert_eq!(engine.items.get(&1).unwrap().y_pos, 0.0);
    assert_eq!(engine.items.get(&1).unwrap().height, new_h1);
    assert_eq!(engine.items.get(&2).unwrap().y_pos, 800.0);
}

#[test]
fn test_scroll_engine_default_muted() {
    let engine = ScrollEngine::new(1000.0);
    assert!(engine.scroll_mode_muted);
    let engine_size = ScrollEngine::with_window_size(1920.0, 1080.0);
    assert!(engine_size.scroll_mode_muted);
}
