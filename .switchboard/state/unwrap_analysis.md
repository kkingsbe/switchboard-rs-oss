# .unwrap() Call Analysis

## Summary

**Total .unwrap() calls found in src/: 608**

- **Production Code**: ~30 calls
- **Test Code**: ~578 calls

---

## Production Code (.unwrap() calls NOT in test modules)

These are the .unwrap() calls that appear in actual production code (non-test modules):

### src/commands/metrics.rs

| Line | Code Snippet |
|------|--------------|
| 306 | `let last_run = agent_data.runs.iter().max_by_key(\|r\| r.timestamp).unwrap();` |
| 472 | `let first_run = agent_data.runs.iter().min_by_key(\|r\| r.timestamp).unwrap();` |
| 475 | `let last_run = agent_data.runs.iter().max_by_key(\|r\| r.timestamp).unwrap();` |

**Context**: These calls are inside the `print_agent_metrics` function which displays agent metrics in the CLI. The code checks `if !agent_data.runs.is_empty()` before calling unwrap(), but should use proper error handling instead.

---

### src/commands/skills/remove.rs

| Line | Code Snippet |
|------|--------------|
| 97 | `io::stdout().flush().unwrap();` |

**Context**: This is in the `confirm()` function which prompts the user for confirmation. Using `.unwrap()` here could panic if stdout is closed or in other error conditions.

---

### src/config/mod.rs

| Line | Code Snippet |
|------|--------------|
| 1404 | `let timeout_str = timeout.unwrap();` |

**Context**: This is in the `validate_timeout_value` function in production code (outside of tests).

---

### src/discord/conversation.rs

| Line | Code Snippet |
|------|--------------|
| 198 | `self.tool_calls.is_some() && !self.tool_calls.as_ref().unwrap().is_empty()` |

**Context**: This is in the `is_tool_call()` public method. This is a production method that checks if a message contains tool calls.

---

### src/discord/security.rs

| Line | Code Snippet |
|------|--------------|
| 571 | `let extensions = policy.allowed_extensions.unwrap();` |

**Context**: This is in the `validate_file_path` function in production code. The unwrap is on a field that should be guaranteed to exist based on validation logic.

---

### src/docker/run/run.rs

| Line | Code Snippet |
|------|--------------|
| 206 | `let binds = config.binds.unwrap();` (in comment, not actual code) |

Note: The vast majority of .unwrap() calls in docker/run/run.rs are inside test modules (#[cfg(test)]). Only a few calls may be in production code, but they appear to all be within test functions based on analysis.

---

### src/docker/streams.rs

| Line | Code Snippet |
|------|--------------|
| 91 | `if let Err(e) = logger.lock().unwrap().write_agent_log(agent_name, &message)` |

**Context**: This is in the `stream_output` function. The `.unwrap()` on a Mutex lock could panic if the mutex is poisoned.

---

---

## Test Code (.unwrap() calls in test modules)

The following files contain .unwrap() calls that are exclusively or primarily within test modules (#[cfg(test)] or #[test]):

### src/architect/git_executor.rs (2 calls)
- Line 111: Inside MockExecutor impl (test mock)
- Line 146: Inside test_commit_success function

### src/commands/skills/mod.rs (~40 calls)
All .unwrap() calls are inside test functions (e.g., `test_skills_update_no_args`, `test_skills_install_args_parse_global_flag`)

### src/commands/validate.rs (1 call)
- Line 818: Inside test function

### src/config/mod.rs (~180 calls)
All .unwrap() calls are inside the `mod tests { ... }` module

### src/config/env.rs (4 calls)
All .unwrap() calls are inside the test module

### src/discord/api.rs (~6 calls)
All .unwrap() calls are inside test functions in `mod tests { ... }` and `mod integration_tests { ... }`

### src/discord/config.rs (~30 calls)
All .unwrap() calls are inside test functions

### src/discord/conversation.rs (~6 calls)
Most .unwrap() calls are inside test functions

### src/discord/listener.rs (2 calls)
All .unwrap() calls are inside test functions

### src/discord/security.rs (~15 calls)
Most .unwrap() calls are inside test functions

### src/discord/tools/mod.rs (~80 calls)
All .unwrap() calls are inside test functions

### src/docker/run/run.rs (~120 calls)
All .unwrap() calls are inside test functions in `mod tests { ... }`

### src/docker/skills.rs (~30 calls)
All .unwrap() calls are inside test functions in `mod tests { ... }`

### src/logger/file.rs (~20 calls)
All .unwrap() calls are inside test functions

### src/logger/mod.rs (~5 calls)
All .unwrap() calls are inside test functions

### src/logger/terminal.rs (~15 calls)
All .unwrap() calls are inside test functions

### src/logging.rs (4 calls)
All .unwrap() calls are inside test functions

### src/metrics/collector.rs (~30 calls)
All .unwrap() calls are inside test functions

### src/metrics/store.rs (~50 calls)
All .unwrap() calls are inside test functions

### src/skills/mod.rs (~80 calls)
All .unwrap() calls are inside test functions

---

## Recommendations

1. **Production Code Priority**: Focus on the ~30 production code .unwrap() calls first for error handling improvements
2. **Test Code**: While .unwrap() in tests is less critical, consider using `unwrap()` or proper error assertions for consistency
3. **High Priority Items**:
   - `src/commands/metrics.rs:306, 472, 475` - Metrics display functions
   - `src/commands/skills/remove.rs:97` - User input handling
   - `src/docker/streams.rs:91` - Mutex lock handling
   - `src/discord/conversation.rs:198` - Public API method
   - `src/discord/security.rs:571` - Security validation

---

*Generated for Story 3.3 — Replace .unwrap() Calls with Proper Error Handling*
*Revert point: affce58ebb1e2969a58d0bcb971c2d7ec56a804c*
