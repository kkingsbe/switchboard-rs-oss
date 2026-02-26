# Testability Enhancement Plan for switchboard-rs CLI

## 1. Executive Summary

The switchboard-rs CLI codebase currently exhibits significant testability limitations due to direct dependencies on external services and system processes. Approximately 40% of the codebase (modules: `docker/mod`, `docker/run`, `architect`, `skills`, `cli/mod`, `scheduler/mod`) has LOW or NO testability because components directly instantiate concrete types like `bollard::Docker` and `std::process::Command`, making them impossible to test in isolation. The primary challenge is that these modules cannot be unit tested without running Docker daemon, executing external commands, or performing file system operations, which creates slow, fragile, and CI-unfriendly tests.

The goal of this enhancement plan is to achieve full testability across all modules through a systematic refactoring approach that introduces trait-based dependency injection. By extracting external dependencies behind well-defined traits, we can enable mock implementations that allow fast, deterministic unit tests. The plan prioritizes the most impactful changes first: abstracting Docker operations (used in 3 core modules) and process execution (used in 4 modules), which will immediately unlock testability for approximately 70% of the currently untestable code.

## 2. Proposed Mock Trait Architecture

### 2.1 Core Traits

#### DockerClientTrait

The primary abstraction for all Docker operations. This trait encapsulates the full lifecycle of Docker interactions needed by the codebase.

```rust
use async_trait::async_trait;
use bollard::image::BuildImageOptions;
use bytes::Bytes;
use std::path::Path;

/// Trait for Docker client operations
/// 
/// This trait abstracts all Docker interactions including image building,
/// container management, and daemon connectivity checks.
#[async_trait]
pub trait DockerClientTrait: Send + Sync {
    /// Check if Docker daemon is available and responsive
    async fn ping(&self) -> Result<(), DockerError>;
    
    /// Check if an image exists locally
    async fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;
    
    /// Build a Docker image from build context
    async fn build_image(
        &self,
        options: &BuildImageOptions<str>,
        context: impl Into<Bytes> + Send,
    ) -> Result<(), DockerError>;
    
    /// Create and run a container with the given configuration
    async fn run_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<AgentExecutionResult, DockerError>;
    
    /// Stop a running container
    async fn stop_container(
        &self,
        container_id: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<(), DockerError>;
    
    /// Get logs from a container
    async fn container_logs(
        &self,
        container_id: &str,
        follow: bool,
        tail: Option<u64>,
    ) -> Result<impl Stream<Item = Result<bytes::Bytes, DockerError>> + Send, DockerError>;
    
    /// Wait for a container to exit
    async fn wait_container(
        &self,
        container_id: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<ExitStatus, DockerError>;
}

/// Result of a container execution
#[derive(Debug, Clone)]
pub struct AgentExecutionResult {
    pub container_id: String,
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
}
```

#### ProcessExecutorTrait

Abstraction for executing external system commands. This enables testing of modules that call `npx`, `docker`, `git`, and other command-line tools.

```rust
/// Trait for executing external processes
///
/// This trait abstracts system command execution, allowing tests to inject
/// mock executors that return predefined results without running real processes.
#[async_trait]
pub trait ProcessExecutorTrait: Send + Sync {
    /// Execute a command and return its output
    async fn execute(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<ProcessOutput, ProcessError>;
    
    /// Execute a command with custom working directory
    async fn execute_with_env(
        &self,
        program: &str,
        args: &[&str],
        env: &[(impl AsRef<str> + Send)],
        working_dir: Option<&Path>,
    ) -> Result<ProcessOutput, ProcessError>;
}

/// Output from a process execution
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Exit status of a process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Code(i32),
    Signal(u32),
    Unknown,
}

/// Errors from process execution
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Command '{program}' failed: {message}")]
    CommandFailed { program: String, message: String },
    
    #[error("IO error executing '{program}': {source}")]
    IoError { 
        program: String, 
        #[source] 
        source: std::io::Error 
    },
}
```

### 2.2 Optional Traits (Future Enhancements)

#### LoggerProviderTrait

For abstracting logging behavior in tests.

```rust
/// Trait for providing loggers
pub trait LoggerProvider: Send + Sync {
    fn create_logger(&self, name: &str) -> Result<Box<dyn Logger>, LoggerError>;
}
```

#### CronSchedulerTrait

For testing scheduler behavior without time-based delays.

```rust
/// Trait for cron-based scheduling
#[async_trait]
pub trait CronSchedulerTrait: Send + Sync {
    async fn add_job(&mut self, job: Box<dyn Job>) -> Result<Uuid, SchedulerError>;
    async fn remove_job(&mut self, job_id: &Uuid) -> Result<(), SchedulerError>;
    async fn start(&mut self) -> Result<(), SchedulerError>;
    async fn stop(&mut self) -> Result<(), SchedulerError>;
    async fn is_running(&self) -> bool;
}
```

### 2.3 Mock Implementation Pattern Using mockall

The plan recommends using the `mockall` crate for generating mock implementations of traits. This provides a declarative API for configuring mock behavior.

```toml
# Add to dev-dependencies in Cargo.toml
mockall = "0.12"
```

Example mock configuration:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    
    mock! {
        DockerClientMock {}
        
        #[async_trait]
        impl DockerClientTrait for DockerClientMock {
            async fn ping(&self) -> Result<(), DockerError>;
            async fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;
            async fn run_container(&self, config: &ContainerConfig) 
                -> Result<AgentExecutionResult, DockerError>;
            // ... other methods
        }
    }
    
    #[tokio::test]
    async fn test_docker_operations() {
        let mut mock_docker = DockerClientMock::new();
        
        // Configure mock to return successful ping
        mock_docker
            .expect_ping()
            .returning(Ok(()));
        
        // Configure mock to simulate image exists
        mock_docker
            .expect_image_exists()
            .with(eq("test-image"), eq("latest"))
            .returning(Ok(true));
        
        // Test the code with mock
        let result = mock_docker.ping().await;
        assert!(result.is_ok());
    }
}
```

## 3. Phased Implementation Plan

### Phase 1: Foundation (Estimated: 2-3 days)

**Objective**: Set up the infrastructure for trait-based dependency injection.

**Tasks**:
1. Add `mockall = "0.12"` to dev-dependencies in [`Cargo.toml`](Cargo.toml:51-58)
2. Add `async-trait = "0.1"` to dependencies
3. Create new module `src/traits/mod.rs` with trait definitions:
   - `DockerClientTrait` with all Docker operations
   - `ProcessExecutorTrait` with command execution methods
4. Create `src/traits/errors.rs` with error types:
   - `DockerError` (adapt existing from `src/docker/mod.rs`)
   - `ProcessError` (new error type)
   - `AgentExecutionResult`, `ProcessOutput`, `ExitStatus` types
5. Document trait contracts with docstrings and examples

**Success Criteria**:
- Traits compile without errors
- All trait methods have documentation
- Error types are comprehensive and well-structured

---

### Phase 2: Docker Abstraction (Highest Priority - Estimated: 4-5 days)

**Objective**: Abstract all Docker operations behind `DockerClientTrait`.

**Impact**: Unlocks testability for `docker/mod`, `docker/run`, `scheduler/mod`, and `cli/mod`.

**Tasks**:

1. **Refactor `src/docker/mod.rs`**:
   - Create `RealDockerClient` struct implementing `DockerClientTrait`
   - Move existing `DockerClient` implementation to `RealDockerClient`
   - Implement all trait methods by delegating to `bollard::Docker`
   - Keep `DockerClient` as a type alias for backward compatibility:
     ```rust
     pub type DockerClient = RealDockerClient;
     ```

2. **Refactor `src/docker/run.rs`**:
   - Update `run_agent()` to accept `Arc<dyn DockerClientTrait>` instead of `DockerClient`
   - Update `ContainerConfig` to use trait-compatible types
   - Ensure all container operations go through the trait

3. **Refactor `src/scheduler/mod.rs`**:
   - Add `docker_client: Arc<dyn DockerClientTrait>` field to `Scheduler`
   - Update `Scheduler::new()` to accept optional `docker_client` parameter
   - Update `execute_agent()` to use injected `docker_client`
   - Default to `RealDockerClient` if none provided (for production)

4. **Refactor `src/cli/mod.rs`**:
   - Update `run_up()` to accept and pass `docker_client`
   - Update `run_build()` to accept and pass `docker_client`
   - Update `run_down()` to accept and pass `docker_client`
   - Update `run_run()` to accept and pass `docker_client`
   - Create `docker_client` in main CLI entry point and pass down

5. **Create integration test for real Docker operations** (in `tests/integration/docker_integration.rs`):
   - Test `RealDockerClient` against actual Docker daemon
   - Use `#[cfg(all(test, feature = "integration"))]` to gate
   - Verify image building, container creation, and execution

**Success Criteria**:
- All Docker-dependent code uses `DockerClientTrait`
- `RealDockerClient` works identically to original `DockerClient`
- Integration tests pass with real Docker
- Unit tests can be written with mock implementations

---

### Phase 3: Process Abstraction (Architect & Skills) (Estimated: 3-4 days)

**Objective**: Abstract external command execution behind `ProcessExecutorTrait`.

**Impact**: Unlocks testability for `architect`, `skills`, `config/mod`, and `cli/mod`.

**Tasks**:

1. **Create `src/process/executor.rs`**:
   - Implement `RealProcessExecutor` using `std::process::Command`
   - Implement `ProcessExecutorTrait` by delegating to `Command`
   - Handle cross-platform differences (Windows vs Unix)

2. **Refactor `src/architect/state.rs`**:
   - Add `executor: Arc<dyn ProcessExecutorTrait>` parameter to functions
   - Replace `Command::new("git")` calls with `executor.execute("git", ...)`
   - Update `commit_progress()` to use injected executor
   - Add optional executor parameter with default `RealProcessExecutor`

3. **Refactor `src/skills/mod.rs`**:
   - Add `executor: Arc<dyn ProcessExecutorTrait>` to `SkillsManager`
   - Replace npx calls with `executor.execute("npx", ...)`
   - Update constructor to accept optional executor

4. **Refactor `src/config/mod.rs`**:
   - File operations already use `std::fs` which can be mocked with tempfile
   - Minimal changes needed, primarily around validation

5. **Refactor `src/docker/mod.rs`** (docker context command):
   - Replace `Command::new("docker")` calls with executor
   - Update `get_docker_socket_path()` to use `ProcessExecutorTrait`

6. **Create unit tests**:
   - `tests/architect_test.rs`: Test architect operations with mock git commands
   - `tests/skills_test.rs`: Test skills operations with mock npx output
   - Mock specific command behaviors: success, failure, timeout

**Success Criteria**:
- All external command execution uses `ProcessExecutorTrait`
- Architect operations can be tested without git
- Skills operations can be tested without npx
- Cross-platform compatibility maintained

---

### Phase 4: Logger/Metrics Injection (Optional - Estimated: 2-3 days)

**Objective**: Abstract logging and metrics collection for enhanced testability.

**Impact**: Enables testing of logging behavior and metrics recording without side effects.

**Tasks**:

1. **Create `src/logger/provider.rs`**:
   - Define `LoggerProvider` trait
   - Implement `RealLoggerProvider` using existing `Logger`
   - Create `MockLogger` for testing (in-memory buffer)

2. **Create `src/metrics/provider.rs`**:
   - Define `MetricsProvider` trait
   - Implement `RealMetricsProvider` using existing `MetricsStore`
   - Create `MockMetricsProvider` for testing

3. **Refactor affected modules**:
   - Update `Scheduler` to accept optional `LoggerProvider` and `MetricsProvider`
   - Update `cli/mod.rs` to pass providers to scheduler
   - Update `docker/run.rs` to use provided logger

4. **Create tests**:
   - Test that appropriate log messages are generated
   - Test metrics are recorded correctly
   - Test error handling in logging/metrics

**Success Criteria**:
- Logging and metrics can be mocked
- Tests verify log messages without writing files
- Tests verify metrics recording without side effects
- Production behavior unchanged

---

### Phase 5: Test Suite Development (Estimated: 5-7 days)

**Objective**: Write comprehensive unit and integration tests using the new mocking infrastructure.

**Tasks**:

1. **Create unit tests for each module**:
   - `tests/docker_unit.rs`: Docker operations with mocks
   - `tests/scheduler_unit.rs`: Scheduler logic with mocks
   - `tests/architect_unit.rs`: Architect workflow with mocks
   - `tests/skills_unit.rs`: Skills management with mocks
   - `tests/cli_unit.rs`: CLI command handlers with mocks

2. **Create integration tests** (feature-gated):
   - `tests/integration/full_workflow.rs`: End-to-end scheduler test
   - `tests/integration/docker_operations.rs`: Real Docker testing
   - `tests/integration/process_execution.rs`: Real process testing

3. **Achieve coverage targets**:
   - Unit tests: >80% coverage for core logic
   - Integration tests: Cover all happy paths and error scenarios
   - Mock coverage: All traits have mock implementations

4. **Add continuous integration**:
   - Run unit tests on every PR
   - Run integration tests nightly or on merge
   - Add coverage reporting

**Success Criteria**:
- All modules have >80% unit test coverage
- Integration tests pass on real Docker
- CI runs tests automatically
- Coverage reports are generated

---

### Summary of Phases

| Phase | Focus | Estimated Effort | Modules Impacted |
|--------|--------|------------------|-----------------|
| 1 | Foundation | 2-3 days | New module creation |
| 2 | Docker Abstraction | 4-5 days | docker, scheduler, cli |
| 3 | Process Abstraction | 3-4 days | architect, skills, config |
| 4 | Logger/Metrics | 2-3 days | scheduler, cli, docker |
| 5 | Test Suite | 5-7 days | All modules |
| **Total** | **16-22 days** | | |

## 4. Module-by-Module Refactoring Guide

### 4.1 docker/mod.rs (NO Testability → HIGH)

**Current Issues**:
- Directly instantiates `bollard::Docker`
- Uses `Command::new("docker")` for context operations
- Cannot be tested without running Docker daemon

**Changes Needed**:

1. **Add trait dependency to struct**:
```rust
// Before:
pub struct DockerClient {
    docker: Docker,
    _image_name: String,
    _image_tag: String,
}

// After:
pub struct DockerClient {
    docker: Arc<dyn DockerClientTrait>,
    _image_name: String,
    _image_tag: String,
}
```

2. **Update constructor to accept trait**:
```rust
// Before:
impl DockerClient {
    pub async fn new(image_name: String, image_tag: String) -> Result<Self, DockerError> {
        let docker = connect_to_docker()?;
        // ...
    }
}

// After:
impl DockerClient {
    pub async fn new_with_client(
        image_name: String,
        image_tag: String,
        client: Arc<dyn DockerClientTrait>,
    ) -> Result<Self, DockerError> {
        client.ping().await?;
        Ok(DockerClient {
            docker: client,
            _image_name: image_name,
            _image_tag: image_tag,
        })
    }
    
    // Convenience method for production
    pub async fn new(image_name: String, image_tag: String) -> Result<Self, DockerError> {
        let real_client = Arc::new(RealDockerClient::new()?);
        Self::new_with_client(image_name, image_tag, real_client).await
    }
}
```

3. **Replace docker context command**:
```rust
// Before:
fn get_docker_socket_path() -> Option<String> {
    let output = Command::new("docker")
        .args(["context", "show"])
        .output()
        .ok()?;
    // ...
}

// After:
pub async fn get_docker_socket_path(executor: &dyn ProcessExecutorTrait) -> Option<String> {
    let output = executor.execute("docker", &["context", "show"]).await.ok()?;
    // ...
}
```

**Test Strategy**:
- Create `MockDockerClient` using mockall
- Test connection logic with successful and failed pings
- Test context parsing with mock command outputs
- Test error handling with various failure scenarios

**Test Example**:
```rust
#[tokio::test]
async fn test_docker_client_ping_success() {
    let mut mock = MockDockerClientTrait::new();
    mock.expect_ping()
        .returning(Ok(()));
    
    let client = DockerClient::new_with_client(
        "test".to_string(),
        "latest".to_string(),
        Arc::new(mock),
    ).await.unwrap();
    
    assert!(true); // Client created successfully
}
```

---

### 4.2 docker/run.rs (NO Testability → HIGH)

**Current Issues**:
- `run_agent()` creates containers directly
- Uses Docker API calls throughout execution flow

**Changes Needed**:

1. **Update function signature**:
```rust
// Before:
pub async fn run_agent(
    config: ContainerConfig,
    logger: Logger,
) -> Result<AgentExecutionResult, DockerError> {
    let docker = connect_to_docker()?;
    // ...
}

// After:
pub async fn run_agent(
    config: ContainerConfig,
    logger: Logger,
    docker: Arc<dyn DockerClientTrait>,
) -> Result<AgentExecutionResult, DockerError> {
    docker.run_container(&config).await
}
```

2. **Refactor container lifecycle**:
```rust
// Extract container creation logic
async fn create_container(
    docker: &dyn DockerClientTrait,
    config: &ContainerConfig,
) -> Result<String, DockerError> {
    // Container creation logic
}

// Extract log streaming logic
async fn stream_logs(
    docker: &dyn DockerClientTrait,
    container_id: &str,
    logger: &mut Logger,
) -> Result<(), DockerError> {
    // Log streaming logic
}
```

**Test Strategy**:
- Test container creation with mock
- Test log streaming with mock
- Test timeout handling with mock
- Test error propagation from Docker operations

**Test Example**:
```rust
#[tokio::test]
async fn test_run_agent_success() {
    let mut mock_docker = MockDockerClientTrait::new();
    mock_docker.expect_run_container()
        .returning(|_| Ok(AgentExecutionResult {
            container_id: "test-container".to_string(),
            exit_code: 0,
            stdout: "success".to_string(),
            stderr: String::new(),
            duration: Duration::from_secs(1),
        }));
    
    let config = ContainerConfig::test_config();
    let logger = Logger::test_logger();
    let docker = Arc::new(mock_docker);
    
    let result = run_agent(config, logger, docker).await.unwrap();
    assert_eq!(result.exit_code, 0);
}
```

---

### 4.3 scheduler/mod.rs (LOW → HIGH)

**Current Issues**:
- Creates `DockerClient` in `execute_agent()`
- Cannot test scheduler logic without Docker

**Changes Needed**:

1. **Add docker_client field**:
```rust
// Before:
pub struct Scheduler {
    agents: Arc<Mutex<Vec<ScheduledAgent>>>,
    running: AtomicBool,
    scheduler: Option<JobScheduler>,
    // ...
}

// After:
pub struct Scheduler {
    agents: Arc<Mutex<Vec<ScheduledAgent>>>,
    running: AtomicBool,
    scheduler: Option<JobScheduler>,
    docker_client: Arc<dyn DockerClientTrait>,
    // ...
}
```

2. **Update constructor**:
```rust
// Before:
pub async fn new(
    clock: Option<Arc<dyn Clock>>,
    log_dir: Option<PathBuf>,
) -> Result<Self, SchedulerError> {
    // ...
}

// After:
pub async fn new(
    clock: Option<Arc<dyn Clock>>,
    log_dir: Option<PathBuf>,
    docker_client: Option<Arc<dyn DockerClientTrait>>,
) -> Result<Self, SchedulerError> {
    let docker = docker_client.unwrap_or_else(|| {
        Arc::new(RealDockerClient::new("switchboard-agent".to_string(), "latest".to_string())
            .expect("Failed to create Docker client")
    });
    
    Ok(Self {
        docker_client: docker,
        // ...
    })
}
```

3. **Update execute_agent**:
```rust
// Before:
async fn execute_agent(
    agent_name: String,
    agents: Arc<Mutex<Vec<ScheduledAgent>>>,
    // ...
) -> Result<(), SchedulerError> {
    let docker = DockerClient::new(image_name, image_tag).await?;
    let result = run_agent(container_config, logger).await?;
    // ...
}

// After:
async fn execute_agent(
    agent_name: String,
    agents: Arc<Mutex<Vec<ScheduledAgent>>>,
    docker: Arc<dyn DockerClientTrait>,
    // ...
) -> Result<(), SchedulerError> {
    let result = run_agent(container_config, logger, docker).await?;
    // ...
}
```

**Test Strategy**:
- Test overlap detection with mock Docker
- Test agent registration without Docker
- Test scheduler lifecycle with mock
- Test error handling with mock failures

**Test Example**:
```rust
#[tokio::test]
async fn test_scheduler_with_mock_docker() {
    let mut mock_docker = MockDockerClientTrait::new();
    mock_docker.expect_run_container()
        .times(1)
        .returning(|_| Ok(test_execution_result()));
    
    let mut scheduler = Scheduler::new(
        Some(Arc::new(MockClock::new())),
        None,
        Some(Arc::new(mock_docker)),
    ).await.unwrap();
    
    let agent = create_test_agent("test", "* * * * *");
    scheduler.register_agent(&agent, PathBuf::new(), PathBuf::new(), 
        "test-image".to_string(), "latest".to_string(), 
        "/test".to_string()).await.unwrap();
    
    // Scheduler operations now testable
}
```

---

### 4.4 architect/mod.rs & architect/state.rs (NO → HIGH)

**Current Issues**:
- Uses `Command::new("git")` for all git operations
- Cannot test without git repository

**Changes Needed**:

1. **Add executor parameter**:
```rust
// Before:
pub fn commit_progress(message: &str) -> Result<(), ArchitectError> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()?;
    // ...
}

// After:
pub fn commit_progress(
    message: &str, 
    executor: &dyn ProcessExecutorTrait,
) -> Result<(), ArchitectError> {
    let output = executor.execute("git", &["commit", "-m", message]).await?;
    // ...
}
```

2. **Update state operations**:
```rust
// All state functions accept optional executor
pub fn save_state(
    state: &ArchitectState,
    executor: Option<&dyn ProcessExecutorTrait>,
) -> Result<()> {
    if let Some(exec) = executor {
        // Auto-commit with executor
        let _ = commit_progress("chore: save state", exec);
    }
    // Save to file...
}

// Default implementation for production
pub fn save_state_prod(state: &ArchitectState) -> Result<()> {
    save_state(state, Some(&RealProcessExecutor))
}
```

3. **Update session module**:
```rust
pub fn start_session(
    current_sprint: u32,
    executor: Option<&dyn ProcessExecutorTrait>,
) -> Result<SessionHandle> {
    let state = ArchitectState::new(current_sprint);
    save_state(&state, executor)?;
    // ...
}
```

**Test Strategy**:
- Test state persistence without git
- Test commit behavior with mock git
- Test session lifecycle without repository
- Test error handling for git failures

**Test Example**:
```rust
#[test]
fn test_commit_progress_with_mock() {
    let mut mock = MockProcessExecutorTrait::new();
    mock.expect_execute()
        .with(eq("git"), eq(vec!["commit", "-m", "test message"]))
        .returning(Ok(ProcessOutput {
            status: ExitStatus::Code(0),
            stdout: b"commit successful".to_vec(),
            stderr: Vec::new(),
        }));
    
    let result = commit_progress("test message", &mock);
    assert!(result.is_ok());
}
```

---

### 4.5 skills/mod.rs (NO → HIGH)

**Current Issues**:
- Calls `npx skills` directly
- Cannot test without npx and skills CLI

**Changes Needed**:

1. **Add executor to SkillsManager**:
```rust
// Before:
pub struct SkillsManager {
    skills_dir: PathBuf,
    global_skills_dir: PathBuf,
    npx_available: bool,
}

// After:
pub struct SkillsManager {
    skills_dir: PathBuf,
    global_skills_dir: PathBuf,
    npx_available: bool,
    executor: Arc<dyn ProcessExecutorTrait>,
}
```

2. **Update constructor**:
```rust
impl SkillsManager {
    pub fn new() -> Self {
        Self::with_executor(Arc::new(RealProcessExecutor))
    }
    
    pub fn with_executor(executor: Arc<dyn ProcessExecutorTrait>) -> Self {
        Self {
            skills_dir: PathBuf::from(".kilocode/skills"),
            global_skills_dir: PathBuf::from(".kilocode/skills"),
            npx_available: false,
            executor,
        }
    }
}
```

3. **Update skill operations**:
```rust
impl SkillsManager {
    pub async fn list_skills(&self) -> Result<Vec<SkillInfo>, SkillsError> {
        let output = self.executor.execute("npx", &["skills", "list"]).await?;
        parse_skills_output(&output.stdout)
    }
    
    pub async fn install_skill(&self, name: &str) -> Result<(), SkillsError> {
        let output = self.executor.execute("npx", &["skills", "install", name]).await?;
        check_install_success(&output)
    }
}
```

**Test Strategy**:
- Test skill listing with mock npx output
- Test skill installation with mock
- Test error handling for missing npx
- Test skill parsing logic

**Test Example**:
```rust
#[tokio::test]
async fn test_list_skills() {
    let mut mock = MockProcessExecutorTrait::new();
    mock.expect_execute()
        .with(eq("npx"), starts_with("skills"))
        .returning(Ok(ProcessOutput {
            status: ExitStatus::Code(0),
            stdout: b"skill1, skill2, skill3".to_vec(),
            stderr: Vec::new(),
        }));
    
    let manager = SkillsManager::with_executor(Arc::new(mock));
    let skills = manager.list_skills().await.unwrap();
    assert_eq!(skills.len(), 3);
}
```

---

### 4.6 cli/mod.rs (LOW → HIGH)

**Current Issues**:
- Creates DockerClient in multiple functions
- Uses Command::new for process management

**Changes Needed**:

1. **Add docker_client parameter to command handlers**:
```rust
// Before:
pub async fn run_up(args: UpCommand, config_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;
    let scheduler = Scheduler::new(None, None).await?;
    // ...
}

// After:
pub async fn run_up(
    args: UpCommand,
    config_path: Option<String>,
    docker_client: Option<Arc<dyn DockerClientTrait>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;
    let scheduler = Scheduler::new(None, None, docker_client).await?;
    // ...
}
```

2. **Update main run function**:
```rust
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Create Docker client once at top level
    let docker_client = match &cli.command {
        Commands::Up(_) | Commands::Run(_) | Commands::Build(_) | Commands::Down(_) => {
            Some(Arc::new(DockerClient::new(
                "switchboard-agent".to_string(),
                "latest".to_string(),
            ).await?)
        }
        _ => None,
    };
    
    match cli.command {
        Commands::Up(args) => run_up(args, cli.config, docker_client).await,
        Commands::Run(args) => run_run(args, cli.config, docker_client).await,
        Commands::Build(args) => run_build(args, cli.config, docker_client).await,
        Commands::Down(args) => run_down(args, cli.config, docker_client).await,
        Commands::List => run_list(cli.config),
        Commands::Logs(args) => run_logs(args, cli.config).await,
        Commands::Metrics(args) => run_metrics(args, cli.config),
        Commands::Validate(args) => run_validate(args, cli.config).await,
    }
}
```

3. **Update process management**:
```rust
// Replace Command::new with executor
pub async fn run_down(
    args: DownCommand,
    config_path: Option<String>,
    docker_client: Option<Arc<dyn DockerClientTrait>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let executor = Arc::new(RealProcessExecutor);
    
    // Stop scheduler with executor
    // ...
}
```

**Test Strategy**:
- Test CLI commands with mock Docker
- Test command parsing and dispatch
- Test error handling at CLI level
- Test command arguments handling

**Test Example**:
```rust
#[tokio::test]
async fn test_run_up_with_mock_docker() {
    let mut mock_docker = MockDockerClientTrait::new();
    mock_docker.expect_ping().returning(Ok(()));
    
    let args = UpCommand { detach: false };
    let config = Some("./test-config.toml".to_string());
    
    // This would require more setup to test full run_up
    // but the pattern demonstrates mock injection
}
```

---

### 4.7 config/mod.rs (MEDIUM → HIGH)

**Current Issues**:
- Uses `std::fs` operations directly
- File operations already testable with tempfile

**Changes Needed**:

Minimal changes needed. The config module primarily uses:
- `std::fs::read_to_string()` - Can use tempfile in tests
- `std::fs::write()` - Can use tempfile in tests

**Optional Enhancement**:

Add FileSystem trait for more control:

```rust
pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error>;
    fn write(&self, path: &Path, content: &str) -> Result<(), std::io::Error>;
    fn exists(&self, path: &Path) -> bool;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }
    
    fn write(&self, path: &Path, content: &str) -> Result<(), std::io::Error> {
        std::fs::write(path, content)
    }
    
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
```

**Test Strategy**:
- Continue using tempfile for file operations
- Optionally mock FileSystem for more control
- Test config parsing with various TOML formats
- Test validation logic

---

### 4.8 logger/mod.rs (HIGH → HIGH)

**Current Status**: Already testable with `TestWriter` pattern.

**No Changes Needed**:

The logger module already uses a trait-based pattern:

```rust
pub trait Writer: Send + Sync {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<()>;
    fn flush(&mut self) -> std::io::Result<()>;
}

pub struct TestWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Writer for TestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(bytes);
        Ok(())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
```

**Test Strategy**:
- Continue using `TestWriter` for logger tests
- Test file logger with tempfile
- Test terminal logger with captures

---

### 4.9 metrics/ (HIGH → HIGH)

**Current Status**: Already testable.

**No Changes Needed**:

The metrics module uses in-memory data structures and file operations that can be tested with tempfile.

**Test Strategy**:
- Continue testing metrics collection
- Test metrics serialization/deserialization
- Test concurrent metrics access

---

### 4.10 scheduler/clock.rs (HIGH → HIGH)

**Current Status**: Already testable with `MockClock` trait.

**No Changes Needed**:

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

pub struct MockClock {
    instant: Mutex<Option<Instant>>,
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        self.instant.lock().unwrap().unwrap_or_else(Instant::now)
    }
}
```

**Test Strategy**:
- Continue using MockClock for time-based tests
- Test scheduler behavior with controlled time
- Test timeout handling

---

## 5. Testing Strategy After Enhancement

### 5.1 Unit Testing Approach

**Principles**:
- **Fast**: Run in <100ms per test
- **Isolated**: No external dependencies (Docker, filesystem, network)
- **Deterministic**: Same inputs produce same outputs
- **Focused**: Test one unit of behavior per test

**Test Organization**:

```
tests/
├── unit/
│   ├── docker_unit.rs          # Docker operations
│   ├── scheduler_unit.rs       # Scheduler logic
│   ├── architect_unit.rs       # Architect workflow
│   ├── skills_unit.rs         # Skills management
│   ├── cli_unit.rs           # CLI commands
│   ├── config_unit.rs         # Config parsing
│   ├── logger_unit.rs        # Logger behavior
│   └── metrics_unit.rs       # Metrics collection
└── integration/
    ├── docker_integration.rs    # Real Docker tests
    ├── full_workflow.rs       # End-to-end tests
    └── process_integration.rs # Real process tests
```

**Mock Usage with mockall**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    
    mock! {
        DockerClientMock {}
        
        #[async_trait]
        impl DockerClientTrait for DockerClientMock {
            async fn ping(&self) -> Result<(), DockerError>;
            async fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;
            async fn run_container(&self, config: &ContainerConfig) 
                -> Result<AgentExecutionResult, DockerError>;
        }
    }
    
    #[tokio::test]
    async fn test_container_execution() {
        let mut mock = DockerClientMock::new();
        
        // Configure expected call
        mock.expect_run_container()
            .with(predicate::always())
            .times(1)
            .returning(|config| Ok(AgentExecutionResult {
                container_id: format!("container-{}", rand::random::<u32>()),
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                duration: Duration::from_secs(1),
            }));
        
        // Test the code
        let result = mock.run_container(&test_config()).await;
        assert!(result.is_ok());
    }
}
```

**Test Structure Pattern**:

```rust
#[tokio::test]
async fn test_<feature>_<scenario>() {
    // Arrange
    let mut mock_docker = MockDockerClientTrait::new();
    mock_docker.expect_<operation>()
        .with(<expectations>)
        .returning(<result>);
    
    let sut = SystemUnderTest::new(mock_docker);
    
    // Act
    let result = sut.<operation>(<input>).await;
    
    // Assert
    assert!(result.is_ok());
    // Additional assertions
}
```

---

### 5.2 Integration Testing Approach

**Principles**:
- **Feature-gated**: Only run when `integration` feature is enabled
- **Real Dependencies**: Use actual Docker, filesystem, processes
- **Comprehensive**: Cover happy paths and error scenarios
- **Environment-specific**: Can be skipped in CI if Docker unavailable

**Feature Flagging**:

```toml
# Cargo.toml
[features]
integration = []
```

```rust
#[cfg(all(test, feature = "integration"))]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Run explicitly with cargo test --features integration -- --ignored
    async fn test_real_docker_build() {
        let client = RealDockerClient::new("test-image".to_string(), "latest".to_string())
            .await
            .expect("Docker must be available");
        
        // Test real Docker operations
        let result = client.build_image(&options, context).await;
        assert!(result.is_ok());
    }
}
```

**Test Categories**:

1. **Happy Path Tests**:
   - Full scheduler lifecycle (up, run, down)
   - Agent execution with real containers
   - Config file parsing and validation

2. **Error Scenario Tests**:
   - Docker daemon unavailable
   - Invalid Docker images
   - Malformed config files
   - Timeout scenarios

3. **Concurrency Tests**:
   - Multiple agents running simultaneously
   - Overlap mode behavior (skip, queue)
   - Race conditions in state updates

**Running Integration Tests**:

```bash
# Run unit tests only (default)
cargo test

# Run integration tests
cargo test --features integration

# Run only integration tests
cargo test --features integration --test integration -- --ignored

# Run specific integration test
cargo test --features integration test_real_docker_build -- --ignored
```

---

### 5.3 Test Organization Structure

**Directory Layout**:

```
tests/
├── mod.rs                    # Test common utilities
├── common/
│   ├── mod.rs               # Shared test helpers
│   ├── fixtures.rs          # Test data fixtures
│   └── assertions.rs        # Custom assertion macros
├── unit/
│   ├── docker_unit.rs       # Docker operations
│   ├── scheduler_unit.rs    # Scheduler logic
│   ├── architect_unit.rs    # Architect workflow
│   ├── skills_unit.rs      # Skills management
│   ├── cli_unit.rs        # CLI commands
│   ├── config_unit.rs      # Config parsing
│   └── mod.rs             # Unit test module
└── integration/
    ├── docker_integration.rs  # Real Docker tests
    ├── full_workflow.rs     # End-to-end tests
    └── mod.rs             # Integration test module
```

**Common Utilities (tests/common/mod.rs)**:

```rust
use tempfile::TempDir;
use std::path::PathBuf;

/// Create a temporary directory for tests
pub fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Create a test agent configuration
pub fn test_agent(name: &str) -> Agent {
    Agent {
        name: name.to_string(),
        prompt: Some(format!("Test prompt for {}", name)),
        schedule: "* * * * *".to_string(),
        ..Default::default()
    }
}

/// Create a test config file
pub fn test_config() -> String {
    r#"
[settings]
image_name = "test-image"
image_tag = "latest"
log_dir = ".switchboard/logs"

[[agents]]
name = "test-agent"
schedule = "0 * * * *"
prompt = "Test prompt"
    "#.to_string()
}

/// Assert that result is error with specific message
pub fn assert_error_contains<T, E>(result: Result<T, E>, expected: &str)
where
    E: std::error::Error,
{
    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains(expected),
                "Expected error to contain '{}', got: '{}'",
                expected, msg
            );
        }
        Ok(_) => panic!("Expected error, got Ok"),
    }
}
```

---

### 5.4 Leveraging mockall for Mocks

**Setting Up mockall**:

```toml
# Cargo.toml
[dev-dependencies]
mockall = "0.12"
```

**Creating Mocks**:

```rust
use mockall::mock;

// Generate mock struct for trait
mock! {
    DockerClientMock {}
    
    #[async_trait]
    impl DockerClientTrait for DockerClientMock {
        async fn ping(&self) -> Result<(), DockerError>;
        async fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;
        async fn run_container(&self, config: &ContainerConfig) 
            -> Result<AgentExecutionResult, DockerError>;
    }
}
```

**Configuring Mock Behavior**:

```rust
#[tokio::test]
async fn test_with_mockall() {
    let mut mock = DockerClientMock::new();
    
    // Simple return value
    mock.expect_ping()
        .returning(Ok(()));
    
    // Conditional return with predicates
    mock.expect_image_exists()
        .with(eq("test-image"), eq("latest"))
        .returning(Ok(true));
    
    mock.expect_image_exists()
        .with(eq("nonexistent"), always())
        .returning(Ok(false));
    
    // Call count verification
    mock.expect_run_container()
        .times(2)
        .returning(|_| Ok(test_result()));
    
    // Sequence of return values
    mock.expect_ping()
        .times(3)
        .returning_const(Ok(()))
        .returning(Err(DockerError::ConnectionError("test".to_string())))
        .returning(Ok(()));
}
```

**Checking Expectations**:

```rust
#[tokio::test]
async fn test_expectation_verification() {
    let mut mock = DockerClientMock::new();
    mock.expect_ping().times(1).returning(Ok(()));
    
    // Execute test
    mock.ping().await.unwrap();
    
    // Verify all expectations met (automatically on drop)
    // Explicit checkpoint:
    mock.checkpoint();
}
```

**Testing Error Scenarios**:

```rust
#[tokio::test]
async fn test_error_handling() {
    let mut mock = DockerClientMock::new();
    
    // Configure mock to return error
    mock.expect_ping()
        .returning(Err(DockerError::ConnectionTimeout {
            timeout_duration: "5s".to_string(),
            suggestion: "test suggestion".to_string(),
        }));
    
    let result = mock.ping().await;
    assert!(result.is_err());
    
    match result {
        Err(DockerError::ConnectionTimeout { timeout_duration, .. }) => {
            assert_eq!(timeout_duration, "5s");
        }
        _ => panic!("Expected ConnectionTimeout error"),
    }
}
```

---

## 6. Risk Assessment & Mitigation

### 6.1 Breaking Changes Risk

**Risk Level**: MEDIUM

**Description**: Introducing trait-based dependency injection requires changing function signatures across multiple modules. This is a breaking change for any external code that uses these modules directly.

**Mitigation Strategies**:

1. **Backward Compatibility Layer**:
```rust
// Keep old API for backward compatibility
impl DockerClient {
    #[deprecated(since = "0.2.0", note = "Use new_with_client for testability")]
    pub async fn new(image_name: String, image_tag: String) -> Result<Self, DockerError> {
        Self::new_with_client(image_name, image_tag, 
            Arc::new(RealDockerClient::new()?)).await
    }
}
```

2. **Gradual Migration**:
   - Introduce traits alongside existing concrete types
   - Mark old methods as deprecated
   - Allow both approaches to coexist during transition period
   - Remove deprecated methods in next major version

3. **Versioning**:
   - Document breaking changes in CHANGELOG.md
   - Use semantic versioning (bump to 0.2.0)
   - Provide migration guide in documentation

**Acceptance Criteria**:
- Existing tests continue to pass during migration
- Deprecation warnings are clear and actionable
- Migration guide covers all breaking changes

---

### 6.2 Backward Compatibility

**Risk Level**: MEDIUM

**Description**: Changes to public APIs may break downstream projects that depend on switchboard-rs.

**Mitigation Strategies**:

1. **Type Aliases**:
```rust
// Provide type aliases for backward compatibility
pub type DockerClient = RealDockerClient;
pub type ProcessExecutor = RealProcessExecutor;
```

2. **Default Implementations**:
```rust
// Provide constructors that use default implementations
impl DockerClient {
    pub async fn new(image_name: String, image_tag: String) -> Result<Self, DockerError> {
        Self::new_with_client(
            image_name,
            image_tag,
            Arc::new(RealDockerClient::new()?),
        ).await
    }
}
```

3. **Feature Flags**:
```rust
// Allow opting out of trait-based API
#[cfg(feature = "legacy")]
pub struct LegacyDockerClient {
    docker: Docker,
}
```

**Acceptance Criteria**:
- Existing code compiles without modification
- Type aliases preserve public API surface
- Documentation clearly indicates deprecation path

---

### 6.3 Performance Impact

**Risk Level**: LOW

**Description**: Adding trait-based abstractions and virtual dispatch may introduce a small performance overhead due to dynamic dispatch.

**Analysis**:

1. **Trait Object Overhead**:
   - Dynamic dispatch adds ~1-2 CPU cycles per method call
   - Negligible for I/O-bound operations (Docker, filesystem)
   - Impact is minimal compared to network latency

2. **Arc Overhead**:
   - Arc adds reference counting (~10 cycles per clone)
   - Only affects cloning, not read operations
   - Necessary for shared ownership across async tasks

3. **Memory Overhead**:
   - Trait objects use fat pointers (16 bytes on 64-bit)
   - Minimal impact given typical Docker operation latency (100ms+)

**Mitigation Strategies**:

1. **Keep Critical Path Fast**:
   - Only use trait objects at module boundaries
   - Internal functions can use concrete types
   - Profile before and after to verify no regression

2. **Generic Where Possible**:
   - Use generics for compile-time polymorphism in hot paths
   - Only use trait objects where dynamic dispatch is required
   - Example:
   ```rust
   // Fast: compile-time dispatch
   pub async fn run_agent_generic<T: DockerClientTrait>(
       config: ContainerConfig,
       docker: T,
   ) -> Result<AgentExecutionResult, DockerError>
   {
       docker.run_container(&config).await
   }
   
   // Flexible: runtime dispatch
   pub async fn run_agent_trait(
       config: ContainerConfig,
       docker: Arc<dyn DockerClientTrait>,
   ) -> Result<AgentExecutionResult, DockerError>
   {
       docker.run_container(&config).await
   }
   ```

3. **Benchmarking**:
   - Add benchmarks for critical operations
   - Compare before and after refactoring
   - Use criterion crate for statistical analysis

**Acceptance Criteria**:
- No measurable performance regression in benchmarks
- I/O-bound operations unchanged
- Memory usage within 5% of baseline

---

### 6.4 Testing Timeline Impact

**Risk Level**: MEDIUM

**Description**: Writing comprehensive tests after refactoring takes significant time. Delays may impact feature delivery.

**Mitigation Strategies**:

1. **Incremental Testing**:
   - Write tests as modules are refactored
   - Don't defer all testing to end
   - Test each phase before moving to next

2. **Coverage Targets**:
   - Phase 1-2: 50% coverage of new code
   - Phase 3: 60% coverage of affected modules
   - Phase 4: 70% coverage overall
   - Phase 5: 80%+ coverage final

3. **Parallel Development**:
   - One developer refactors modules
   - Another writes tests in parallel
   - Code reviews focus on test quality

4. **Test Debt Tracking**:
   - Document areas that need more tests
   - Create backlog items for test improvements
   - Pay down debt in iterations

**Timeline Breakdown**:

| Phase | Development | Testing | Review | Total |
|--------|-------------|----------|--------|--------|
| 1: Foundation | 2 days | 1 day | 0.5 days | 3.5 days |
| 2: Docker | 4 days | 2 days | 1 day | 7 days |
| 3: Process | 3 days | 1.5 days | 0.5 days | 5 days |
| 4: Logger/Metrics | 2 days | 1 day | 0.5 days | 3.5 days |
| 5: Test Suite | 3 days | 3 days | 1 day | 7 days |
| **Total** | **14 days** | **8.5 days** | **3.5 days** | **26 days** |

**Acceptance Criteria**:
- Each phase passes tests before proceeding
- Code coverage measured and reported
- Testing debt tracked and prioritized

---

### 6.5 Additional Risks

**Mock Complexity**:
- **Risk**: Mocks become difficult to maintain as codebase evolves
- **Mitigation**: Keep mock setup close to test, use helpers, document mock contracts

**Over-Mocking**:
- **Risk**: Tests become brittle by mocking too much
- **Mitigation**: Only mock external dependencies, test real business logic

**Test False Positives**:
- **Risk**: Mocks return unrealistic data, tests pass but bugs exist
- **Mitigation**: Integration tests catch this, use realistic test data

**Refactoring Fatigue**:
- **Risk**: Long refactoring loses momentum
- **Mitigation**: Small incremental changes, celebrate milestones, track progress

---

## 7. Success Criteria

### 7.1 Coverage Metrics

**Target Coverage by Module**:

| Module | Current Coverage | Target Coverage | Priority |
|--------|-----------------|-----------------|----------|
| docker/mod.rs | 0% | 85% | CRITICAL |
| docker/run.rs | 0% | 85% | CRITICAL |
| scheduler/mod.rs | 30% | 80% | HIGH |
| architect/mod.rs | 0% | 85% | HIGH |
| architect/state.rs | 20% | 85% | HIGH |
| skills/mod.rs | 0% | 85% | HIGH |
| cli/mod.rs | 15% | 75% | MEDIUM |
| config/mod.rs | 70% | 80% | MEDIUM |
| logger/mod.rs | 60% | 75% | LOW |
| metrics/ | 50% | 75% | LOW |

**Overall Project Coverage**:
- **Current**: ~35%
- **Target (Phase 5)**: 80%+
- **Stretch Goal**: 85%

**Measurement**:
```bash
# Generate coverage report
cargo llvm-cov --html --output-dir coverage

# View coverage by module
cargo llvm-cov --open
```

---

### 7.2 Test Count Goals

**Unit Tests**:
- **Current**: ~50 tests
- **Target (Phase 5)**: 300+ tests
- **Breakdown**:
  - Docker operations: 50 tests
  - Scheduler logic: 60 tests
  - Architect workflow: 40 tests
  - Skills management: 30 tests
  - CLI commands: 50 tests
  - Config parsing: 40 tests
  - Other modules: 30 tests

**Integration Tests**:
- **Current**: ~20 tests
- **Target (Phase 5)**: 50+ tests
- **Breakdown**:
  - Docker operations: 15 tests
  - Full workflow: 10 tests
  - Error scenarios: 15 tests
  - Concurrency: 10 tests

**Test Execution Time**:
- **Unit Test Suite**: <30 seconds
- **Integration Test Suite**: <5 minutes
- **Full Test Suite**: <5.5 minutes

---

### 7.3 Module Testability Definition

**"Fully Testable in Isolation"** means each module can:

1. **Run all unit tests** without:
   - Running Docker daemon
   - Executing external processes
   - Accessing real filesystem (except through tempfile)
   - Making network requests
   - Waiting for time-based delays

2. **Test all public functions** with:
   - Deterministic inputs and outputs
   - Configurable mock behavior
   - Verifiable error scenarios
   - Edge case coverage

3. **Verify core business logic** including:
   - State transitions
   - Error handling
   - Validation logic
   - Scheduling behavior

4. **Achieve >80% code coverage** for:
   - All public functions
   - Internal helper functions
   - Error paths
   - Edge cases

**Module-Specific Success Criteria**:

#### docker/mod.rs
- [ ] Docker connection logic testable
- [ ] Image building logic testable
- [ ] Context parsing testable
- [ ] Error handling testable
- [ ] Mock client implementation available
- [ ] >85% coverage

#### docker/run.rs
- [ ] Container creation testable
- [ ] Log streaming testable
- [ ] Timeout handling testable
- [ ] Exit status parsing testable
- [ ] Mockable with DockerClientTrait
- [ ] >85% coverage

#### scheduler/mod.rs
- [ ] Agent registration testable
- [ ] Overlap detection testable
- [ ] Queue management testable
- [ ] Cron scheduling testable
- [ ] Scheduler lifecycle testable
- [ ] Mockable Docker client
- [ ] >80% coverage

#### architect/mod.rs
- [ ] Session protocol testable
- [ ] State persistence testable
- [ ] Git operations mockable
- [ ] Session lifecycle testable
- [ ] Mockable process executor
- [ ] >85% coverage

#### architect/state.rs
- [ ] State loading testable
- [ ] State saving testable
- [ ] Marker file management testable
- [ ] Git integration mockable
- [ ] Mockable process executor
- [ ] >85% coverage

#### skills/mod.rs
- [ ] Skill listing testable
- [ ] Skill installation testable
- [ ] Skill discovery testable
- [ ] NPX calls mockable
- [ ] Mockable process executor
- [ ] >85% coverage

#### cli/mod.rs
- [ ] Command parsing testable
- [ ] Command dispatch testable
- [ ] Error handling testable
- [ ] Docker operations mockable
- [ ] >75% coverage

#### config/mod.rs
- [ ] Config parsing testable (already done)
- [ ] Validation logic testable (already done)
- [ ] File operations testable
- [ ] >80% coverage

#### logger/mod.rs
- [ ] Logger creation testable (already done)
- [ ] Log writing testable (already done)
- [ ] File logger testable
- [ ] Terminal logger testable
- [ ] >75% coverage

#### metrics/
- [ ] Metrics collection testable (already done)
- [ ] Metrics storage testable
- [ ] Metrics serialization testable
- [ ] >75% coverage

---

### 7.4 Quality Metrics

**Test Quality Indicators**:
- **Test-to-Code Ratio**: >1.5 lines of test per line of code
- **Flaky Test Rate**: <1% of tests
- **Test Execution Time**: <30s for unit suite
- **Mock Coverage**: All traits have mock implementations
- **Documentation Coverage**: All tests have docstrings explaining purpose

**Code Quality Indicators**:
- **Lint Passes**: clippy warnings = 0
- **Format Passes**: rustfmt passes
- **Type Safety**: No unsafe code
- **Error Handling**: All Result types properly handled

---

### 7.5 CI/CD Integration

**Continuous Integration Requirements**:
- [ ] Unit tests run on every PR
- [ ] Integration tests run on merge
- [ ] Coverage generated on every build
- [ ] Coverage gates set at 70% minimum
- [ ] Tests run on Windows, macOS, Linux

**Success Criteria**:
- All CI tests pass consistently
- Coverage reports are viewable
- Build time <10 minutes

---

## 8. Appendix: Quick Reference

### 8.1 Module Testability Summary Table

| Module | Current Testability | Target Testability | Key Changes Needed |
|--------|-------------------|-------------------|-------------------|
| `docker/mod.rs` | NO | HIGH | Implement `DockerClientTrait`, inject trait |
| `docker/run.rs` | NO | HIGH | Use `DockerClientTrait` for all operations |
| `scheduler/mod.rs` | LOW | HIGH | Add `docker_client` field, inject trait |
| `architect/mod.rs` | NO | HIGH | Implement `ProcessExecutorTrait`, inject trait |
| `architect/state.rs` | NO | HIGH | Use `ProcessExecutorTrait` for git commands |
| `skills/mod.rs` | NO | HIGH | Use `ProcessExecutorTrait` for npx calls |
| `cli/mod.rs` | LOW | HIGH | Inject `DockerClientTrait` and `ProcessExecutorTrait` |
| `config/mod.rs` | MEDIUM | HIGH | Minimal changes, already testable with tempfile |
| `logger/mod.rs` | HIGH | HIGH | No changes needed |
| `metrics/` | HIGH | HIGH | No changes needed |
| `scheduler/clock.rs` | HIGH | HIGH | No changes needed |

### 8.2 Trait Quick Reference

#### DockerClientTrait
```rust
#[async_trait]
pub trait DockerClientTrait: Send + Sync {
    async fn ping(&self) -> Result<(), DockerError>;
    async fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;
    async fn build_image(&self, options: &BuildImageOptions<str>, context: impl Into<Bytes> + Send) -> Result<(), DockerError>;
    async fn run_container(&self, config: &ContainerConfig) -> Result<AgentExecutionResult, DockerError>;
    async fn stop_container(&self, container_id: &str, timeout: Option<Duration>) -> Result<(), DockerError>;
    async fn container_logs(&self, container_id: &str, follow: bool, tail: Option<u64>) -> Result<impl Stream<Item = Result<bytes::Bytes, DockerError>> + Send, DockerError>;
    async fn wait_container(&self, container_id: &str, timeout: Option<Duration>) -> Result<ExitStatus, DockerError>;
}
```

#### ProcessExecutorTrait
```rust
#[async_trait]
pub trait ProcessExecutorTrait: Send + Sync {
    async fn execute(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError>;
    async fn execute_with_env(&self, program: &str, args: &[&str], env: &[(impl AsRef<str> + Send)], working_dir: Option<&Path>) -> Result<ProcessOutput, ProcessError>;
}
```

#### Clock (Already Implemented)
```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}
```

### 8.3 Mock Setup Quick Reference

#### Create Mock with mockall
```rust
use mockall::mock;

mock! {
    DockerClientMock {}
    
    #[async_trait]
    impl DockerClientTrait for DockerClientMock {
        async fn ping(&self) -> Result<(), DockerError>;
        async fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;
    }
}
```

#### Configure Mock Expectations
```rust
let mut mock = DockerClientMock::new();

// Simple return
mock.expect_ping().returning(Ok(()));

// With arguments
mock.expect_image_exists()
    .with(eq("image"), eq("tag"))
    .returning(Ok(true));

// Multiple times
mock.expect_ping().times(3).returning(Ok(()));

// Error case
mock.expect_ping()
    .returning(Err(DockerError::ConnectionError("test".to_string())));
```

### 8.4 Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    
    // Setup mock
    mock! {
        DockerClientMock {}
        
        #[async_trait]
        impl DockerClientTrait for DockerClientMock {
            async fn ping(&self) -> Result<(), DockerError>;
        }
    }
    
    #[tokio::test]
    async fn test_<feature>_<scenario>() {
        // Arrange
        let mut mock = DockerClientMock::new();
        mock.expect_ping().returning(Ok(()));
        
        // Act
        let result = mock.ping().await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### 8.5 Phase Checklist

#### Phase 1: Foundation
- [ ] Add mockall to dev-dependencies
- [ ] Add async-trait to dependencies
- [ ] Create `src/traits/mod.rs`
- [ ] Define `DockerClientTrait`
- [ ] Define `ProcessExecutorTrait`
- [ ] Create error types
- [ ] Document all traits

#### Phase 2: Docker Abstraction
- [ ] Implement `RealDockerClient`
- [ ] Refactor `docker/mod.rs`
- [ ] Refactor `docker/run.rs`
- [ ] Refactor `scheduler/mod.rs`
- [ ] Refactor `cli/mod.rs`
- [ ] Create integration tests

#### Phase 3: Process Abstraction
- [ ] Implement `RealProcessExecutor`
- [ ] Refactor `architect/state.rs`
- [ ] Refactor `skills/mod.rs`
- [ ] Refactor `docker/mod.rs` (context)
- [ ] Create unit tests

#### Phase 4: Logger/Metrics (Optional)
- [ ] Define `LoggerProvider` trait
- [ ] Define `MetricsProvider` trait
- [ ] Implement providers
- [ ] Refactor modules to use providers
- [ ] Create tests

#### Phase 5: Test Suite
- [ ] Write unit tests for all modules
- [ ] Write integration tests
- [ ] Achieve coverage targets
- [ ] Set up CI/CD
- [ ] Document testing practices

### 8.6 Common Testing Patterns

#### Pattern: Mock Construction
```rust
let mut mock = MockDockerClientTrait::new();
mock.expect_operation()
    .with(predicate)
    .returning(value);
```

#### Pattern: Test with TempDir
```rust
#[test]
fn test_with_temp_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    // Use temp directory for file operations
}
```

#### Pattern: Async Test
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

#### Pattern: Error Testing
```rust
#[tokio::test]
async fn test_error_case() {
    let mut mock = MockTrait::new();
    mock.expect_method()
        .returning(Err(Error::Variant));
    
    let result = method_under_test().await;
    assert!(result.is_err());
}
```

### 8.7 Migration Guide

#### For Existing Code Using DockerClient
```rust
// Old way
let client = DockerClient::new("image".to_string(), "latest".to_string()).await?;

// New way (with backward compatibility)
let client = DockerClient::new("image".to_string(), "latest".to_string()).await?;

// New way (for testing)
let mock = MockDockerClientTrait::new();
let client = DockerClient::new_with_client(
    "image".to_string(),
    "latest".to_string(),
    Arc::new(mock),
).await?;
```

#### For Existing Code Calling Processes
```rust
// Old way
let output = Command::new("git")
    .args(["commit", "-m", "message"])
    .output()?;

// New way
let executor = Arc::new(RealProcessExecutor);
let output = executor.execute("git", &["commit", "-m", "message"]).await?;
```

---

## Conclusion

This testability enhancement plan provides a comprehensive, phased approach to achieving full testability across the switchboard-rs CLI codebase. By introducing trait-based dependency injection for Docker operations and process execution, we unlock the ability to write fast, deterministic unit tests for approximately 70% of the currently untestable code.

The plan prioritizes the highest-impact changes first (Docker abstraction), then progressively adds abstractions for other external dependencies. Each phase includes clear success criteria, risk mitigation strategies, and concrete examples to guide implementation.

Following this plan will result in:
- 80%+ test coverage across all modules
- 300+ unit tests and 50+ integration tests
- Fast test suites that run in under 30 seconds
- Confidence to refactor and extend the codebase
- Improved code quality through test-driven development

The investment of approximately 26 development days will pay dividends throughout the lifetime of the project by enabling faster development cycles, fewer bugs, and more confident refactoring.
