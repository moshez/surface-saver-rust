use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn create_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create subdirectories
    fs::create_dir(temp_dir.path().join("box1")).unwrap();
    fs::create_dir(temp_dir.path().join("box2")).unwrap();
    fs::create_dir(temp_dir.path().join("box3")).unwrap();

    // Valid JSON in box1
    let valid_json = r#"[
        {
            "name": "Item 1",
            "description": "First item",
            "categories": ["category1"]
        },
        {
            "name": "Item 2",
            "description": "Second item"
        }
    ]"#;
    fs::write(temp_dir.path().join("box1/contents.json"), valid_json).unwrap();

    // Invalid JSON in box2 (missing required field)
    let invalid_json = r#"[
        {
            "name": "Item without description"
        }
    ]"#;
    fs::write(temp_dir.path().join("box2/invalid.json"), invalid_json).unwrap();

    // Wrong type in box3
    let wrong_type_json = r#"{
        "name": "Not an array",
        "description": "This should be an array"
    }"#;
    fs::write(
        temp_dir.path().join("box3/wrong_type.json"),
        wrong_type_json,
    )
    .unwrap();

    // Add a non-JSON file that should be ignored
    fs::write(temp_dir.path().join("box1/readme.txt"), "This is not JSON").unwrap();

    // Add a JSON file in the root directory (should be ignored)
    fs::write(temp_dir.path().join("root_level.json"), valid_json).unwrap();

    temp_dir
}

#[test]
fn test_validate_all_valid() {
    let temp_dir = TempDir::new().unwrap();

    // Create only valid files
    fs::create_dir(temp_dir.path().join("box1")).unwrap();
    let valid_json = r#"[{"name": "Test", "description": "Valid item"}]"#;
    fs::write(temp_dir.path().join("box1/valid.json"), valid_json).unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "validate", temp_dir.path().to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(
        output.stdout.is_empty() || String::from_utf8_lossy(&output.stdout).contains("Running")
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("error:"));
}

#[test]
fn test_validate_with_errors() {
    let temp_dir = create_test_structure();

    let output = Command::new("cargo")
        .args(&["run", "--", "validate", temp_dir.path().to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("box2/invalid.json:1:1: error:"));
    assert!(stderr.contains("description"));
    assert!(stderr.contains("box3/wrong_type.json:1:1: error:"));
    assert!(stderr.contains("array"));
}

#[test]
fn test_validate_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "validate", temp_dir.path().to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_validate_nonexistent_directory() {
    let output = Command::new("cargo")
        .args(&["run", "--", "validate", "/nonexistent/directory"])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error:"));
}

#[test]
fn test_validate_ignores_non_json_files() {
    let temp_dir = TempDir::new().unwrap();

    fs::create_dir(temp_dir.path().join("box1")).unwrap();
    fs::write(temp_dir.path().join("box1/data.txt"), "Not JSON").unwrap();
    fs::write(temp_dir.path().join("box1/image.png"), &[0u8; 100]).unwrap();
    fs::write(
        temp_dir.path().join("box1/valid.json"),
        r#"[{"name": "Test", "description": "Valid"}]"#,
    )
    .unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "validate", temp_dir.path().to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_validate_nested_directories_ignored() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested structure
    fs::create_dir_all(temp_dir.path().join("box1/nested/deep")).unwrap();

    // Put invalid JSON in nested directory (should be ignored)
    fs::write(
        temp_dir.path().join("box1/nested/deep/invalid.json"),
        r#"{"invalid": "json"}"#,
    )
    .unwrap();

    // Put valid JSON in box1
    fs::write(
        temp_dir.path().join("box1/valid.json"),
        r#"[{"name": "Test", "description": "Valid"}]"#,
    )
    .unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "validate", temp_dir.path().to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

// Search command tests

fn create_search_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create subdirectories
    fs::create_dir(temp_dir.path().join("electronics")).unwrap();
    fs::create_dir(temp_dir.path().join("stationery")).unwrap();

    // Electronics items
    let electronics_json = r#"[
        {
            "name": "Red LED",
            "description": "Small red light-emitting diode",
            "categories": ["electronics", "components"],
            "notes": "5mm diameter, 2V forward voltage"
        },
        {
            "name": "Arduino Uno",
            "description": "Microcontroller board based on ATmega328P",
            "categories": ["electronics", "boards", "arduino"]
        }
    ]"#;
    fs::write(
        temp_dir.path().join("electronics/items.json"),
        electronics_json,
    )
    .unwrap();

    // Stationery items
    let stationery_json = r#"[
        {
            "name": "Blue Notebook",
            "description": "Spiral-bound notebook with blue cover",
            "categories": ["stationery", "notebooks"],
            "notes": "100 pages, ruled"
        },
        {
            "name": "Red Pen",
            "description": "Ballpoint pen with red ink",
            "categories": ["stationery", "pens"]
        }
    ]"#;
    fs::write(
        temp_dir.path().join("stationery/items.json"),
        stationery_json,
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_search_basic() {
    let temp_dir = create_search_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "red",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 2 items"));
    assert!(stdout.contains("Red LED"));
    assert!(stdout.contains("Red Pen"));
}

#[test]
fn test_search_multiple_keywords() {
    let temp_dir = create_search_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "red",
            "pen",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 1 items"));
    assert!(stdout.contains("Red Pen"));
    assert!(!stdout.contains("Red LED"));
}

#[test]
fn test_search_by_name_field() {
    let temp_dir = create_search_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "--name",
            "notebook",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 1 items"));
    assert!(stdout.contains("Blue Notebook"));
}

#[test]
fn test_search_by_categories() {
    let temp_dir = create_search_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "--categories",
            "electronics",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 2 items"));
    assert!(stdout.contains("Red LED"));
    assert!(stdout.contains("Arduino Uno"));
}

#[test]
fn test_search_no_results() {
    let temp_dir = create_search_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "nonexistent",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No items found"));
}

#[test]
fn test_search_case_insensitive() {
    let temp_dir = create_search_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "ARDUINO",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 1 items"));
    assert!(stdout.contains("Arduino Uno"));
}

#[test]
fn test_search_combined_field_flags() {
    let temp_dir = create_search_test_structure();

    // Search for "red" in name field and "electronics" in all fields
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "--name",
            "red",
            "--all",
            "electronics",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 1 items"));
    assert!(stdout.contains("Red LED"));
}

#[test]
fn test_search_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "search",
            temp_dir.path().to_str().unwrap(),
            "anything",
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No items found"));
}

#[test]
fn test_search_nonexistent_directory() {
    let output = Command::new("cargo")
        .args(&["run", "--", "search", "/nonexistent/directory", "test"])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error searching:"));
}

// Consolidate command tests

fn create_consolidate_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple directories with JSON files
    for dir_name in ["dir1", "dir2", "dir3"] {
        fs::create_dir(temp_dir.path().join(dir_name)).unwrap();

        // Create multiple JSON files in each directory
        let items1 = r#"[
            {
                "name": "Item A",
                "description": "First item"
            }
        ]"#;

        let items2 = r#"[
            {
                "name": "Item B",
                "description": "Second item",
                "categories": ["test"]
            }
        ]"#;

        fs::write(temp_dir.path().join(dir_name).join("file1.json"), items1).unwrap();

        fs::write(temp_dir.path().join(dir_name).join("file2.json"), items2).unwrap();
    }

    temp_dir
}

#[test]
fn test_consolidate_basic() {
    let temp_dir = create_consolidate_test_structure();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "consolidate",
            temp_dir.path().to_str().unwrap(),
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Consolidation complete:"));
    assert!(stdout.contains("Successful directories: 3"));
    assert!(stdout.contains("Failed directories: 0"));
    assert!(stdout.contains("Total items consolidated: 6"));

    // Verify all.json files were created
    for dir_name in ["dir1", "dir2", "dir3"] {
        let all_json_path = temp_dir.path().join(dir_name).join("all.json");
        assert!(all_json_path.exists());

        // Verify content
        let content = fs::read_to_string(&all_json_path).unwrap();
        let items: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(items.len(), 2);

        // Verify items are sorted
        assert_eq!(items[0]["name"], "Item A");
        assert_eq!(items[1]["name"], "Item B");
    }
}

#[test]
fn test_consolidate_with_existing_all_json() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    // Create regular JSON file
    let items = r#"[{"name": "New Item", "description": "New"}]"#;
    fs::write(sub_dir.join("data.json"), items).unwrap();

    // Create existing all.json (should be overwritten)
    let old_all = r#"[{"name": "Old Item", "description": "Old"}]"#;
    fs::write(sub_dir.join("all.json"), old_all).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "consolidate",
            temp_dir.path().to_str().unwrap(),
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify all.json was overwritten with new content
    let all_json_content = fs::read_to_string(sub_dir.join("all.json")).unwrap();
    assert!(all_json_content.contains("New Item"));
    assert!(!all_json_content.contains("Old Item"));
}

#[test]
fn test_consolidate_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "consolidate",
            temp_dir.path().to_str().unwrap(),
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successful directories: 0"));
    assert!(stdout.contains("Failed directories: 0"));
}

#[test]
fn test_consolidate_with_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    // Create invalid JSON file
    fs::write(sub_dir.join("invalid.json"), "not valid json").unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "consolidate",
            temp_dir.path().to_str().unwrap(),
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Failed directories: 1"));
    assert!(stdout.contains("Errors:"));
}

#[test]
fn test_consolidate_nonexistent_directory() {
    let output = Command::new("cargo")
        .args(&["run", "--", "consolidate", "/nonexistent/directory"])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error:"));
}
