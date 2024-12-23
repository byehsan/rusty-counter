use rusty_counter::Counter;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

#[test]
fn test_initialize_counter_from_empty_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("data.bin");
    let backup_path = temp_dir.path().join("backup.bin");

    // Create an empty file
    File::create(&file_path).expect("Failed to create file");

    // Create the counter instance
    let counter = Counter::new(file_path.to_str().unwrap(), backup_path.to_str().unwrap());

    // Initialize the counter
    assert!(
        counter.load_or_initialize(),
        "Counter should initialize successfully"
    );

    // Verify the counter's value
    assert_eq!(
        counter.get(),
        0,
        "The counter value should be initialized to '0'"
    );
}

// Test case to ensure the backup file is used when the main file is invalid
#[test]
fn test_initialize_counter_from_backup() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("data.bin");
    let backup_path = temp_dir.path().join("backup.bin");

    // Create a backup file with some value
    let mut backup_file = File::create(&backup_path).expect("Failed to create backup file");
    backup_file
        .write_all(b"1234")
        .expect("Failed to write to backup file");

    // Create the counter instance
    let counter = Counter::new(file_path.to_str().unwrap(), backup_path.to_str().unwrap());

    // Initialize the counter (it should read from the backup file)
    assert!(
        counter.load_or_initialize(),
        "Counter should initialize successfully from backup"
    );

    // Verify the counter's value
    assert_eq!(
        counter.get(),
        1234,
        "The counter value should be loaded from the backup file"
    );
}

// Test case for counter value after incrementing and decrementing
#[test]
fn test_increment_and_decrement() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("data.bin");
    let backup_path = temp_dir.path().join("backup.bin");

    // Create the counter instance
    let counter = Counter::new(file_path.to_str().unwrap(), backup_path.to_str().unwrap());

    // Initialize the counter
    assert!(
        counter.load_or_initialize(),
        "Counter should initialize successfully"
    );

    // Increment the counter
    counter.increment();
    assert_eq!(
        counter.get(),
        1,
        "The counter value should be '1' after incrementing"
    );

    // Decrement the counter
    counter.decrement();
    assert_eq!(
        counter.get(),
        0,
        "The counter value should be '0' after decrementing"
    );
}
#[test]
fn test_concurrent_increment_and_decrement() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("data.bin");
    let backup_path = temp_dir.path().join("backup.bin");

    // Create the counter instance
    let counter = Arc::new(Counter::new(
        file_path.to_str().unwrap(),
        backup_path.to_str().unwrap(),
    ));

    // Initialize the counter
    counter.load_or_initialize();

    // Spawn multiple threads to increment and decrement the counter concurrently
    let counter_clone_1 = Arc::clone(&counter);
    let counter_clone_2 = Arc::clone(&counter);
    let counter_clone_3 = Arc::clone(&counter);

    let handles: Vec<_> = vec![
        thread::spawn(move || {
            for _ in 0..1000 {
                counter_clone_1.increment();
            }
        }),
        thread::spawn(move || {
            for _ in 0..1000 {
                counter_clone_2.increment();
            }
        }),
        thread::spawn(move || {
            for _ in 0..500 {
                counter_clone_3.decrement();
            }
        }),
    ]
    .into_iter()
    .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // After 2000 increments and 500 decrements, the counter should be 1500
    assert_eq!(
        counter.get(),
        1500,
        "The counter value should be '1500' after concurrent operations"
    );
}
#[test]
fn test_corrupted_file() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("data.bin");
    let backup_path = temp_dir.path().join("backup.bin");

    // Create a file with corrupted content (non-integer value)
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"corrupted_data")
        .expect("Failed to write to file");

    // Create the counter instance
    let counter = Counter::new(file_path.to_str().unwrap(), backup_path.to_str().unwrap());

    // Initialize the counter (it should fail to parse the file and use the backup)
    assert!(
        counter.load_or_initialize(),
        "Counter should initialize from backup on corrupted file"
    );

    // Verify the counter's value (it should load from the backup, which is empty in this case)
    assert_eq!(
        counter.get(),
        0,
        "The counter value should be '0' after loading from backup"
    );
}

// Edge case: Test file permission error (e.g., if the directory is read-only)
#[test]
fn test_permission_error() {
    // Create a temporary directory for testing
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("data.bin");

    // Set the directory to read-only (to simulate permission error)
    std::fs::set_permissions(temp_dir.path(), std::fs::Permissions::from_mode(0o444))
        .expect("Failed to set permissions");

    // Create the counter instance
    let counter = Counter::new(file_path.to_str().unwrap(), file_path.to_str().unwrap());

    // Try to load the counter, expecting a failure due to permission issues
    assert!(
        !counter.load_or_initialize(),
        "Counter should fail to initialize due to permission error"
    );
}
