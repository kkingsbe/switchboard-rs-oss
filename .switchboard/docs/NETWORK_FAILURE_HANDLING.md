# Network Failure Handling: Skills Management

This document defines the expected behavior, error handling, and testing approach for network failure scenarios in the switchboard skills management system.

## Overview

The skills management system is designed to handle network failures gracefully, providing clear error messages to users and maintaining system stability even when network resources are unavailable. Skills installation relies on npx and network access to remote repositories, so proper network failure handling is critical for a robust user experience.

**Key Design Principle:** The system should **never panic** or crash due to network failures. All network-related errors must be handled through Result types and propagated appropriately.

## Network Failure Behavior

### Graceful Degradation

When network connectivity is unavailable or unstable, the skills system exhibits the following behaviors:

| Operation | Network Available | Network Unavailable | Behavior |
|-----------|-------------------|---------------------|----------|
| **List local skills** | ✅ Works normally | ✅ Works normally | Local operations are unaffected by network |
| **Install skill via npx** | ✅ Completes successfully | ❌ Fails with clear error | Container exits with non-zero code |
| **Update skills** | ✅ Completes successfully | ❌ Fails with clear error | Error indicates network issue |
| **Check npx availability** | ✅ Returns `Ok(())` | ⚠️ May time out | Returns `SkillsError::NpxNotFound` or timeout |
| **Generate entrypoint script** | ✅ Completes successfully | ✅ Completes successfully | Script generation is offline |

### Expected Error Messages

The system provides clear, actionable error messages for network-related failures:

#### 1. Network Unavailable Error

When the network is completely unavailable:

```
Error: Network unavailable for install operation
Details: DNS resolution failed for skills.sh
```

**Example scenarios that trigger this error:**
- DNS resolution failures for skill repositories
- Complete network outage
- Firewall blocking outbound connections

#### 2. npx Command Failed Error

When npx execution fails due to network issues:

```
Error: npx command failed
Command: npx skills add owner/repo
Exit code: 1
Stderr: network unreachable
```

**Example scenarios that trigger this error:**
- Unable to fetch skill packages from npm registry
- Connection timeout during package download
- Repository unreachable (e.g., GitHub connection refused)

#### 3. Container Install Failed Error

When skill installation fails within the Docker container:

```
Error: Skill installation failed in container
Skill source: owner/repo
Agent name: test-agent
Exit code: 1
Stderr: fetch failed: unable to access 'https://github.com/owner/repo'
```

**Example scenarios that trigger this error:**
- Git clone fails due to network
- Repository authentication fails
- Repository no longer exists

#### 4. Skill Not Found Error

When a skill cannot be found or accessed:

```
Error: Skill not found: owner/repo
```

**Example scenarios that trigger this error:**
- Typo in skill specification
- Repository deleted or renamed
- Private repository without credentials

### Container Exit Codes

The generated entrypoint script ensures proper exit code propagation:

| Scenario | Exit Code | Behavior |
|----------|-----------|----------|
| **Successful installation** | 0 | Normal execution continues |
| **Network failure during install** | 1 | Container exits with failure |
| **npx not found** | 1 | Container exits with failure |
| **Invalid skill format** | 1 | Container exits with failure |
| **Timeout** | 124 (or custom) | Container exits with timeout code |

The shell script uses `set -e` to ensure any command failure results in immediate exit with a non-zero code. This failure is captured by the error handler and propagated to the container.

### Metrics Recording

The metrics system tracks network-related failures to help monitor system health:

#### Metrics Fields for Network Failures

| Metric Field | Type | Description |
|--------------|------|-------------|
| `exit_code` | Integer | Non-zero when installation fails |
| `skills_installed_count` | Integer | Number of skills successfully installed |
| `skills_failed_count` | Integer | Number of skills that failed (including network failures) |
| `skills_install_time_seconds` | Float | Time spent attempting installation |
| `total_runs` | Integer | Incremented for each run attempt |

#### Example Metrics After Network Failure

```json
{
  "agents": {
    "agent-with-network-issues": {
      "total_runs": 1,
      "total_skills_installed": 0,
      "last_run_exit_code": 1,
      "skills_install_time_seconds": 5.0
    }
  }
}
```

The metrics system persists failure data to disk, ensuring that historical failure data is not lost even when operations fail.

## Testing Network Failures

### Test File

[`tests/skills_network_failure.rs`](../tests/skills_network_failure.rs)

### Test Coverage

The test suite includes 13 tests covering various aspects of network failure handling:

| Test Name | Purpose | What It Verifies |
|-----------|---------|------------------|
| `test_result_types_used_for_fallible_operations` | Type safety | Operations that can fail return `Result` types, not panic |
| `test_error_messages_are_clear_and_actionable` | User experience | Error messages include command, exit code, and stderr |
| `test_metrics_can_record_network_failures` | Metrics | Failure metrics are correctly recorded and persisted |
| `test_script_exits_nonzero_on_skill_installation_failure` | Exit codes | Generated script exits with non-zero code on failure |
| `test_no_panic_on_network_failures` | Stability | No panics occur for any network failure scenario |
| `test_script_includes_detailed_error_reporting` | Debugging | Script logs detailed error information |
| `test_metrics_persistence_after_failures` | Data persistence | Metrics persist correctly after failures |
| `test_comprehensive_error_coverage` | Error types | `SkillsError` enum covers all network scenarios |
| `test_skill_format_validation_no_panic` | Input validation | Invalid skill formats return errors, not panic |

### Running Network Failure Tests

#### Prerequisites

Ensure you have:
- Rust toolchain installed
- The switchboard project cloned
- Write permissions to create temporary test directories

#### Running All Tests

```bash
cargo test --test skills_network_failure
```

#### Running Specific Tests

```bash
# Test result types
cargo test --test skills_network_failure test_result_types_used_for_fallible_operations -- --nocapture

# Test error messages
cargo test --test skills_network_failure test_error_messages_are_clear_and_actionable -- --nocapture

# Test metrics recording
cargo test --test skills_network_failure test_metrics_can_record_network_failures -- --nocapture

# Test script exit behavior
cargo test --test skills_network_failure test_script_exits_nonzero_on_skill_installation_failure -- --nocapture
```

#### Understanding Test Output

Tests output descriptive messages about what they verify:

```
running 13 tests
test skills_network_failure::test_result_types_used_for_fallible_operations ... ok
test skills_network_failure::test_error_messages_are_clear_and_actionable ... ok
test skills_network_failure::test_metrics_can_record_network_failures ... ok
...
```

All tests should pass if the error handling is implemented correctly.

### Testing Approach

The tests use mock implementations and controlled environments to simulate various network failure scenarios:

1. **Type System Verification**: Tests ensure that fallible operations return `Result` types, preventing panics

2. **Error Message Quality**: Tests verify that error messages include actionable information

3. **Metrics Validation**: Tests confirm that failure metrics are recorded and persisted correctly

4. **Script Behavior**: Tests validate that generated shell scripts exit with appropriate codes

5. **Error Coverage**: Tests verify that all network-related error scenarios are covered

Since actual network failures are difficult to reproduce reliably in CI environments, the tests focus on verifying that:
- Appropriate error handling paths are exercised
- User-facing error messages are clear and actionable
- The system degrades gracefully rather than crashing
- Local operations continue to work during outages

## Troubleshooting Network Issues

### Diagnosing Network Problems

When users encounter network-related errors, follow these steps:

#### Step 1: Identify the Error Type

Check the error message to determine the specific failure:

| Error Pattern | Likely Cause |
|---------------|--------------|
| "DNS resolution failed" | DNS server issues or incorrect hostname |
| "network unreachable" | No network connection or firewall blocking |
| "connection refused" | Remote service not running or wrong port |
| "ENOTFOUND" | Hostname cannot be resolved |
| "ETIMEDOUT" | Connection attempt timed out |
| "fetch failed: unable to access" | Git clone or HTTP fetch failed |

#### Step 2: Verify Network Connectivity

Check basic network connectivity:

```bash
# Check if you can reach common servers
ping -c 4 google.com

# Check DNS resolution
nslookup skills.sh

# Check if you can reach npm registry
curl -I https://registry.npmjs.org/

# Check if you can reach GitHub (for git-based skills)
curl -I https://github.com/
```

#### Step 3: Check npx Installation

Verify that npx is installed and accessible:

```bash
# Check npx version
npx --version

# Try a simple npx command
npx --help
```

If npx is not found, install it:

```bash
# Install npm (which includes npx)
# On Ubuntu/Debian
sudo apt-get install npm

# On macOS
brew install node

# Verify installation
npm --version
npx --version
```

#### Step 4: Check Docker and Network Configuration

Verify Docker can access the network:

```bash
# Check Docker daemon is running
docker ps

# Test Docker network connectivity
docker run --rm alpine ping -c 4 google.com

# Check Docker daemon logs
journalctl -u docker -n 50
```

If Docker containers cannot access the network:
- Check firewall rules for Docker
- Verify Docker network settings (`docker network ls`)
- Check proxy settings if using a corporate network

#### Step 5: Check Skill Repository Accessibility

Verify the skill repository is accessible:

```bash
# For git-based skills
git ls-remote https://github.com/owner/repo.git

# For npm-based skills
npm view <package-name>

# Test the specific npx command
npx skills add owner/repo
```

#### Step 6: Check Proxy Settings

If behind a corporate proxy, verify proxy configuration:

```bash
# Check environment variables
echo $HTTP_PROXY
echo $HTTPS_PROXY
echo $NO_PROXY

# Configure npm proxy
npm config set proxy http://proxy.example.com:8080
npm config set https-proxy http://proxy.example.com:8080

# Configure git proxy
git config --global http.proxy http://proxy.example.com:8080
git config --global https.proxy http://proxy.example.com:8080
```

#### Step 7: Check DNS Configuration

If DNS resolution fails:

```bash
# Check DNS servers
cat /etc/resolv.conf

# Test DNS resolution
dig skills.sh
nslookup skills.sh

# Try alternative DNS servers (e.g., Google DNS)
echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf
```

### Common Issues and Solutions

#### Issue 1: "npx: command not found"

**Symptom:** Error indicating npx cannot be found

**Solution:** Install Node.js and npm (which includes npx):

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install nodejs npm

# Verify installation
which npx
```

#### Issue 2: "DNS resolution failed"

**Symptom:** Unable to resolve skill repository hostnames

**Solution:**
1. Check internet connectivity
2. Verify DNS server settings in `/etc/resolv.conf`
3. Try using public DNS servers (8.8.8.8, 1.1.1.1)
4. Check firewall rules for DNS queries

#### Issue 3: "Connection timed out"

**Symptom:** Network operations timeout after waiting

**Solution:**
1. Check firewall rules blocking outbound connections
2. Verify proxy settings if required
3. Test connectivity to the specific host with `curl` or `wget`
4. Increase timeout values if network is slow

#### Issue 4: "fetch failed: unable to access"

**Symptom:** Git clone or HTTP fetch fails

**Solution:**
1. Verify the skill repository URL is correct
2. Check if the repository exists and is public
3. For private repositories, configure authentication
4. Check git proxy settings if behind a corporate firewall

#### Issue 5: "network unreachable"

**Symptom:** Complete network unavailability

**Solution:**
1. Check physical network connection
2. Restart network interfaces: `sudo systemctl restart networking`
3. Check router/gateway connectivity
4. Verify no VPN is interfering

### Using Metrics to Diagnose Issues

Review metrics to identify patterns of network failures:

```bash
# View metrics file
cat ~/.switchboard/metrics.json

# Or view in the configured metrics directory
cat <metrics_directory>/metrics.json
```

Look for:
- High `skills_failed_count` for specific agents
- Repeated exit code 1 for installation failures
- Long `skills_install_time_seconds` indicating slow networks or timeouts

### Getting Additional Help

If the above steps don't resolve the issue:

1. **Collect diagnostic information:**
   ```bash
   # System information
   uname -a

   # Network configuration
   ip addr show
   ip route show

   # Docker information
   docker info
   docker version

   # Node/npm information
   node --version
   npm --version
   npx --version
   ```

2. **Enable verbose logging:**
   - Run switchboard with debug flags if available
   - Check container logs: `docker logs <container_id>`
   - Review the entrypoint script output

3. **Report the issue:**
   - Include the full error message
   - Provide the diagnostic information above
   - Specify which operation failed (install, update, list)
   - Include the skill specification causing issues

## Implementation Guidelines

### DO

- Use `Result<T, SkillsError>` for all fallible operations
- Implement clear, actionable error messages
- Ensure the system never panics due to network failures
- Record network failures in metrics
- Use `set -e` in shell scripts to exit on error
- Validate input formats before attempting network operations
- Provide retry logic for transient failures
- Set reasonable timeouts for network operations

### DO NOT

- Panic on network failures
- Hide error details from users
- Attempt network operations without timeouts
- Ignore non-zero exit codes from subprocesses
- Assume network is always available
- Mix different types of errors in a single error message
- Remove error context when propagating errors

## Error Type Reference

### SkillsError Variants for Network Failures

| Variant | Fields | When Used |
|---------|--------|-----------|
| `NetworkUnavailable` | `operation: String`, `message: String` | Network is unavailable for an operation |
| `NpxCommandFailed` | `command: String`, `exit_code: i32`, `stderr: String` | npx command execution fails |
| `ContainerInstallFailed` | `skill_source: String`, `agent_name: String`, `exit_code: i32`, `stderr: String` | Installation fails in container |
| `SkillNotFound` | `skill_source: String` | Skill cannot be found or accessed |

All error variants implement `Display` for user-friendly messages and `Debug` for debugging.

## References

- Test file: [`tests/skills_network_failure.rs`](../tests/skills_network_failure.rs)
- Error types: [`src/skills.rs`](../src/skills.rs) (SkillsError enum)
- Metrics implementation: [`src/metrics.rs`](../src/metrics.rs)
- Entrypoint script generation: [`src/docker/skills.rs`](../src/docker/skills.rs)
- Skills documentation: [`docs/skills-feature.md`](../docs/skills-feature.md)
- Related performance docs: [`docs/PERFORMANCE_SKILLS_LIST.md`](./PERFORMANCE_SKILLS_LIST.md), [`docs/PERFORMANCE_SKILLS_INSTALL.md`](./PERFORMANCE_SKILLS_INSTALL.md)

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-20 | Initial documentation of network failure handling |

---

**Last Updated:** 2026-02-20  
**Maintainer:** Development Team  
**Related Sprint:** Sprint 4, Task 3
