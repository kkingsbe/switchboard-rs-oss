# Edge Case Handling for Skills

This document describes how the switchboard skills module handles various edge cases and what error messages users will see.

## Overview

The skills module (`src/skills/mod.rs`) handles skill discovery, parsing, and installation. This document covers edge cases in:

- Skill name handling
- Special characters
- Malformed metadata
- Empty/corrupted directories
- Large files

---

## 1. Extremely Long Skill Names

### Description
Skills with names exceeding 200 characters.

### Expected Behavior
The parser accepts very long names but may truncate them for display purposes. The system does not enforce a strict length limit but handles long names gracefully.

### Test Coverage
- [`test_extremely_long_skill_name`](tests/skills_edge_cases.rs) - 250 character name
- [`test_skill_name_200_chars`](tests/skills_edge_cases.rs) - Boundary case at 200 characters

### Error Messages
If the name is rejected:
```
FieldMissing: name field is required
```

---

## 2. Special Characters in Skill Sources

### Description
Skill sources containing special characters like `/`, `\`, `:`, `*`, `?`, quotes, and Unicode characters.

### Expected Behavior
All special characters are properly handled and accepted. The YAML parser handles:
- Forward slashes (`/`) in URLs
- Backslashes (`\`) in Windows paths
- Colons (`:`) in scheme URIs
- Asterisks (`*`) and question marks (`?`) in descriptions
- Double and single quotes
- Unicode characters (CJK, Japanese, etc.)

### Test Coverage
- [`test_special_char_forward_slash`](tests/skills_edge_cases.rs) - `/` in URLs
- [`test_special_char_backslash`](tests/skills_edge_cases.rs) - `\` in paths
- [`test_special_char_colon`](tests/skills_edge_cases.rs) - `:` in schemes
- [`test_special_char_asterisk_question`](tests/skills_edge_cases.rs) - `*` and `?`
- [`test_special_char_quotes`](tests/skills_edge_cases.rs) - `"` and `'`
- [`test_special_char_unicode`](tests/skills_edge_cases.rs) - Unicode characters
- [`test_unicode_full`](tests/skills_edge_cases.rs) - Full Unicode in name/description/source

### Error Messages
No errors expected for valid special characters.

---

## 3. Malformed Skill Metadata

### Description
SKILL.md files with missing or invalid YAML frontmatter.

### Missing SKILL.md File

**Expected Behavior**: Returns an IO error when attempting to read a non-existent file.

**Error Message**:
```
IoError: IO error during read SKILL.md on <path>: No such file or directory
```

**Test Coverage**: [`test_missing_skill_file`](tests/skills_edge_cases.rs)

### Invalid YAML/TOML in Frontmatter

**Expected Behavior**: Returns a `MalformedSkillMetadata` error with details about the YAML parsing failure.

**Error Message**:
```
MalformedSkillMetadata: Invalid YAML: <parsing error details>
```

**Test Coverage**: 
- [`test_invalid_yaml_syntax`](tests/skills_edge_cases.rs) - Invalid YAML syntax
- [`test_toml_instead_of_yaml`](tests/skills_edge_cases.rs) - TOML instead of YAML

### Missing Required Fields

**Expected Behavior**: The `name` field is required. Returns `FieldMissing` error if not present.

**Error Message**:
```
FieldMissing: Required field 'name' is missing
```

**Test Coverage**: 
- [`test_missing_name_field`](tests/skills_edge_cases.rs) - Missing required `name`
- [`test_missing_version_field`](tests/skills_edge_cases.rs) - `version` is optional

### Missing Frontmatter

**Expected Behavior**: Returns `MissingFrontmatter` error when no `---` delimiters are found.

**Error Message**:
```
MissingFrontmatter: Missing YAML frontmatter in SKILL.md file: <path>
```

**Test Coverage**:
- [`test_no_frontmatter`](tests/skills_edge_cases.rs) - No frontmatter at all
- [`test_empty_frontmatter`](tests/skills_edge_cases.rs) - Only delimiters
- [`test_partial_frontmatter_opening`](tests/skills_edge_cases.rs) - Missing closing delimiter
- [`test_whitespace_only_skill_file`](tests/skills_edge_cases.rs) - Only whitespace

---

## 4. Empty Skill Directories

### Description
Skills directories that are empty or contain only subdirectories without SKILL.md files.

### Empty Skills Directory

**Expected Behavior**: The directory scan completes without errors but finds no skills.

**Error Message**: None - empty directories are handled gracefully.

**Test Coverage**: [`test_empty_skills_directory`](tests/skills_edge_cases.rs)

### Directory with Only Subdirectories

**Expected Behavior**: Subdirectories without SKILL.md files are skipped during skill discovery. The `InvalidSkillDirectory` error may be returned when attempting to parse a skill from such a directory.

**Error Message**:
```
InvalidSkillDirectory: No SKILL.md found in directory: <path>
```

**Test Coverage**: [`test_directory_only_subdirs`](tests/skills_edge_cases.rs)

---

## 5. Corrupted/Binary Data

### Description
SKILL.md files containing non-text/binary content or extremely large files.

### Binary Data in SKILL.md

**Expected Behavior**: Returns an IO error due to UTF-8 encoding issues.

**Error Message**:
```
IoError: IO error during read SKILL.md on <path>: <encoding error>
```

**Test Coverage**: [`test_binary_data_in_skill_file`](tests/skills_edge_cases.rs)

### Very Large Files

**Expected Behavior**: Large files (1MB+) are handled successfully. The parser reads the entire file into memory and parses the YAML.

**Error Message**: None - large files are accepted.

**Test Coverage**: [`test_very_large_skill_file`](tests/skills_edge_cases.rs)

### Very Long Field Values

**Expected Behavior**: Extremely long field values (50k+ characters) are accepted.

**Error Message**: None - long values are accepted.

**Test Coverage**: [`test_very_long_field_value`](tests/skills_edge_cases.rs)

---

## 6. Additional Edge Cases

### Nested YAML Structures

**Description**: Complex nested arrays in the YAML frontmatter.

**Expected Behavior**: Parsing may succeed with reduced fidelity for nested structures (arrays may be empty).

**Test Coverage**: [`test_nested_yaml_arrays`](tests/skills_edge_cases.rs)

### Multiple YAML Documents

**Description**: SKILL.md files containing multiple YAML documents separated by `---`.

**Expected Behavior**: Only the first YAML document is parsed.

**Test Coverage**: [`test_multiple_yaml_documents`](tests/skills_edge_cases.rs)

---

## Summary Table

| Edge Case | Behavior | Error Type |
|-----------|----------|------------|
| Long skill names (>200 chars) | Accepted (possibly truncated) | None |
| Special characters | Accepted | None |
| Missing SKILL.md | Returns error | `SkillsError::IoError` |
| Invalid YAML | Returns error | `SkillsError::MalformedSkillMetadata` |
| Missing required `name` field | Returns error | `SkillsError::FieldMissing` |
| Missing frontmatter | Returns error | `SkillsError::MissingFrontmatter` |
| Empty skills directory | Handled gracefully | None |
| Directory without SKILL.md | May return error | `SkillsError::InvalidSkillDirectory` |
| Binary data in SKILL.md | Returns error | `SkillsError::IoError` |
| Very large files (>1MB) | Handled successfully | None |
| Unicode characters | Accepted | None |

---

## Running the Tests

```bash
# Run all edge case tests
cargo test --test skills_edge_cases

# Run a specific test
cargo test --test skills_edge_cases test_extremely_long_skill_name
```

---

## Related Files

- [`src/skills/mod.rs`](src/skills/mod.rs) - SkillsManager and parsing logic
- [`src/skills/error.rs`](src/skills/error.rs) - Error type definitions
- [`tests/skills_edge_cases.rs`](tests/skills_edge_cases.rs) - Edge case test suite
