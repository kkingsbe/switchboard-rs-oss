# GPU Passthrough Toggle Implementation Plan

## 1. Overview & Motivation

### Why This Feature Is Needed

Currently, GPU passthrough is **always enabled** for all containers. This is problematic for:

1. **Agents that don't need GPU resources** - Running CPU-only workloads with GPU passthrough wastes resources and may incur unnecessary licensing costs
2. **Cost optimization** - Cloud GPU instances are expensive; agents that don't need GPU acceleration should be able to opt out
3. **Testing and development** - Developers may want to test container behavior without GPU access
4. **Resource isolation** - In shared environments, reserving GPUs for agents that actually need them improves overall system efficiency

### Current Behavior (Always On)

In [`src/docker/run/run.rs`](src/docker/run/run.rs):

- **`build_container_env_vars()`** (lines 166-181) unconditionally prepends NVIDIA environment variables:
  ```rust
  let nvidia_env_vars = vec![
      "NVIDIA_VISIBLE_DEVICES=all".to_string(),
      "NVIDIA_DRIVER_CAPABILITIES=compute,utility,display".to_string(),
      "NVIDIA_DISABLE_REQUIRE=1".to_string(),
  ];
  ```

- **`build_host_config()`** (lines 294-308) unconditionally adds GPU device requests:
  ```rust
  HostConfig {
      // ...
      device_requests: Some(vec![bollard::models::DeviceRequest {
          driver: Some("".to_string()),
          count: Some(-1),
          device_ids: None,
          capabilities: Some(vec![vec!["gpu".to_string()]]),
          options: Some(std::collections::HashMap::new()),
      }]),
      ..Default::default()
  }
  ```

### Desired Behavior (Toggleable)

Add a boolean `gpu` field to `ContainerConfig` that:
- Defaults to `true` for backward compatibility
- When `true`: Current behavior (NVIDIA env vars + GPU device requests)
- When `false`: No NVIDIA env vars, no GPU device requests

---

## 2. Implementation Details

### A. Add `gpu` Field to ContainerConfig Struct

**File:** [`src/docker/run/types.rs`](src/docker/run/types.rs)

**Location:** Lines 69-94 (ContainerConfig struct definition)

**Changes:**

1. Add the `gpu` field to the struct:
```rust
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Name of the agent
    pub agent_name: String,
    /// Environment variables in KEY=value format
    pub env_vars: Vec<String>,
    /// Optional timeout duration (e.g., "30s", "5m")
    pub timeout: Option<String>,
    /// Whether to mount the workspace as read-only
    pub readonly: bool,
    /// Prompt for the AI agent
    pub prompt: String,
    /// Optional list of skill identifiers for the agent
    pub skills: Option<Vec<String>>,
    /// Optional silent timeout duration (e.g., "30s", "5m", "1h", "0" to disable)
    pub silent_timeout: Option<String>,
    /// Whether to enable GPU passthrough to the container
    ///
    /// When `true` (default), NVIDIA GPU environment variables and device
    /// requests are added to the container configuration, enabling GPU access.
    /// When `false`, the container runs without GPU access.
    ///
    /// Default: `true`
    #[serde(default = "default_gpu")]
    pub gpu: bool,
}
```

2. Add the default function and update `ContainerConfig::new()`:
```rust
/// Default value for gpu field - true for backward compatibility
fn default_gpu() -> bool {
    true
}

impl ContainerConfig {
    pub fn new(agent_name: String) -> Self {
        ContainerConfig {
            agent_name,
            env_vars: Vec::new(),
            timeout: None,
            readonly: false,
            prompt: String::new(),
            skills: None,
            silent_timeout: None,
            gpu: true,  // Default to GPU enabled
        }
    }
}
```

### B. Modify `build_container_env_vars()` in src/docker/run/run.rs

**File:** [`src/docker/run/run.rs`](src/docker/run/run.rs)

**Location:** Lines 166-181

**Current Code:**
```rust
pub fn build_container_env_vars(env_vars: &[String]) -> Vec<String> {
    // NVIDIA GPU environment variables for GPU passthrough
    let nvidia_env_vars = vec![
        "NVIDIA_VISIBLE_DEVICES=all".to_string(),
        "NVIDIA_DRIVER_CAPABILITIES=compute,utility,display".to_string(),
        "NVIDIA_DISABLE_REQUIRE=1".to_string(),
    ];

    // Combine NVIDIA env vars with user-provided env vars
    // NVIDIA vars are prepended so user can override if needed
    let mut result = nvidia_env_vars;
    result.extend(env_vars.to_vec());
    result
}
```

**Modified Code:**
```rust
/// Build container environment variables from custom environment variables in the config.
///
/// This function prepares environment variables for the Docker container by converting
/// the configuration's custom environment variables into the format expected by Docker.
/// Each environment variable is a "KEY=value" string that will be passed to the container.
///
/// # GPU Passthrough
///
/// When `gpu` is `true` (default), NVIDIA GPU environment variables are prepended:
/// - `NVIDIA_VISIBLE_DEVICES=all`
/// - `NVIDIA_DRIVER_CAPABILITIES=compute,utility,display`
/// - `NVIDIA_DISABLE_REQUIRE=1`
///
/// When `gpu` is `false`, no GPU environment variables are added.
///
/// # Arguments
///
/// * `env_vars` - Custom environment variables from the config
/// * `gpu` - Whether to enable GPU passthrough (adds NVIDIA env vars if true)
///
/// # Returns
///
/// A vector of "KEY=value" strings suitable for Docker container configuration
pub fn build_container_env_vars(env_vars: &[String], gpu: bool) -> Vec<String> {
    let mut result = Vec::new();

    // Add NVIDIA GPU environment variables if GPU is enabled
    if gpu {
        result.push("NVIDIA_VISIBLE_DEVICES=all".to_string());
        result.push("NVIDIA_DRIVER_CAPABILITIES=compute,utility,display".to_string());
        result.push("NVIDIA_DISABLE_REQUIRE=1".to_string());
    }

    // Combine with user-provided env vars
    // User vars come after GPU vars so they can override if needed
    result.extend(env_vars.to_vec());
    result
}
```

**Important:** Update all call sites of `build_container_env_vars()` to pass the `gpu` parameter. Based on code analysis, the primary call site is in `run_agent()` at line 862:
```rust
let container_env_vars = build_container_env_vars(&config.env_vars, config.gpu);
```

### C. Modify `build_host_config()` in src/docker/run/run.rs

**File:** [`src/docker/run/run.rs`](src/docker/run/run.rs)

**Location:** Lines 242-309

**Current Code (excerpt):**
```rust
pub fn build_host_config(workspace: &str, readonly: bool) -> HostConfig {
    // ... binds setup ...

    HostConfig {
        auto_remove: Some(true),
        binds: Some(binds),
        // GPU device requests for NVIDIA Container Toolkit
        device_requests: Some(vec![bollard::models::DeviceRequest {
            driver: Some("".to_string()),
            count: Some(-1),
            device_ids: None,
            capabilities: Some(vec![vec!["gpu".to_string()]]),
            options: Some(std::collections::HashMap::new()),
        }]),
        ..Default::default()
    }
}
```

**Modified Code:**
```rust
/// Build the HostConfig for a Docker container with workspace mount.
///
/// This function creates the host configuration for a Docker container, primarily
/// setting up the workspace bind mount and optionally configuring GPU passthrough.
///
/// # GPU Passthrough
///
/// When `gpu` is `true` (default), GPU device requests are added to reserve
/// and expose NVIDIA GPUs to the container. This is equivalent to running:
/// `docker run --gpus all`
///
/// When `gpu` is `false`, no GPU device requests are added.
///
/// # Arguments
///
/// * `workspace` - Workspace path on the host system to mount into /workspace
/// * `readonly` - Whether to mount the workspace as read-only
/// * `gpu` - Whether to enable GPU passthrough (adds device requests if true)
///
/// # Returns
///
/// A `HostConfig` with workspace mount configured and optionally GPU support
pub fn build_host_config(workspace: &str, readonly: bool, gpu: bool) -> HostConfig {
    // ... binds setup ...

    // Build device requests for GPU passthrough
    let device_requests = if gpu {
        Some(vec![bollard::models::DeviceRequest {
            driver: Some("".to_string()),
            count: Some(-1),
            device_ids: None,
            capabilities: Some(vec![vec!["gpu".to_string()]]),
            options: Some(std::collections::HashMap::new()),
        }])
    } else {
        None
    };

    HostConfig {
        auto_remove: Some(true),
        binds: Some(binds),
        device_requests,
        ..Default::default()
    }
}
```

**Important:** Update all call sites of `build_host_config()`. Based on code analysis, it is called in `build_container_config()` at line 398:
```rust
pub fn build_container_config(
    image: &str,
    env_vars: Vec<String>,
    readonly: bool,
    workspace: &str,
    agent_name: &str,
    _timeout: u64,
    cmd: Option<&[String]>,
) -> Config<String> {
    // Need to pass gpu through this function too
    let host_config = build_host_config(workspace, readonly, true); // Will be updated
```

**Note:** `build_container_config()` will also need to accept a `gpu` parameter to pass through.

---

## 3. Configuration Schema

### TOML Configuration (switchboard.toml)

The `gpu` field will be available at the **container level** for each agent:

```toml
[[agents]]
name = "gpu-agent"
prompt_file = "prompts/gpu-task.md"
schedule = "0 */6 * * *"
[agents.container]
gpu = true  # Enable GPU (default, can be omitted)

[[agents]]
name = "cpu-agent"
prompt_file = "prompts/cpu-task.md"
schedule = "0 */6 * * *"
[agents.container]
gpu = false  # Disable GPU for this agent
```

### Configuration Flow

1. **TOML Config** (`switchboard.toml`) → parsed by `src/config/mod.rs` into `Agent` struct
2. **Agent Config** → converted to `ContainerConfig` when creating containers
3. **ContainerConfig** → passed to `run_agent()` which calls `build_container_env_vars()` and `build_host_config()`

**Note:** Currently, the `gpu` field does not exist in the `Agent` struct in `src/config/mod.rs`. The implementation may require:

- Option A: Add `gpu` field directly to `Agent` struct (simpler)
- Option B: Add `container` field with nested structure (more complex, follows `settings.container` pattern if it exists)

Based on the current `Agent` struct structure (lines 982-1017 in `src/config/mod.rs`), **Option A** is recommended - add `gpu: Option<bool>` directly to `Agent`.

### Example Agent Configuration with GPU Disabled

```toml
[[agents]]
name = "documentation-builder"
prompt_file = "prompts/docs.md"
schedule = "0 9 * * *"
gpu = false  # Disable GPU - this agent only processes markdown

[[agents]]
name = "code-analyzer"
prompt_file = "prompts/analyze.md"
schedule = "0 10 * * *"
gpu = true  # Enable GPU - this agent needs GPU for LLM inference
```

---

## 4. Backward Compatibility

### Default Behavior

- **Default value for `gpu`: `true`** (backward compatible)
- Existing configurations without `gpu` specified will continue to work exactly as before
- No changes required to existing `switchboard.toml` files

### Migration Path

1. **No immediate action required** - existing configs work unchanged
2. **Optional optimization** - users can set `gpu = false` for CPU-only agents to:
   - Reduce container startup time (no GPU initialization)
   - Avoid consuming GPU resources unnecessarily
   - Improve security posture (fewer device access)

---

## 5. Testing Strategy

### Unit Tests for ContainerConfig Parsing

**File:** [`src/docker/run/run.rs`](src/docker/run/run.rs) (tests module)

Add tests for `build_container_env_vars()` and `build_host_config()`:

```rust
#[test]
fn test_build_container_env_vars_with_gpu_enabled() {
    let env_vars = vec!["CUSTOM_VAR=value".to_string()];
    let result = build_container_env_vars(&env_vars, true);
    
    assert!(result.contains(&"NVIDIA_VISIBLE_DEVICES=all".to_string()));
    assert!(result.contains(&"NVIDIA_DRIVER_CAPABILITIES=compute,utility,display".to_string()));
    assert!(result.contains(&"NVIDIA_DISABLE_REQUIRE=1".to_string()));
    assert!(result.contains(&"CUSTOM_VAR=value".to_string()));
}

#[test]
fn test_build_container_env_vars_with_gpu_disabled() {
    let env_vars = vec!["CUSTOM_VAR=value".to_string()];
    let result = build_container_env_vars(&env_vars, false);
    
    assert!(!result.iter().any(|v| v.starts_with("NVIDIA_")));
    assert!(result.contains(&"CUSTOM_VAR=value".to_string()));
}

#[test]
fn test_build_host_config_with_gpu_enabled() {
    let config = build_host_config("/workspace", false, true);
    assert!(config.device_requests.is_some());
    let device_requests = config.device_requests.unwrap();
    assert!(!device_requests.is_empty());
}

#[test]
fn test_build_host_config_with_gpu_disabled() {
    let config = build_host_config("/workspace", false, false);
    assert!(config.device_requests.is_none());
}
```

### Integration Test with GPU = false

Test that when `gpu = false`:
- No `NVIDIA_*` environment variables are present
- No GPU device requests are added to the container config
- Container still starts and runs successfully (without GPU access)

### Integration Test with GPU = true (Default)

Test that when `gpu = true` or not specified:
- All three NVIDIA environment variables are present
- GPU device requests are configured
- Existing behavior is preserved

---

## 6. Files to Modify

### Primary File: src/docker/run/run.rs

| Function | Lines | Change |
|----------|-------|--------|
| `build_container_env_vars()` | 166-181 | Add `gpu: bool` parameter, conditional NVIDIA vars |
| `build_host_config()` | 242-309 | Add `gpu: bool` parameter, conditional device_requests |
| `build_container_config()` | 388-418 | Add `gpu` parameter, pass through to `build_host_config()` |
| `run_agent()` | 843+ | Pass `config.gpu` through call chain |

### Secondary File: src/docker/run/types.rs

| Struct/Impl | Lines | Change |
|-------------|-------|--------|
| `ContainerConfig` struct | 69-94 | Add `gpu: bool` field with `#[serde(default = "default_gpu")]` |
| `default_gpu()` | (new) | Default function returning `true` |
| `ContainerConfig::new()` | 145-155 | Initialize `gpu: true` |

### Optional (Config Level): src/config/mod.rs

If exposing at agent config level:

| Struct | Lines | Change |
|--------|-------|--------|
| `Agent` struct | 982-1017 | Add `gpu: Option<bool>` field |
| `Agent::effective_gpu()` | (new) | Resolution method if needed |

---

## 7. Implementation Order/Steps

### Step 1: Add `gpu` Field to ContainerConfig

**File:** `src/docker/run/types.rs`

- Add `gpu: bool` field to `ContainerConfig` struct with serde default
- Add `default_gpu()` function
- Update `ContainerConfig::new()` to set `gpu: true`

### Step 2: Update build_container_env_vars()

**File:** `src/docker/run/run.rs`

- Add `gpu: bool` parameter
- Wrap NVIDIA env vars in `if gpu { ... }` block
- Update docstring to document GPU behavior

### Step 3: Update build_host_config()

**File:** `src/docker/run/run.rs`

- Add `gpu: bool` parameter
- Create `device_requests` conditionally based on `gpu`
- Update docstring to document GPU behavior

### Step 4: Update build_container_config()

**File:** `src/docker/run/run.rs`

- Add `gpu: bool` parameter
- Pass `gpu` to `build_host_config()`

### Step 5: Update run_agent() Call Chain

**File:** `src/docker/run/run.rs`

- Pass `config.gpu` to `build_container_env_vars()` and `build_container_config()`

### Step 6: Add Unit Tests

**File:** `src/docker/run/run.rs` (tests module)

- Add tests for `build_container_env_vars()` with GPU on/off
- Add tests for `build_host_config()` with GPU on/off

### Step 7: Update switchboard.sample.toml Documentation

**File:** `switchboard.sample.toml`

- Add documentation for the `gpu` field
- Add example configurations for GPU-enabled and CPU-only agents

---

## 8. Verification Checklist

After implementation, verify:

- [ ] **Code compiles** - `cargo build` succeeds
- [ ] **Existing tests pass** - `cargo test` passes all existing tests
- [ ] **New tests pass** - `cargo test` passes new GPU toggle tests
- [ ] **GPU enabled by default** - When `gpu` field is omitted, NVIDIA vars are added
- [ ] **GPU can be disabled** - When `gpu = false`, no NVIDIA vars or GPU device requests
- [ ] **Backward compatible** - Existing switchboard.toml files work without modification

---

## 9. Example Implementation Diff

```diff
diff --git a/src/docker/run/types.rs b/src/docker/run/types.rs
--- a/src/docker/run/types.rs
+++ b/src/docker/run/types.rs
@@ -91,6 +91,13 @@ pub struct ContainerConfig {
     /// stuck or unresponsive agents.
     pub silent_timeout: Option<String>,
+
+    /// Whether to enable GPU passthrough to the container
+    ///
+    /// When `true` (default), NVIDIA GPU environment variables and device
+    /// requests are added to the container configuration.
+    #[serde(default = "default_gpu")]
+    pub gpu: bool,
 }

+fn default_gpu() -> bool {
+    true
+}

 impl ContainerConfig {
     pub fn new(agent_name: String) -> Self {
         ContainerConfig {
@@ -151,6 +158,7 @@ impl ContainerConfig {
             prompt: String::new(),
             skills: None,
             silent_timeout: None,
+            gpu: true,
         }
     }
 }

diff --git a/src/docker/run/run.rs b/src/docker/run/run.rs
--- a/src/docker/run/run.rs
+++ b/src/docker/run/run.rs
@@ -163,17 +163,22 @@ fn generate_run_id() -> String {
 /// # Returns
 ///
 /// A vector of "KEY=value" strings suitable for Docker container configuration.
-pub fn build_container_env_vars(env_vars: &[String]) -> Vec<String> {
-    // NVIDIA GPU environment variables for GPU passthrough
-    // These are required when using the nvidia runtime to ensure
-    // GPUs are visible and accessible inside the container
-    let nvidia_env_vars = vec![
-        "NVIDIA_VISIBLE_DEVICES=all".to_string(),
-        "NVIDIA_DRIVER_CAPABILITIES=compute,utility,display".to_string(),
-        "NVIDIA_DISABLE_REQUIRE=1".to_string(),
-    ];
-
-    // Combine NVIDIA env vars with user-provided env vars
-    // NVIDIA vars are prepended so user can override if needed
-    let mut result = nvidia_env_vars;
-    result.extend(env_vars.to_vec());
-    result
+pub fn build_container_env_vars(env_vars: &[String], gpu: bool) -> Vec<String> {
+    let mut result = Vec::new();
+
+    // Add NVIDIA GPU environment variables if GPU is enabled
+    if gpu {
+        result.push("NVIDIA_VISIBLE_DEVICES=all".to_string());
+        result.push("NVIDIA_DRIVER_CAPABILITIES=compute,utility,display".to_string());
+        result.push("NVIDIA_DISABLE_REQUIRE=1".to_string());
+    }
+
+    // Combine with user-provided env vars
+    result.extend(env_vars.to_vec());
+    result
 }
```

---

## 10. Risks and Considerations

### Potential Risks

1. **API Change Complexity** - Adding a required parameter to public functions may break existing call sites. Mitigate by using default parameter pattern or updating all call sites.

2. **Testing Environment** - Tests requiring actual GPU hardware may fail in CI environments without NVIDIAContainer toolkit. Consider mocking or conditional test execution.

3. **Documentation Gap** - Users may not realize GPU is enabled by default, leading to unexpected resource consumption. Mitigate by documenting prominently.

### Alternatives Considered

1. **Global Setting Only** - Add GPU toggle to `[settings]` section only. Rejected because different agents may need different GPU requirements.

2. **Separate ContainerSettings Struct** - Create a new struct for container-level settings. More complex, requires significant refactoring of the config-to-ContainerConfig conversion.

3. **Runtime Detection** - Auto-detect if GPU is needed based on agent type. Rejected because it's implicit and may have edge cases.

---

## 11. References

- Current GPU implementation: [`src/docker/run/run.rs:166-181`](src/docker/run/run.rs:166) (`build_container_env_vars`) and [`src/docker/run/run.rs:294-308`](src/docker/run/run.rs:294) (`build_host_config`)
- Config system: [`src/config/mod.rs`](src/config/mod.rs)
- ContainerConfig struct: [`src/docker/run/types.rs:69-94`](src/docker/run/types.rs:69)
- Serde default pattern example: [`src/config/mod.rs:831`](src/config/mod.rs:831) (`default_observability_enabled`)
