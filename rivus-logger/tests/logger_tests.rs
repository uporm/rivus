use rivus_logger::LoggerConfig;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[test]
fn test_log_cleanup_scenarios() {
    let dir = "./target/cleanup_scenarios_logs";
    // Clean up before start
    if Path::new(dir).exists() {
        std::fs::remove_dir_all(dir).unwrap();
    }
    std::fs::create_dir_all(dir).unwrap();

    let file_name = "test_scenarios.log";

    // ----------------------------------------------------------------
    // Scenario 1: Cleanup on start
    // ----------------------------------------------------------------
    // Create dummy files (5 files)
    let dates = vec![
        "2023-10-20",
        "2023-10-21",
        "2023-10-22",
        "2023-10-23",
        "2023-10-24",
    ];

    for date in &dates {
        File::create(format!("{}/{}.{}", dir, file_name, date)).unwrap();
    }

    // Configure logger
    // max_files: 3
    // cleanup_period: 1 second
    let _guard = LoggerConfig::new()
        .log_dir(dir)
        .file_prefix(file_name)
        .max_files(3)
        .cleanup_interval(Duration::from_secs(1))
        .enable_console(false)
        .init();

    // Give the cleanup thread some time for initial cleanup
    thread::sleep(Duration::from_millis(500));

    // Verify initial cleanup
    let files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    // Should have <= 3 (plus active log)
    // Oldest "2023-10-20" should be gone
    let old_file = format!("{}.{}", file_name, "2023-10-20");
    assert!(
        !files.contains(&old_file),
        "Scenario 1: Oldest file {} should be deleted on start",
        old_file
    );

    // ----------------------------------------------------------------
    // Scenario 2: Periodic cleanup
    // ----------------------------------------------------------------
    // Now we have ~3 files. Let's add more to exceed the limit again.
    // Use newer dates.
    let new_dates = vec!["2023-10-25", "2023-10-26", "2023-10-27"];
    for date in &new_dates {
        File::create(format!("{}/{}.{}", dir, file_name, date)).unwrap();
    }

    // Now we have significantly more files.
    // Wait for the next period (1s + buffer)
    thread::sleep(Duration::from_millis(1500));

    // Verify periodic cleanup
    let files_after: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    // The oldest from the remaining previous set (likely 2023-10-22) should now be gone
    // because we added 25, 26, 27.
    // Sorted: 27, 26, 25, 24, 23, 22...
    // Keep 3: 27, 26, 25 (plus active log)
    // 22 should be gone.

    println!("Files after periodic cleanup: {:?}", files_after);
    let old_file_2 = format!("{}.{}", file_name, "2023-10-22");
    // Depending on exactly how many were kept in round 1, let's just check the oldest of the NEW set isn't gone,
    // but the oldest of the OLD set is definitely gone.

    // Actually, checking count is safer if we account for active log.
    // But specific file check is more robust against "active log" noise if we pick a really old one.

    assert!(
        !files_after.contains(&old_file_2),
        "Scenario 2: Older file {} should be deleted by periodic cleanup",
        old_file_2
    );
}
