use wazoo_tutorial::db::TutorialDatabase;

#[test]
fn test_in_memory_db_crud() {
    // Create an in-memory database
    let db = TutorialDatabase::open_in_memory().expect("Failed to create in-memory database");

    // Assert initial count is 0
    assert_eq!(db.count_notes().unwrap(), 0);

    // Add a note
    let id1 = db
        .add_note("Learn Rust", "Master ownership and borrowing")
        .unwrap();
    assert!(id1 > 0);

    // Add a second note
    let _id2 = db
        .add_note("Learn Iced", "Understand The Elm Architecture")
        .unwrap();
    assert_eq!(db.count_notes().unwrap(), 2);

    // Fetch notes and verify content
    let notes = db.get_all_notes().unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].title, "Learn Iced"); // Most recent first
    assert_eq!(notes[1].title, "Learn Rust");

    // Delete note 1
    let deleted = db.delete_note(id1).unwrap();
    assert_eq!(deleted, 1);
    assert_eq!(db.count_notes().unwrap(), 1);
}
