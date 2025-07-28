use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const BOX_CONTENTS_SCHEMA: &str = include_str!("../assets/box-contents-schema.json");

pub fn validate_directory(directory: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the schema once
    let schema: Value = serde_json::from_str(BOX_CONTENTS_SCHEMA)?;
    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|e| format!("Failed to compile schema: {e}"))?;

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
}
