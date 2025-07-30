# Claude Development Notes

This file contains development notes and context for Claude when working on the Surface Saver project.

## Project Overview

Surface Saver is a Rust CLI tool for managing JSON inventory files. It has two main commands:
1. `validate` - Validates JSON files against a schema
2. `search` - Searches for items by keywords with field-specific filtering

## Key Implementation Details

### Search Feature

The search command supports:
- Case-insensitive keyword matching
- Multiple keywords (AND logic - all must match)
- Field-specific searches via flags
- Default behavior searches all fields if no specific field is selected

**Important Search Logic:**
- When using field flags like `--name`, only those specific fields are searched for the associated keywords
- Keywords are processed in order with their associated field flags
- Example: `search --name notebook --all red` searches for "notebook" in name field AND "red" in all fields

### Testing

The project has comprehensive test coverage:
- **Unit tests**: In `src/search.rs` and `src/validator.rs` modules
- **Integration tests**: In `tests/integration_test.rs`

Run tests with: `cd src/surface-saver-rust && cargo test -- --test-threads=1` (use thread limiting if memory constrained)

### Code Style

- No comments unless explicitly requested
- Follow existing patterns in the codebase
- Use existing dependencies (check Cargo.toml first)
- Match the style of neighboring code

### CRITICAL: Task Completion Requirements

**A task is NOT complete until ALL of the following are done:**

1. **Implementation** - The feature/fix is fully implemented
2. **Unit Tests** - Comprehensive unit tests are written and passing
3. **Integration Tests** - Integration tests verify the feature works end-to-end
4. **Lint Clean** - The code must pass ALL of these checks:
   ```bash
   cd src/surface-saver-rust && cargo fmt --check
   cd src/surface-saver-rust && cargo clippy -- -D warnings
   cd src/surface-saver-rust && cargo check
   ```

If any of these requirements are not met, the task should remain marked as "in_progress" in the todo list.

### Linting and Type Checking

Before completing any task, run:
```bash
cd src/surface-saver-rust && cargo fmt
cd src/surface-saver-rust && cargo clippy
cd src/surface-saver-rust && cargo check
```

### CI Coverage Configuration

The CI uses `--lib` flag for tarpaulin to measure unit test coverage only, avoiding potential inconsistencies with integration test coverage measurement.

**Coverage Tracking Best Practices:**
- Avoid multi-line format! macros as they can be counted differently by tarpaulin on different platforms
- Refactor multi-line expressions by extracting them into variables:
  ```rust
  // Instead of:
  output.push(format!(
      "  Total items: {}",
      result.total_items
  ));
  
  // Use:
  let total_items = result.total_items;
  let total_msg = format!("  Total items: {total_items}");
  output.push(total_msg);
  ```
- This ensures each statement is tracked as a distinct line for consistent coverage measurement across environments

### Important: Working Directory

**Always run cargo commands from the project directory:**
```bash
cd src/surface-saver-rust && cargo build
cd src/surface-saver-rust && cargo test
cd src/surface-saver-rust && cargo run -- <args>
```

The project is located at `/opt/kalyke/homedir/src/surface-saver-rust`, not in the root directory.

## Common Tasks

### Adding New Search Fields

1. Update the `SearchOptions` struct in `src/search.rs`
2. Add command-line argument in `Commands::Search` enum in `src/main.rs`
3. Update the `should_search_*` methods in `SearchOptions`
4. Modify `matches_keyword` function to handle the new field
5. Add unit and integration tests

### Modifying JSON Schema

1. Update the schema in `src/validator.rs`
2. Update the `Item` struct in `src/search.rs`
3. Update documentation in README.md
4. Add tests for new fields

## Architecture

```
src/
├── main.rs          # CLI entry point and command handling
├── validator.rs     # JSON validation logic
├── search.rs        # Search functionality
└── lib.rs          # Library exports

tests/
└── integration_test.rs  # Integration tests
```

## Dependencies

- `clap` - Command-line argument parsing
- `serde` / `serde_json` - JSON serialization
- `jsonschema` - JSON schema validation
- `walkdir` - Directory traversal
- `tempfile` - Test utilities

## Performance Considerations

- The search function loads all JSON files into memory
- For large datasets, consider implementing streaming or indexing
- Directory traversal uses `walkdir` for efficiency

## Future Improvements

Potential enhancements to consider:
- Fuzzy search support
- OR logic for keywords
- Regular expression support
- Export search results to various formats
- Performance optimizations for large datasets
- Configuration file support