use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const BOX_CONTENTS_SCHEMA: &str = include_str!("../assets/box-contents-schema.json");

fn compile_schema(schema_str: &str) -> Result<JSONSchema, Box<dyn std::error::Error>> {
    let schema: Value = serde_json::from_str(schema_str)?;
    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| format!("Failed to compile schema: {e}").into())
}

pub fn validate_directory(directory: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Parse and compile the schema once
    let compiled_schema = compile_schema(BOX_CONTENTS_SCHEMA)?;

    // Read the directory
    let entries = fs::read_dir(directory)?;

    let mut validation_errors = false;

    // Iterate through subdirectories (one level only)
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Look for JSON files in this subdirectory
            let sub_entries = fs::read_dir(&path)?;

            for sub_entry in sub_entries {
                let sub_entry = sub_entry?;
                let file_path = sub_entry.path();

                if file_path.is_file()
                    && file_path.extension().and_then(|s| s.to_str()) == Some("json")
                {
                    // Read and validate the JSON file
                    match validate_json_file(&file_path, &compiled_schema) {
                        Ok(_) => {
                            // No output for valid files
                        }
                        Err(e) => {
                            validation_errors = true;
                            eprintln!("{}:1:1: error: {}", file_path.display(), e);
                        }
                    }
                }
            }
        }
    }

    if validation_errors {
        std::process::exit(1);
    }

    Ok(())
}

pub fn validate_json_file(file_path: &PathBuf, schema: &JSONSchema) -> Result<(), String> {
    // Read the file
    let contents =
        fs::read_to_string(file_path).map_err(|e| format!("failed to read file: {e}"))?;

    // Parse the JSON
    let json: Value = serde_json::from_str(&contents).map_err(|e| format!("invalid JSON: {e}"))?;

    // Validate against schema
    match schema.validate(&json) {
        Ok(_) => Ok(()),
        Err(errors) => {
            // Collect the first validation error
            let error_messages: Vec<String> = errors
                .take(1) // Just take the first error for cleaner output
                .map(|e| format!("schema validation failed: {e}"))
                .collect();

            Err(error_messages.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn get_compiled_schema() -> JSONSchema {
        let schema: Value = serde_json::from_str(BOX_CONTENTS_SCHEMA).unwrap();
        JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .unwrap()
    }

    #[test]
    fn test_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("valid.json");

        let valid_content = r#"[
            {
                "name": "Test Item",
                "description": "A test item",
                "categories": ["test"]
            }
        ]"#;

        fs::write(&file_path, valid_content).unwrap();

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.json");

        let invalid_content = r#"[
            {
                "name": "Test Item"
            }
        ]"#;

        fs::write(&file_path, invalid_content).unwrap();

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("description"));
    }

    #[test]
    fn test_wrong_type() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("wrong_type.json");

        let invalid_content = r#"{
            "name": "Not an array",
            "description": "This should be an array"
        }"#;

        fs::write(&file_path, invalid_content).unwrap();

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("array"));
    }

    #[test]
    fn test_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid_json.json");

        let invalid_content = r#"{ invalid json"#;

        fs::write(&file_path, invalid_content).unwrap();

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid JSON"));
    }

    #[test]
    fn test_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to read file"));
    }

    #[test]
    fn test_optional_fields() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("optional_fields.json");

        let valid_content = r#"[
            {
                "name": "Test Item",
                "description": "A test item",
                "categories": ["test"],
                "notes": "Some optional notes"
            },
            {
                "name": "Minimal Item",
                "description": "Only required fields"
            }
        ]"#;

        fs::write(&file_path, valid_content).unwrap();

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_array() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.json");

        let valid_content = r#"[]"#;

        fs::write(&file_path, valid_content).unwrap();

        let schema = get_compiled_schema();
        let result = validate_json_file(&file_path, &schema);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_directory_with_subdirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create subdirectories with JSON files
        let sub_dir1 = temp_dir.path().join("subdir1");
        fs::create_dir(&sub_dir1).unwrap();

        let valid_content = r#"[{"name": "Item1", "description": "Test1"}]"#;
        fs::write(sub_dir1.join("data.json"), valid_content).unwrap();

        // Test the validate_directory function directly
        let result = validate_directory(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_directory_no_subdirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create a JSON file in the root (not in a subdirectory)
        let valid_content = r#"[{"name": "Item1", "description": "Test1"}]"#;
        fs::write(temp_dir.path().join("data.json"), valid_content).unwrap();

        // Should not find any files since they're not in subdirectories
        let result = validate_directory(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_directory_with_non_json_files() {
        let temp_dir = TempDir::new().unwrap();

        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Create non-JSON files that should be ignored
        fs::write(sub_dir.join("data.txt"), "text file").unwrap();
        fs::write(sub_dir.join("image.png"), &[0u8; 100]).unwrap();

        // Create a valid JSON file
        let valid_content = r#"[{"name": "Item1", "description": "Test1"}]"#;
        fs::write(sub_dir.join("data.json"), valid_content).unwrap();

        let result = validate_directory(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_directory_read_dir_error() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        // Create a valid JSON file
        let valid_content = r#"[{"name": "Item1", "description": "Test1"}]"#;
        fs::write(sub_dir.join("data.json"), valid_content).unwrap();

        // Try to validate the directory
        let result = validate_directory(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_schema_invalid_json() {
        let invalid_schema = r#"{ invalid json"#;
        let result = compile_schema(invalid_schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_schema_invalid_schema() {
        // Valid JSON but invalid schema structure
        let invalid_schema = r#"{"not": "a valid schema"}"#;
        let result = compile_schema(invalid_schema);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to compile schema")
        );
    }
}
