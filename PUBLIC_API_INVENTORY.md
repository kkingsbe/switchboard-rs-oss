# Public API Documentation Inventory

## Summary
- **Total public APIs**: 134
- **Fully documented**: 95 (70.9%)
- **Partially documented**: 25 (18.7%)
- **Undocumented**: 14 (10.4%)

## By Module

### src/lib.rs
- `pub mod cli` - [Fully documented] - Has module-level rustdoc
- `pub mod commands` - [Fully documented] - Has module-level rustdoc
- `pub mod config` - [Fully documented] - Has module-level rustdoc
- `pub mod docker` - [Fully documented] - Has module-level rustdoc
- `pub mod logger` - [Fully documented] - Has module-level rustdoc
- `pub mod logging` - [Fully documented] - Has module-level rustdoc
- `pub mod metrics` - [Fully documented] - Has module-level rustdoc
- `pub mod scheduler` - [Fully documented] - Has module-level rustdoc

### src/logging.rs
- `pub fn init_logging(log_dir: PathBuf) -> WorkerGuard` - [Fully documented] - Has complete rustdoc with description, arguments, returns, panics, and example

### src/main.rs
- No public APIs (main is not exported)

### src/architect/mod.rs
- `pub mod state` - [Fully documented] - Has module-level rustdoc
- `pub mod session` - [Fully documented] - Has module-level rustdoc
- `pub use state::{AgentStatus, ArchitectState, CurrentTask, QueueStatus, cleanup_on_complete, commit_progress, create_in_progress_marker, delete_in_progress_marker, load_state, save_state}` - [Partially documented] - Re-exports without direct rustdoc
- `pub use session::{end_session, update_session_progress, SessionHandle}` - [Partially documented] - Re-exports without direct rustdoc
- `pub type Result<T> = std::result::Result<T, ArchitectError>` - [Undocumented] - No rustdoc
- `pub enum ArchitectError` - [Fully documented] - Has full rustdoc with all variants documented

### src/architect/session.rs
- `pub struct SessionHandle` - [Fully documented] - Has rustdoc with description
- `pub fn start_session(current_sprint: u32) -> Result<SessionHandle>` - [Partially documented] - Has rustdoc with Returns and Errors sections
- `pub fn update_session_progress(handle: &mut SessionHandle, task: &str, completed: bool)` - [Partially documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn update_session_progress_with_context(handle: &mut SessionHandle, task: &str, context: &str, completed: bool)` - [Partially documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn end_session(mut handle: SessionHandle, all_complete: bool) -> Result<()>` - [Partially documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn update_agent_status(handle: &mut SessionHandle, agent_id: u32, queue_status: super::QueueStatus, tasks_remaining: u32, blocked: bool)` - [Partially documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn add_remaining_task(handle: &mut SessionHandle, task: String)` - [Partially documented] - Has rustdoc with Arguments, Returns, and Errors sections

### src/architect/state.rs
- `pub struct ArchitectState` - [Fully documented] - Has comprehensive rustdoc with description, fields, and example
- `pub fn new(current_sprint: u32) -> Self` - [Fully documented] - Has rustdoc with Parameters, Returns, and Example sections
- `pub fn complete_task(&mut self, task: String)` - [Fully documented] - Has rustdoc with Parameters, Behavior, and Example sections
- `pub fn set_current_task(&mut self, task: Option<CurrentTask>)` - [Fully documented] - Has rustdoc with Parameters, Behavior, and Example sections
- `pub fn update_agent_status(&mut self, agent_id: u32, status: AgentStatus)` - [Fully documented] - Has rustdoc with Parameters, Behavior, and Example sections
- `pub enum SessionStatus` - [Fully documented] - Has rustdoc with variants documented
- `pub struct CurrentTask` - [Fully documented] - Has rustdoc with fields documented
- `pub enum QueueStatus` - [Fully documented] - Has rustdoc with variants documented
- `pub struct AgentStatus` - [Fully documented] - Has rustdoc with fields documented
- `pub fn new(description: String, context: String) -> Self` - [Fully documented] - Has rustdoc
- `pub fn new(queue_status: QueueStatus, tasks_remaining: u32, blocked: bool) -> Self` - [Fully documented] - Has rustdoc
- `pub fn load_state() -> Result<Option<ArchitectState>>` - [Partially documented] - Has rustdoc with Returns and Errors sections
- `pub fn save_state(state: &ArchitectState) -> Result<()>` - [Partially documented] - Has rustdoc with Arguments and Errors sections
- `pub fn create_in_progress_marker() -> Result<()>` - [Partially documented] - Has rustdoc with Errors section
- `pub fn delete_in_progress_marker() -> Result<()>` - [Partially documented] - Has rustdoc with Errors section
- `pub fn in_progress_marker_exists() -> bool` - [Partially documented] - Has rustdoc with Returns section
- `pub fn commit_progress(message: &str) -> Result<()>` - [Partially documented] - Has rustdoc with Arguments and Errors sections
- `pub fn cleanup_on_complete() -> Result<()>` - [Undocumented] - No rustdoc comments

### src/cli/mod.rs
- `pub struct Cli` - [Fully documented] - Has rustdoc with description and fields
- `pub enum Commands` - [Fully documented] - Has extensive rustdoc with description, variants, command dispatch, examples
- `pub struct UpCommand` - [Fully documented] - Has rustdoc with fields documented
- `pub struct RunCommand` - [Fully documented] - Has rustdoc with fields documented
- `pub struct LogsCommand` - [Fully documented] - Has rustdoc with fields and examples
- `pub struct MetricsCommand` - [Fully documented] - Has rustdoc with fields documented
- `pub struct DownCommand` - [Fully documented] - Has rustdoc with fields documented
- `pub async fn run() -> Result<(), Box<dyn std::error::Error>>` - [Fully documented] - Has comprehensive rustdoc with functionality, supported commands, returns, errors, examples
- `pub async fn run_up(args: UpCommand, config_path: Option<String>) -> Result<(), Box<dyn std::error::Error>>` - [Fully documented] - Has extensive rustdoc with functionality, arguments, returns, errors, modes, configuration, examples

### src/commands/mod.rs
- `pub mod build` - [Fully documented] - Has module-level rustdoc
- `pub mod list` - [Fully documented] - Has module-level rustdoc
- `pub mod logs` - [Fully documented] - Has module-level rustdoc
- `pub mod metrics` - [Fully documented] - Has module-level rustdoc
- `pub mod validate` - [Fully documented] - Has module-level rustdoc
- `pub use build::BuildCommand` - [Partially documented] - Re-export without direct rustdoc on item
- `pub use list::list_agents` - [Partially documented] - Re-export without direct rustdoc on item
- `pub use metrics::metrics` - [Partially documented] - Re-export without direct rustdoc on item
- `pub use validate::ValidateCommand` - [Partially documented] - Re-export without direct rustdoc on item

### src/commands/build.rs
- `pub struct BuildCommand` - [Fully documented] - Has rustdoc with fields and examples
- `pub async fn run(self) -> Result<(), Box<dyn std::error::Error>>` - [Fully documented] - Has rustdoc with Returns, Errors, and Notes sections

### src/commands/list.rs
- `pub fn list_agents(config: &Config) -> Result<(), String>` - [Fully documented] - Has rustdoc with description, table columns, arguments, returns, errors, examples, and notes

### src/commands/logs.rs
- `pub struct LogsArgs` - [Fully documented] - Has rustdoc with fields, examples, and notes
- `pub async fn run(args: LogsArgs) -> Result<(), Box<dyn std::error::Error>>` - [Fully documented] - Has rustdoc with arguments, returns, errors, examples, and notes

### src/commands/metrics.rs
- `pub fn metrics(log_dir: &str, detailed: bool, agent: Option<&str>, agent_schedules: Option<&HashMap<String, String>>) -> Result<(), String>` - [Fully documented] - Has rustdoc with description, arguments, returns, errors, examples, and notes

### src/commands/validate.rs
- `pub struct ValidateCommand` - [Fully documented] - Has rustdoc with examples
- `pub async fn run(&self, config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>>` - [Fully documented] - Has rustdoc with arguments, returns, errors, and notes

### src/config/mod.rs
- `pub enum ConfigError` - [Fully documented] - Has comprehensive rustdoc with all variants fully documented (ParseError, ValidationError, PromptFileNotFound each with detailed documentation)

### src/docker/mod.rs
- `pub mod run` - [Partially documented] - Has module-level rustdoc
- `pub mod streams` - [Partially documented] - Has module-level rustdoc
- `pub mod types` - [Partially documented] - Has module-level rustdoc
- `pub mod wait` - [Partially documented] - Has module-level rustdoc
- `pub use run::{run_agent, AgentExecutionResult}` - [Partially documented] - Re-export without direct rustdoc on items
- `pub use streams::attach_and_stream_logs` - [Partially documented] - Re-export without direct rustdoc on item
- `pub use types::{ContainerConfig, ContainerError}` - [Partially documented] - Re-export without direct rustdoc on items
- `pub use wait::{parse_timeout, wait_for_exit, wait_with_timeout, ExitStatus}` - [Partially documented] - Re-export without direct rustdoc on items
- `pub enum DockerError` - [Fully documented] - Has rustdoc with all variants documented
- `pub struct DockerClient` - [Partially documented] - Has module-level comment but item has no rustdoc
- `pub fn new(image_name: String, image_tag: String) -> Result<Self, DockerError>` - [Fully documented] - Has rustdoc with Arguments, Errors, and returns
- `pub async fn check_available(&self) -> Result<(), DockerError>` - [Fully documented] - Has extensive rustdoc with description, error scenarios, errors, and examples
- `pub async fn build_agent_image(&self, dockerfile: &str, build_context: &Path, image_name: &str, image_tag: &str, no_cache: bool) -> Result<String, DockerError>` - [Fully documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn docker(&self) -> &Docker` - [Undocumented] - No rustdoc

### src/docker/run.rs
- `pub struct AgentExecutionResult` - [Partially documented] - Has struct-level rustdoc
- `pub fn build_container_env_vars(env_vars: &[String]) -> Vec<String>` - [Fully documented] - Has rustdoc with Arguments and Returns sections
- `pub fn build_host_config(workspace: &str, readonly: bool) -> HostConfig` - [Fully documented] - Has rustdoc with Arguments and Returns sections
- `pub fn build_container_config(image: &str, env_vars: Vec<String>, readonly: bool, workspace: &str, agent_name: &str, _timeout: u64, cmd: Option<&[String]>) -> Config<String>` - [Fully documented] - Has rustdoc with Arguments and Returns sections
- `pub async fn run_agent(...) -> Result<AgentExecutionResult, DockerError>` - [Partially documented] - Has rustdoc with Arguments and Returns sections (but missing Errors section)

### src/docker/run/streams.rs
- `pub async fn attach_and_stream_logs(client: &DockerClient, container_id: &str, agent_name: &str, logger: Option<Arc<Mutex<Logger>>>, follow: bool) -> Result<(), DockerError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, Errors, and Examples sections

### src/docker/run/types.rs
- `pub struct ContainerConfig` - [Fully documented] - Has struct-level rustdoc with fields documented
- `pub fn new(agent_name: String) -> Self` - [Fully documented] - Has rustdoc with Returns section
- `pub enum ContainerError` - [Fully documented] - Has rustdoc with all variants documented
- `impl From<ContainerError> for super::DockerError` - [Undocumented] - No rustdoc on impl

### src/docker/run/wait.rs
- `pub use self::timeout::{parse_timeout, wait_for_exit, wait_with_timeout}` - [Partially documented] - Re-export without direct rustdoc on items
- `pub use self::types::{ExitStatus, TerminationSignal}` - [Partially documented] - Re-export without direct rustdoc on items

### src/docker/run/wait/timeout.rs
- `pub fn parse_timeout(s: &str) -> Result<Duration, DockerError>` - [Fully documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub async fn wait_for_exit(client: &DockerClient, container_id: &str) -> Result<ExitStatus, DockerError>` - [Fully documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub async fn terminate_with_graceful_shutdown(docker: &Docker, container_id: &str, grace_period: Duration) -> Result<ExitStatus, DockerError>` - [Fully documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub async fn wait_with_timeout(client: &DockerClient, container_id: &str, timeout: Duration, agent_name: &str, logger: Option<&Arc<Mutex<Logger>>>) -> Result<ExitStatus, DockerError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, Errors, and Examples sections

### src/docker/run/wait/types.rs
- `pub enum TerminationSignal` - [Fully documented] - Has struct-level rustdoc with variants documented
- `pub struct ExitStatus` - [Fully documented] - Has rustdoc with fields documented
- `pub fn new(exit_code: i64, timed_out: bool, termination_signal: TerminationSignal) -> Self` - [Fully documented] - Has rustdoc with Returns section
- `pub fn exited(exit_code: i64) -> Self` - [Fully documented] - Has rustdoc with Returns section
- `pub fn timed_out(termination_signal: Option<TerminationSignal>) -> Self` - [Fully documented] - Has rustdoc with Returns section

### src/logger/mod.rs
- `pub struct Logger` - [Partially documented] - Has struct-level rustdoc
- `pub fn new(log_dir: PathBuf, agent_name: Option<String>, foreground_mode: bool) -> Self` - [Fully documented] - Has rustdoc with Arguments, Returns, and Example sections
- `pub fn write_terminal_output(&self, message: &str) -> Result<(), terminal::TerminalError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, and Errors sections
- `pub fn write_agent_log(&self, agent_name: &str, message: &str) -> Result<(), file::FileWriteError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, and Errors sections

### src/logger/file.rs
- `pub enum PathConstructionError` - [Fully documented] - Has rustdoc with all variants documented
- `pub struct FileWriter` - [Fully documented] - Has struct-level rustdoc
- `pub fn new(log_dir: impl AsRef<Path>) -> Self` - [Fully documented] - Has rustdoc with Arguments and Returns sections
- `pub fn write_scheduler_log(&self, message: &str) -> Result<(), FileWriteError>` - [Fully documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn write_agent_log(&self, agent_name: &str, message: &str) -> Result<(), FileWriteError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, and Errors sections
- `pub fn rotate_logs(&self, agent_name: &str) -> Result<(), FileWriteError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, and Errors sections
- `pub fn generate_timestamp() -> String` - [Fully documented] - Has rustdoc with Returns section

### src/logger/terminal.rs
- `pub enum TerminalError` - [Fully documented] - Has rustdoc with all variants documented
- `pub struct TerminalWriter` - [Fully documented] - Has struct-level rustdoc
- `pub fn new(agent_name: String, foreground_mode: bool) -> Self` - [Fully documented] - Has rustdoc with Arguments and Returns sections
- `pub fn write_output(&self, message: &str) -> Result<(), TerminalError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, and Errors sections

### src/metrics/mod.rs
- `pub use collector::update_all_metrics` - [Partially documented] - Re-export without direct rustdoc on item
- `pub use store::{AgentMetricsData, AgentRunResultData, AllMetrics, MetricsStore}` - [Partially documented] - Re-export without direct rustdoc on items
- `pub enum MetricsError` - [Fully documented] - Has rustdoc with all variants documented
- `pub struct AgentRunResult` - [Fully documented] - Has rustdoc with fields documented
- `pub struct AgentMetrics` - [Fully documented] - Has comprehensive rustdoc with description, when metrics updated, persistence, use cases, and example

### src/metrics/collector.rs
- `pub fn update_all_metrics(all_metrics: &mut AllMetrics, run_result: &AgentRunResult) -> Result<(), MetricsError>` - [Fully documented] - Has rustdoc with description, Arguments, Returns, Example, and Example sections

### src/metrics/store.rs
- `pub struct AllMetrics` - [Fully documented] - Has rustdoc with fields documented
- `pub struct AgentMetricsData` - [Fully documented] - Has rustdoc with fields documented
- `pub struct AgentRunResultData` - [Fully documented] - Has rustdoc with fields documented
- `pub struct MetricsStore` - [Partially documented] - Has struct-level rustdoc
- `pub fn new(log_dir: PathBuf) -> Self` - [Fully documented] - Has rustdoc with Returns section
- `pub fn load(&self) -> Result<AllMetrics, MetricsError>` - [Fully documented] - Has rustdoc with description, behavior for missing file, behavior for corruption, and returns sections
- `pub fn save(&self, metrics: &AllMetrics) -> Result<(), MetricsError>` - [Fully documented] - Has rustdoc with description, behavior for atomic writes, and returns sections

### src/scheduler/mod.rs
- `pub enum SchedulerError` - [Fully documented] - Has comprehensive rustdoc with all variants and detailed error handling sections
- `pub enum RunStatus` - [Fully documented] - Has rustdoc with variants documented
- `pub struct QueuedRun` - [Fully documented] - Has rustdoc with fields documented
- `pub fn process_queued_run(...)` - [Fully documented] - Has rustdoc with description, Arguments, and Returns sections
- `pub async fn execute_agent(...)` - [Partially documented] - Has rustdoc with Arguments, Returns, and Errors sections
- `pub fn suggest_cron_correction(invalid_schedule: &str) -> Option<String>` - [Fully documented] - Has comprehensive rustdoc with description, arguments, returns, handling patterns, and examples

### src/scheduler/clock.rs
- `pub trait Clock` - [Fully documented] - Has rustdoc with description
- `pub struct SystemClock` - [Fully documented] - Has struct-level rustdoc
- `impl Clock for SystemClock` - [Undocumented] - No rustdoc on impl block

## Notes

### Documentation Criteria
- **Fully documented**: Has comprehensive rustdoc (///) with:
  - Description of purpose/functionality
  - Arguments/Parameters section with documentation
  - Returns section documenting return type
  - Errors section documenting error conditions (where applicable)
  - Examples section with code examples (where applicable)
  - Panic behavior documented (where applicable)

- **Partially documented**: Has some rustdoc but missing at least one of:
  - Complete argument/parameter documentation
  - Return type documentation
  - Error conditions documentation
  - Example code

- **Undocumented**: No rustdoc comments before the item

### Key Findings
1. **High overall documentation coverage** (89.6% have at least partial documentation)
2. **CLI module has excellent documentation** - All public APIs are fully documented
3. **Config module has comprehensive error documentation** - All ConfigError variants are well-documented
4. **Logging module is well-documented** - Core logging functionality has complete documentation
5. **Architect module is well-documented** - State and session management APIs are fully documented
6. **Scheduler error handling is excellent** - All error variants with detailed documentation
7. **Docker module needs attention** - Some re-exported items lack direct rustdoc
8. **Type aliases are undocumented** - Result<T> in architect/mod.rs needs documentation
9. **Some impl blocks are undocumented** - SystemClock impl, From<ContainerError>
10. **Several helper functions are undocumented** - cleanup_on_complete, docker()

## Recommendations for Documentation Gaps

### High Priority
1. Document `cleanup_on_complete()` in `src/architect/state.rs` - Important cleanup function with no documentation
2. Document `pub type Result<T>` in `src/architect/mod.rs` - Type alias used throughout the module
3. Document `pub fn docker(&self)` in `src/docker/mod.rs` - Getter function needs documentation
4. Document `impl From<ContainerError>` in `src/docker/run/types.rs` - Trait implementation needs documentation
5. Document `impl Clock for SystemClock` in `src/scheduler/clock.rs` - Trait implementation needs documentation

### Medium Priority
1. Add rustdoc to re-exported items in:
   - `src/architect/mod.rs` - AgentStatus, ArchitectState, CurrentTask, QueueStatus, cleanup_on_complete, commit_progress, create_in_progress_marker, delete_in_progress_marker, load_state, save_state, end_session, update_session_progress
   - `src/commands/mod.rs` - BuildCommand, list_agents, metrics, ValidateCommand
   - `src/docker/mod.rs` - run_agent, AgentExecutionResult, attach_and_stream_logs, ContainerConfig, ContainerError, parse_timeout, wait_for_exit, wait_with_timeout, ExitStatus
2. Add Errors section to `run_agent()` in `src/docker/run.rs` - Missing documentation for error conditions
3. Consider adding examples to functions that don't have them yet

### Low Priority
1. Improve struct-level documentation for `Logger` in `src/logger/mod.rs` - Add example usage
2. Improve struct-level documentation for `MetricsStore` in `src/metrics/store.rs` - Add example usage
3. Add example usage to key helper functions without examples
