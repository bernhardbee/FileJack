# Tools Documentation for README

This is the new comprehensive tools section to add to README.md

## All 14 Available Tools

### Core File Operations

#### 1. read_file
Read contents from a file.
- **Parameters:** `path` (string, required)

#### 2. write_file  
Write/replace contents of a file (creates parent directories).
- **Parameters:** `path` (string), `content` (string)

#### 3. append_file ⭐ NEW
Append content to a file (creates if not exists).
- **Parameters:** `path` (string), `content` (string)
- **Use case:** Log files, incremental data collection

#### 4. read_lines ⭐ NEW
Read specific lines from a file (supports line ranges and tail mode).
- **Parameters:** `path` (string), `start_line` (number, optional), `end_line` (number, optional), `tail` (number, optional)
- **Use case:** Large log files, selective reading
- **Example:** `{"path": "/var/log/app.log", "tail": 50}` - Read last 50 lines

### Directory Operations

#### 5. list_directory
List directory contents (recursive option available).
- **Parameters:** `path` (string), `recursive` (boolean, optional)

#### 6. create_directory ⭐ NEW
Create a directory (recursive option for parent directories).
- **Parameters:** `path` (string), `recursive` (boolean, optional)

#### 7. remove_directory ⭐ NEW
Remove a directory (recursive option for non-empty directories).
- **Parameters:** `path` (string), `recursive` (boolean, optional)
- **⚠️ Warning:** `recursive: true` deletes all contents

### File Management

#### 8. delete_file
Delete a file.
- **Parameters:** `path` (string)

#### 9. move_file
Move or rename a file.
- **Parameters:** `from` (string), `to` (string)

#### 10. copy_file
Copy a file.
- **Parameters:** `from` (string), `to` (string)

### File Information

#### 11. get_metadata
Get file/directory metadata (size, timestamps, permissions).
- **Parameters:** `path` (string)
- **Returns:** size, is_file, is_dir, is_symlink, modified, created, readonly

#### 12. file_exists ⭐ NEW
Check if a file or directory exists.
- **Parameters:** `path` (string)
- **Returns:** `true` or `false`
- **Use case:** Conditional operations, pre-flight checks

### Search & Discovery

#### 13. search_files ⭐ NEW
Search for files matching a glob pattern.
- **Parameters:** `path` (string), `pattern` (string), `recursive` (boolean, optional), `max_results` (number, optional)
- **Example:** `{"path": "/var/log", "pattern": "*.log", "recursive": true}`
- **Patterns:** `*.txt`, `test_*.rs`, `**/*.json`

#### 14. grep_file ⭐ NEW
Search file contents using regular expressions.
- **Parameters:** `path` (string), `pattern` (string), `max_matches` (number, optional), `context_lines` (number, optional)
- **Returns:** Array of matches with line numbers and optional context
- **Example:** `{"path": "/var/log/app.log", "pattern": "ERROR|FATAL", "context_lines": 2}`

## Summary

**7 new tools added in v0.2.0:**
- `append_file` - Append to files (essential for logging)
- `file_exists` - Check file existence
- `create_directory` - Create directories explicitly
- `remove_directory` - Remove directories
- `read_lines` - Read specific lines or tail files
- `search_files` - Find files by glob pattern
- `grep_file` - Search file contents with regex

**Total:** 14 comprehensive file operation tools
**Test Coverage:** 107 tests (100% passing)
