# Surface Saver

A command-line tool for validating and searching JSON inventory files following the Surface Saver schema.

## Overview

Surface Saver helps manage and search through inventory JSON files that describe items with names, descriptions, categories, and notes. It provides three main functions:

1. **Validate** - Check that JSON files conform to the required schema
2. **Search** - Find items by keywords across multiple fields
3. **Consolidate** - Merge all JSON files in each subdirectory into a single all.json file

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd surface-saver-rust

# Build the project
cargo build --release

# Run tests
cargo test
```

## Usage

### Validate Command

Validate all JSON files in a directory structure:

```bash
surface-saver-rust validate <directory>
```

The validator will:
- Check all JSON files in immediate subdirectories
- Verify required fields (name, description)
- Validate data types
- Report any schema violations with file paths and line numbers

### Search Command

Search for items by keywords:

```bash
surface-saver-rust search <directory> [keywords...]
```

#### Search Options

- `--name` - Search only in the name field
- `--description` - Search only in the description field
- `--categories` - Search only in categories
- `--notes` - Search only in the notes field
- `--all` - Search in all fields (default if no field is specified)

#### Search Examples

```bash
# Search for "notebook" in all fields
surface-saver-rust search ./data notebook

# Search for "red" only in the name field
surface-saver-rust search ./data --name red

# Search for items matching both "red" AND "notebook"
surface-saver-rust search ./data red notebook

# Search for "notebook" in name and "red" in all fields
surface-saver-rust search ./data --name notebook --all red

# Search in specific fields
surface-saver-rust search ./data --categories electronics --description arduino
```

### Consolidate Command

Consolidate all JSON files in each subdirectory into a single `all.json` file:

```bash
surface-saver-rust consolidate <directory>
```

The consolidator will:
- Scan all immediate subdirectories
- For each subdirectory containing JSON files:
  - Read all JSON files (except existing `all.json`)
  - Merge all items into a single array
  - Sort items alphabetically by name
  - Write the result to `all.json` in that subdirectory
- Report success/failure statistics

#### Consolidate Example

```bash
surface-saver-rust consolidate ./data
```

Output:
```
Consolidation complete:
  Successful directories: 3
  Failed directories: 0
  Total items consolidated: 15
```

After running, each subdirectory will have an `all.json` file containing all items from that directory's JSON files, sorted alphabetically.

## JSON Schema

Each JSON file should contain an array of items with the following structure:

```json
[
  {
    "name": "Item Name",
    "description": "Detailed description of the item",
    "categories": ["category1", "category2"],
    "notes": "Additional notes about the item"
  }
]
```

**Required fields:**
- `name` (string): Brief name or title
- `description` (string): Detailed description

**Optional fields:**
- `categories` (array of strings): Categories the item belongs to
- `notes` (string): Additional notes or information

## Directory Structure

The tool expects a directory structure like:

```
data/
├── category1/
│   └── items.json
├── category2/
│   └── items.json
└── category3/
    └── items.json
```

- JSON files should be in immediate subdirectories
- Files in the root directory are ignored
- Files in nested subdirectories (deeper than one level) are ignored
- Non-JSON files are ignored

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with limited threads (if memory constrained)
cargo test -- --test-threads=1

# Run specific test suite
cargo test search::
cargo test integration_test::
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## License

[License information here]