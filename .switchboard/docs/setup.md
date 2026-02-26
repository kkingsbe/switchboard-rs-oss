# Switchboard Setup Guide

This guide provides comprehensive instructions for configuring Switchboard, including directory structure setup and API key management.

## Prerequisites

Before proceeding with the setup, ensure you have Switchboard installed. See the [README.md](../README.md) for installation instructions.

---

## Section 1: .kilocode Directory Setup Guide

The `.kilocode` directory is where Switchboard stores all configuration files, agent prompts, and logs. This directory is typically located in your home directory (`~/.kilocode` on Unix-like systems).

### Directory Structure

The typical `.kilocode` directory structure looks like this:

```
~/.kilocode/
├── config.toml          # Main configuration file
├── agents/              # Agent prompt files
│   ├── daily-report.md
│   └── code-review.md
└── logs/                # Log files
```

### Manual Directory Creation

If you need to create this directory structure manually, follow these steps:

1. **Create the base directory:**
   ```bash
   mkdir -p ~/.kilocode
   ```

2. **Create the agents directory:**
   ```bash
   mkdir -p ~/.kilocode/agents
   ```

3. **Create the logs directory:**
   ```bash
   mkdir -p ~/.kilocode/logs
   ```

4. **Copy or create a configuration file:**
   - You can copy the sample configuration from the project:
     ```bash
     cp switchboard.sample.toml ~/.kilocode/config.toml
     ```
   - Or create a new one based on your requirements

### Permissions and Ownership

Ensure that you have the appropriate permissions for the `.kilocode` directory:

- **Ownership**: The directory should be owned by your user account
- **Permissions**: Read, write, and execute permissions (typically `755` or `700`)
- **Security**: For added security, use `700` permissions to restrict access to only the owner

To verify or set permissions:

```bash
# Check current permissions
ls -la ~/.kilocode

# Set secure permissions (owner only)
chmod 700 ~/.kilocode

# Set standard permissions (owner can read/write/execute, others can read/execute)
chmod 755 ~/.kilocode
```

### Sample Configurations

The `examples/` directory in the Switchboard project contains sample configuration files that you can reference or use as templates:

- [`examples/basic.toml`](../examples/basic.toml) - A minimal configuration to get started
- [`examples/advanced.toml`](../examples/advanced.toml) - A comprehensive configuration showcasing all available options

Review these examples to understand the available configuration options and customize `config.toml` to suit your needs.

### Automatic Directory Creation

Switchboard will automatically create the `.kilocode` directory structure on first run if it doesn't exist. However, you may prefer to set it up manually to:

- Control the exact directory structure
- Pre-populate configuration files
- Set up specific agent prompts before running Switchboard
- Ensure proper permissions and ownership

### Troubleshooting

If you encounter issues with the `.kilocode` directory setup:

- **Permission errors**: Ensure you have write permissions to your home directory and can create directories
- **Configuration not found**: Verify that `config.toml` exists in `~/.kilocode/`
- **Path resolution issues**: Check that your home directory is correctly set (typically `$HOME` environment variable)
- **Ownership issues**: On some systems, you may need to use `sudo` if the directory was created by another user

For more detailed installation and setup troubleshooting, see [docs/INSTALLATION_TROUBLESHOOTING.md](./INSTALLATION_TROUBLESHOOTING.md).

---

## Section 2: Where to Place API Keys

Switchboard leverages Kilocode, which requires API keys to access various AI models. These keys authenticate your requests to model providers such as OpenAI, Anthropic, and others. Without valid API keys, Switchboard cannot communicate with AI models and will not function properly.

### API Key Location

API keys should be stored securely in one of two locations:

#### Option 1: `.env` File (Recommended)

Create a `.env` file in the `.kilocode` directory:

```bash
~/.kilocode/.env
```

Example `.env` file format:

```bash
# AI Model API Keys
OPENAI_API_KEY=sk-your-openai-key-here
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key-here

# Other API Keys (as needed)
GITHUB_TOKEN=ghp-your-github-token-here
GOOGLE_API_KEY=your-google-api-key-here
COHERE_API_KEY=your-cohere-api-key-here
```

#### Option 2: System Environment Variables

You can also set API keys as system environment variables. This approach is useful for:

- Docker containers
- CI/CD pipelines
- Production deployments

**Linux/macOS (Bash/Zsh):**
```bash
export OPENAI_API_KEY="sk-your-key"
export ANTHROPIC_API_KEY="sk-ant-your-key"
export GITHUB_TOKEN="ghp-your-token"
```

To make these persist across sessions, add them to your shell configuration file (`.bashrc`, `.zshrc`, etc.):

```bash
echo 'export OPENAI_API_KEY="sk-your-key"' >> ~/.bashrc
echo 'export ANTHROPIC_API_KEY="sk-ant-your-key"' >> ~/.bashrc
source ~/.bashrc
```

**Windows PowerShell:**
```powershell
$env:OPENAI_API_KEY="sk-your-key"
$env:ANTHROPIC_API_KEY="sk-ant-your-key"
```

To make these persist, set them as system environment variables using the System Properties dialog or use:

```powershell
[System.Environment]::SetEnvironmentVariable('OPENAI_API_KEY', 'sk-your-key', 'User')
[System.Environment]::SetEnvironmentVariable('ANTHROPIC_API_KEY', 'sk-ant-your-key', 'User')
```

### Common API Keys

Here are the common API keys you may need to configure for Switchboard:

#### AI Model Providers

| Provider | Environment Variable | Key Format | Documentation |
|----------|---------------------|------------|---------------|
| OpenAI | `OPENAI_API_KEY` | `sk-...` | https://platform.openai.com/api-keys |
| Anthropic (Claude) | `ANTHROPIC_API_KEY` | `sk-ant-...` | https://console.anthropic.com/settings/keys |
| Google AI (Gemini) | `GOOGLE_API_KEY` | `AIza...` | https://makersuite.google.com/app/apikey |
| Cohere | `COHERE_API_KEY` | `...` | https://dashboard.cohere.com/api-keys |
| Replicate | `REPLICATE_API_TOKEN` | `r8_...` | https://replicate.com/account/api-tokens |
| Mistral AI | `MISTRAL_API_KEY` | `...` | https://console.mistral.ai/api-keys |

#### Additional Services

| Service | Environment Variable | Purpose |
|---------|---------------------|---------|
| GitHub | `GITHUB_TOKEN` | GitHub API access for MCP servers |
| Database | `DATABASE_URL` | Database connection strings |
| Custom | `VAR_NAME` | Any custom variables for MCP servers |

### API Key File Format Examples

#### OpenAI API Key

```bash
# In ~/.kilocode/.env
OPENAI_API_KEY=sk-proj-abc123def456ghi789jkl012mno345pqr678stu901vwx234yz
```

#### Anthropic API Key

```bash
# In ~/.kilocode/.env
ANTHROPIC_API_KEY=sk-ant-api03-abc123def456ghi789jkl012mno345pqr678stu901vwx234yz
```

#### Multiple Keys

```bash
# In ~/.kilocode/.env
# OpenAI
OPENAI_API_KEY=sk-proj-your-openai-key-here

# Anthropic
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key-here

# GitHub (for MCP servers)
GITHUB_TOKEN=ghp_your-github-token-here

# Google AI
GOOGLE_API_KEY=AIzaSyYourGoogleAPIKeyHere

# Database (example)
DATABASE_URL=postgresql://user:password@localhost:5432/mydb
```

### Security Best Practices

Follow these security practices to protect your API keys:

- **Never commit `.env` files to version control** - Add `.env` to your `.gitignore` file
  ```bash
  # In .gitignore
  .env
  *.env
  ```

- **Use environment-specific keys** - Maintain separate keys for development and production environments

- **Rotate keys regularly** - Change your API keys periodically to reduce the impact of potential leaks

- **Use key management services in production** - For production deployments, consider using:
  - AWS Secrets Manager
  - HashiCorp Vault
  - Azure Key Vault
  - Google Cloud Secret Manager

- **Limit key permissions** - Only grant the minimum necessary permissions to each API key

- **Monitor usage** - Regularly review API key usage logs for suspicious activity

- **Keep `.env` file permissions restricted** - Set permissions to `600` (read/write for owner only)
  ```bash
  chmod 600 ~/.kilocode/.env
  ```

- **Use key rotation policies** - Many providers offer automated key rotation or expiration

- **Document key sources** - Keep track of where each API key is used for easier management

> **Warning**: API keys are sensitive credentials. Exposing them can lead to unauthorized access and billing charges. Always follow security best practices when managing API keys.

### Obtaining API Keys

Different AI model providers have different requirements for obtaining API keys:

#### OpenAI

1. Visit [https://platform.openai.com/api-keys](https://platform.openai.com/api-keys)
2. Create an OpenAI account if you don't have one
3. Navigate to the API keys section
4. Click "Create new secret key"
5. Copy the key immediately (you won't be able to see it again)
6. Keys start with `sk-`

#### Anthropic (Claude)

1. Visit [https://console.anthropic.com/settings/keys](https://console.anthropic.com/settings/keys)
2. Create an Anthropic account if you don't have one
3. Navigate to API Keys in settings
4. Click "Create Key"
5. Give the key a descriptive name
6. Copy and store the key securely
7. Keys start with `sk-ant-`

#### GitHub Token

1. Visit [https://github.com/settings/tokens](https://github.com/settings/tokens)
2. Click "Generate new token" (classic) or "Generate new token" (fine-grained)
3. Select the required scopes/permissions
4. Generate and copy the token
5. Tokens start with `ghp_` (classic) or `github_pat_` (fine-grained)

#### Other Providers

Consult the documentation for your chosen model provider to learn how to obtain API keys. Common providers include:

- **Google AI (Gemini)**: [Google AI Studio](https://makersuite.google.com/app/apikey)
- **Cohere**: [Cohere Dashboard](https://dashboard.cohere.com/api-keys)
- **Replicate**: [Replicate Account Settings](https://replicate.com/account/api-tokens)
- **Mistral AI**: [Mistral Console](https://console.mistral.ai/api-keys)

**Note**: Different models may require different key formats and naming conventions. Always refer to the specific model provider's documentation for the correct key format and required environment variable name.

---

## Model Configuration Examples

Model configuration is essential for controlling how AI agents interact with language models. This includes setting the model provider, specifying which model variant to use, and fine-tuning parameters like temperature and token limits. Proper model configuration ensures optimal performance and cost control for your automated tasks.

### Basic Model Configuration

For simple use cases, you can configure model settings directly in your configuration file or pass them as environment variables to the agent container. Here's a basic example:

```toml
[[agent]]
name = "simple-task"
schedule = "0 0 9 * * *"
prompt = "Review the code changes in the last 24 hours."

# Pass model configuration as environment variables
env = {
  # Model provider
  MODEL_PROVIDER = "openai",
  
  # Specific model to use
  MODEL_NAME = "gpt-4",
  
  # Temperature for response randomness (0.0 to 2.0)
  # Lower values = more deterministic, higher values = more creative
  MODEL_TEMPERATURE = "0.7",
  
  # Maximum tokens for model response
  # Controls the length of generated output
  MODEL_MAX_TOKENS = "2048"
}
```

**Field Explanations:**
- **MODEL_PROVIDER**: The AI service provider (e.g., `openai`, `anthropic`, `google`)
- **MODEL_NAME**: Specific model variant (e.g., `gpt-4`, `claude-3-opus`, `gemini-pro`)
- **MODEL_TEMPERATURE**: Controls randomness in responses
  - `0.0` - Deterministic, consistent responses
  - `0.5-0.7` - Balanced creativity and consistency (recommended for most tasks)
  - `1.0+` - More creative, varied responses
- **MODEL_MAX_TOKENS**: Maximum length of the model's output in tokens

### Advanced Model Configuration

For complex scenarios requiring multiple models, fallback configurations, or provider-specific settings, you can use advanced configuration patterns:

```toml
[[agent]]
name = "code-reviewer"
schedule = "0 0 */4 * * *"
prompt = """
Perform a comprehensive code review of the latest changes.
Focus on security vulnerabilities, performance issues, and code quality.
"""

# Primary model configuration
env = {
  # Primary provider
  MODEL_PROVIDER = "anthropic",
  MODEL_NAME = "claude-3-opus-20240229",
  
  # Fine-tuned parameters for code review
  MODEL_TEMPERATURE = "0.2",      # Lower temperature for more consistent analysis
  MODEL_MAX_TOKENS = "4096",       # Allow longer, detailed reports
  
  # Fallback configuration
  FALLBACK_PROVIDER = "openai",
  FALLBACK_MODEL = "gpt-4-turbo",
  FALLBACK_TEMPERATURE = "0.2",
  FALLBACK_MAX_TOKENS = "4096",
  
  # Rate limiting and retry settings
  MAX_RETRIES = "3",
  RETRY_DELAY = "5",               # Seconds between retries
  TIMEOUT = "120",                 # Request timeout in seconds
  
  # Provider-specific settings
  ANTHROPIC_MAX_CONTEXT = "200000",
  ANTHROPIC_TOP_P = "0.9",         # Nucleus sampling parameter
}

[[agent]]
name = "creative-writer"
schedule = "0 0 10 * * 1"
prompt = "Generate a weekly status report for stakeholders."

# Different configuration for creative tasks
env = {
  MODEL_PROVIDER = "openai",
  MODEL_NAME = "gpt-4-turbo",
  MODEL_TEMPERATURE = "0.9",      # Higher for creative tasks
  MODEL_MAX_TOKENS = "3072",
  
  # OpenAI-specific parameters
  OPENAI_TOP_P = "0.95",
  OPENAI_FREQUENCY_PENALTY = "0.1",  # Reduce repetition
  OPENAI_PRESENCE_PENALTY = "0.1",    # Encourage new topics
  
  # Streaming configuration
  STREAM_RESPONSE = "true",
}

[[agent]]
name = "summarizer"
schedule = "0 30 * * * *"
prompt = "Summarize the recent activity logs."

# Cost-effective configuration for frequent runs
env = {
  MODEL_PROVIDER = "anthropic",
  MODEL_NAME = "claude-3-haiku-20240307",  # Smaller, faster, cheaper
  MODEL_TEMPERATURE = "0.5",
  MODEL_MAX_TOKENS = "1024",
  
  # Performance optimization
  ENABLE_CACHING = "true",
  BATCH_SIZE = "10",
}
```

**Advanced Settings Explained:**
- **FALLBACK_* settings**: Alternative model configuration to use if the primary model fails
- **MAX_RETRIES**: Number of retry attempts for failed requests
- **RETRY_DELAY**: Wait time between retries in seconds
- **TIMEOUT**: Maximum time to wait for a model response
- **Provider-specific parameters**: Additional settings like `TOP_P`, `FREQUENCY_PENALTY`, `PRESENCE_PENALTY`
- **STREAM_RESPONSE**: Enable streaming responses for faster feedback
- **ENABLE_CACHING**: Cache responses for repeated requests

### Model Provider Examples

Different AI providers have unique configuration requirements and model variants. Here are examples for common providers:

#### OpenAI Configuration

```toml
[[agent]]
name = "openai-agent"
schedule = "0 0 * * * *"
prompt = "Analyze system metrics and generate insights."

env = {
  # OpenAI-specific configuration
  MODEL_PROVIDER = "openai",
  MODEL_NAME = "gpt-4-turbo-preview",
  MODEL_TEMPERATURE = "0.7",
  MODEL_MAX_TOKENS = "2048",
  
  # OpenAI parameters
  OPENAI_API_BASE = "https://api.openai.com/v1",
  OPENAI_ORGANIZATION = "org-your-id",
  OPENAI_TOP_P = "0.9",
  OPENAI_FREQUENCY_PENALTY = "0.0",
  OPENAI_PRESENCE_PENALTY = "0.0",
}
```

**OpenAI Model Variants:**
- `gpt-4-turbo-preview` - Latest GPT-4 model with 128K context
- `gpt-4` - Original GPT-4 model with 8K context
- `gpt-3.5-turbo` - Fast, cost-effective model
- `gpt-4-32k` - GPT-4 with 32K context window

#### Anthropic (Claude) Configuration

```toml
[[agent]]
name = "anthropic-agent"
schedule = "0 15 * * * *"
prompt = "Perform security analysis of the codebase."

env = {
  # Anthropic-specific configuration
  MODEL_PROVIDER = "anthropic",
  MODEL_NAME = "claude-3-opus-20240229",
  MODEL_TEMPERATURE = "0.3",
  MODEL_MAX_TOKENS = "4096",
  
  # Anthropic parameters
  ANTHROPIC_API_BASE = "https://api.anthropic.com",
  ANTHROPIC_VERSION = "2023-06-01",
  ANTHROPIC_MAX_CONTEXT = "200000",
  ANTHROPIC_TOP_P = "0.95",
  ANTHROPIC_TOP_K = "0",
}
```

**Claude Model Variants:**
- `claude-3-opus-20240229` - Most capable, 200K context
- `claude-3-sonnet-20240229` - Balanced performance and speed, 200K context
- `claude-3-haiku-20240307` - Fast, cost-effective, 200K context

#### Google AI (Gemini) Configuration

```toml
[[agent]]
name = "gemini-agent"
schedule = "0 0 */6 * * *"
prompt = "Generate documentation for API endpoints."

env = {
  # Google AI-specific configuration
  MODEL_PROVIDER = "google",
  MODEL_NAME = "gemini-pro",
  MODEL_TEMPERATURE = "0.8",
  MODEL_MAX_TOKENS = "2048",
  
  # Google AI parameters
  GOOGLE_API_BASE = "https://generativelanguage.googleapis.com/v1",
  GOOGLE_TOP_P = "0.9",
  GOOGLE_TOP_K = "40",
}
```

**Gemini Model Variants:**
- `gemini-pro` - General-purpose model
- `gemini-pro-vision` - Multimodal model (text and images)
- `gemini-ultra` - Most capable model (when available)

#### Cohere Configuration

```toml
[[agent]]
name = "cohere-agent"
schedule = "0 0 12 * * *"
prompt = "Draft marketing copy for new features."

env = {
  # Cohere-specific configuration
  MODEL_PROVIDER = "cohere",
  MODEL_NAME = "command",
  MODEL_TEMPERATURE = "0.75",
  MODEL_MAX_TOKENS = "2048",
  
  # Cohere parameters
  COHERE_API_BASE = "https://api.cohere.ai/v1",
  COHERE_TOP_P = "0.75",
  COHERE_TOP_K = "0",
  COHERE_PRESENCE_PENALTY = "0.0",
  COHERE_FREQUENCY_PENALTY = "0.0",
}
```

**Cohere Model Variants:**
- `command` - General-purpose text generation
- `command-light` - Faster, cost-effective model
- `command-r` - Latest model with improved capabilities

### Environment Variable Override

Environment variables provide a flexible way to override model settings without modifying configuration files. This is particularly useful for:
- Testing different models temporarily
- Managing secrets securely
- Adapting to different deployment environments
- A/B testing model configurations

#### Override Mechanism

Environment variables take precedence over values defined in the configuration file. The priority order is:

1. **Environment Variables** (highest priority)
2. **Agent-level `env` settings** in config file
3. **Global settings** (if applicable)

#### Common Environment Variables

```bash
# Primary model configuration
export MODEL_PROVIDER="openai"
export MODEL_NAME="gpt-4-turbo-preview"
export MODEL_TEMPERATURE="0.7"
export MODEL_MAX_TOKENS="2048"

# Fallback model configuration
export FALLBACK_PROVIDER="anthropic"
export FALLBACK_MODEL="claude-3-sonnet-20240229"

# API endpoints (for custom or on-premise deployments)
export OPENAI_API_BASE="https://custom-api.example.com/v1"
export ANTHROPIC_API_BASE="https://custom-anthropic.example.com"

# Rate limiting and retry settings
export MAX_RETRIES="5"
export RETRY_DELAY="10"
export TIMEOUT="180"

# Provider-specific settings
export OPENAI_ORGANIZATION="org-your-id"
export ANTHROPIC_VERSION="2023-06-01"

# Feature flags
export STREAM_RESPONSE="true"
export ENABLE_CACHING="true"
```

#### Setting Environment Variables

**Temporary (session-based):**
```bash
# Set variables for current session
export MODEL_PROVIDER="anthropic"
export MODEL_NAME="claude-3-opus-20240229"

# Run switchboard with the override
switchboard start
```

**Persistent (shell configuration):**
```bash
# Add to ~/.bashrc or ~/.zshrc
echo 'export MODEL_PROVIDER="openai"' >> ~/.bashrc
echo 'export MODEL_NAME="gpt-4-turbo-preview"' >> ~/.bashrc
source ~/.bashrc
```

**Docker containers:**
```bash
# Pass environment variables to Docker containers
docker run -e MODEL_PROVIDER="anthropic" \
           -e MODEL_NAME="claude-3-opus-20240229" \
           -e ANTHROPIC_API_KEY="your-key" \
           switchboard-agent
```

**In configuration file with environment variable references:**
```toml
[[agent]]
name = "configurable-agent"
schedule = "0 0 * * * *"
prompt = "Process data using the configured model."

# Default values that can be overridden by environment variables
env = {
  MODEL_PROVIDER = "${MODEL_PROVIDER:-openai}",      # Fallback to "openai" if not set
  MODEL_NAME = "${MODEL_NAME:-gpt-4}",               # Fallback to "gpt-4" if not set
  MODEL_TEMPERATURE = "${MODEL_TEMPERATURE:-0.7}",   # Fallback to 0.7 if not set
  MODEL_MAX_TOKENS = "${MODEL_MAX_TOKENS:-2048}",    # Fallback to 2048 if not set
}
```

#### Example: Production vs Development Configuration

**Development:**
```bash
# Use faster, cheaper models for development
export MODEL_NAME="gpt-3.5-turbo"
export MODEL_TEMPERATURE="0.8"
export MODEL_MAX_TOKENS="1024"
```

**Production:**
```bash
# Use more capable models for production
export MODEL_NAME="gpt-4-turbo-preview"
export MODEL_TEMPERATURE="0.5"
export MODEL_MAX_TOKENS="4096"
```

**Testing:**
```bash
# Use a different provider for testing
export MODEL_PROVIDER="anthropic"
export MODEL_NAME="claude-3-haiku-20240307"
export MAX_RETRIES="10"
export RETRY_DELAY="2"
```

> **Note**: When using environment variable overrides, ensure your `.env` file or shell configuration includes the API key for the provider you're using. The model provider and API key must match for successful API calls.

---

## MCP Server Definitions

MCP (Model Context Protocol) servers extend the capabilities of AI agents by providing access to external tools and resources. These servers act as bridges between agents and various services, enabling agents to interact with file systems, databases, web APIs, version control systems, and more. By configuring MCP servers, you can empower your agents to perform complex tasks like reading and writing files, querying databases, making HTTP requests, and managing GitHub repositories.

### What is MCP?

The Model Context Protocol (MCP) is a standardized protocol that defines how AI agents communicate with external services and tools. MCP servers run as separate processes that expose a set of tools through a well-defined interface. When an agent needs to perform an action that requires external access, it can invoke tools provided by MCP servers.

**Key Concepts:**

- **MCP Server**: A process that implements the MCP protocol and exposes tools to agents
- **Tools**: Individual capabilities provided by an MCP server (e.g., "read_file", "write_file", "query_database")
- **Protocol**: A standardized communication format that agents and servers use to exchange requests and responses

**Common MCP Server Capabilities:**

- **File System Operations**: Reading, writing, listing, and deleting files
- **Database Access**: Querying and modifying databases (PostgreSQL, MySQL, SQLite, etc.)
- **Version Control**: Interacting with Git repositories (creating branches, committing, pushing)
- **Web Services**: Making HTTP requests to REST APIs and web services
- **Cloud Services**: Interacting with AWS, GCP, Azure, and other cloud platforms
- **Custom Tools**: User-defined capabilities specific to your workflow

### Configuring MCP Servers

MCP servers are defined in the Kilo Code CLI configuration file, typically located at `~/.kilocode/config.toml`. Each MCP server is configured with parameters that specify how to start the server, what command to run, and any required environment variables.

**Basic MCP Server Configuration Structure:**

```toml
[[mcp_servers]]
name = "server-name"                    # Unique identifier for the server
type = "server-type"                     # Type of MCP server (filesystem, github, postgresql, etc.)
command = "command-to-run"              # Command to start the server
args = ["arg1", "arg2"]                 # Arguments to pass to the command
env = {                                  # Environment variables for the server
  "VAR_NAME" = "value",
  "ANOTHER_VAR" = "another-value"
}
```

**Configuration Fields Explained:**

- **name** (required): A unique identifier for this MCP server. Used to reference the server in agent configurations.
- **type** (required): The type of MCP server. Common types include `filesystem`, `github`, `postgresql`, `web`, and `custom`.
- **command** (required): The executable command to start the MCP server process.
- **args** (optional): An array of command-line arguments to pass to the server command.
- **env** (optional): A key-value mapping of environment variables to set for the server process.

**Multiple MCP Servers Example:**

```toml
# File system MCP server
[[mcp_servers]]
name = "filesystem"
type = "filesystem"
command = "mcp-filesystem-server"
args = ["/path/to/workspace"]
env = {
  "ALLOW_WRITE" = "true"
}

# GitHub MCP server
[[mcp_servers]]
name = "github"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "ghp_your_github_token_here"
}

# PostgreSQL MCP server
[[mcp_servers]]
name = "database"
type = "postgresql"
command = "mcp-postgresql-server"
args = ["--connection-string", "postgresql://user:password@localhost:5432/mydb"]
env = {}
```

### Common MCP Server Types

This section provides configuration examples for common MCP server types. Each example shows a complete TOML configuration with all necessary parameters.

#### File System Server

The file system MCP server provides agents with tools to read, write, list, and manipulate files on the local or mounted file system.

```toml
[[mcp_servers]]
name = "filesystem"
type = "filesystem"
command = "mcp-filesystem-server"
args = ["/path/to/workspace"]
env = {
  # Allow write operations (set to "false" for read-only access)
  "ALLOW_WRITE" = "true",
  
  # Allowed directories (comma-separated, empty means all directories)
  "ALLOWED_DIRECTORIES" = "/path/to/workspace,/tmp/safe",
  
  # Denied directories (comma-separated, takes precedence over allowed)
  "DENIED_DIRECTORIES" = "/etc,/root,/var/log",
  
  # Maximum file size for read operations (in bytes, 0 = unlimited)
  "MAX_FILE_SIZE" = "10485760",
  
  # Maximum number of files to return in directory listings
  "MAX_LISTINGS" = "1000"
}
```

**Available Tools:**

- `read_file(path)`: Read the contents of a file
- `write_file(path, content)`: Write content to a file
- `list_directory(path)`: List files and directories in a path
- `delete_file(path)`: Delete a file
- `create_directory(path)`: Create a new directory
- `file_exists(path)`: Check if a file or directory exists

#### GitHub Server

The GitHub MCP server enables agents to interact with GitHub repositories, including creating issues, pull requests, managing branches, and accessing repository information.

```toml
[[mcp_servers]]
name = "github"
type = "github"
command = "mcp-github-server"
args = []
env = {
  # GitHub personal access token (required)
  "GITHUB_TOKEN" = "ghp_your_github_token_here",
  
  # Default repository (can be overridden in tool calls)
  "DEFAULT_REPO" = "owner/repo-name",
  
  # GitHub Enterprise API URL (if using GitHub Enterprise)
  # "GITHUB_API_URL" = "https://github.yourcompany.com/api/v3",
  
  # Number of items to return per page
  "PER_PAGE" = "30"
}
```

**Available Tools:**

- `create_issue(title, body, labels)`: Create a new GitHub issue
- `get_issue(issue_number)`: Get details of an issue
- `list_issues(state, labels)`: List issues in the repository
- `create_branch(branch_name, base_branch)`: Create a new branch
- `delete_branch(branch_name)`: Delete a branch
- `create_pull_request(title, body, head, base)`: Create a pull request
- `get_pull_request(pr_number)`: Get details of a pull request
- `list_pull_requests(state)`: List pull requests
- `get_file_contents(path, ref)`: Get file contents from the repository
- `update_file(path, content, message)`: Update a file in the repository

> **Note**: The GitHub token must have appropriate scopes for the operations you want to perform. Common scopes include `repo` (full repository access), `issues` (issue access), and `pull_requests` (pull request access).

#### PostgreSQL Server

The PostgreSQL MCP server allows agents to query and interact with PostgreSQL databases. This is useful for agents that need to analyze data, generate reports, or perform database operations.

```toml
[[mcp_servers]]
name = "database"
type = "postgresql"
command = "mcp-postgresql-server"
args = ["--connection-string", "postgresql://user:password@localhost:5432/mydb"]
env = {
  # Alternatively, you can pass the connection string via environment variable
  # "DATABASE_URL" = "postgresql://user:password@localhost:5432/mydb",
  
  # Enable read-only mode (set to "true" for safety)
  "READ_ONLY" = "false",
  
  # Maximum number of rows to return from SELECT queries
  "MAX_ROWS" = "1000",
  
  # Query timeout in seconds (0 = no timeout)
  "QUERY_TIMEOUT" = "30",
  
  # Allow schema modifications (true/false)
  "ALLOW_SCHEMA_MODIFICATIONS" = "false"
}
```

**Available Tools:**

- `execute_query(sql)`: Execute a SQL query and return results
- `list_tables()`: List all tables in the database
- `describe_table(table_name)`: Get schema information for a table
- `get_table_row_count(table_name)`: Get the number of rows in a table

> **Warning**: Be careful when giving agents write access to databases. Consider using read-only mode for production databases or creating a dedicated read-only user account.

#### Web Server

The web MCP server enables agents to make HTTP requests to web services and APIs. This is useful for fetching data from external services, interacting with REST APIs, and performing web scraping.

```toml
[[mcp_servers]]
name = "web"
type = "web"
command = "mcp-web-server"
args = []
env = {
  # Base URL for API requests (optional, can be overridden per request)
  "BASE_URL" = "https://api.example.com/v1",
  
  # Default timeout for HTTP requests (in seconds)
  "TIMEOUT" = "30",
  
  # Maximum number of redirects to follow
  "MAX_REDIRECTS" = "5",
  
  # User agent header
  "USER_AGENT" = "MCP-Web-Server/1.0",
  
  # Default headers (JSON format)
  "DEFAULT_HEADERS" = "{\"Accept\": \"application/json\", \"Content-Type\": \"application/json\"}",
  
  # Allowed hosts (comma-separated, empty means all hosts)
  "ALLOWED_HOSTS" = "api.example.com,api.another.com",
  
  # Blocked hosts (comma-separated)
  "BLOCKED_HOSTS" = "localhost,127.0.0.1"
}
```

**Available Tools:**

- `get(url, headers, params)`: Perform an HTTP GET request
- `post(url, headers, body)`: Perform an HTTP POST request
- `put(url, headers, body)`: Perform an HTTP PUT request
- `delete(url, headers)`: Perform an HTTP DELETE request
- `patch(url, headers, body)`: Perform an HTTP PATCH request
- `head(url, headers)`: Perform an HTTP HEAD request

#### Custom MCP Servers

You can create and configure custom MCP servers for specialized tools or workflows. Custom servers can be written in any language that supports the MCP protocol.

```toml
[[mcp_servers]]
name = "custom-tools"
type = "custom"
command = "/path/to/your/custom-mcp-server"
args = ["--config", "/path/to/server-config.json"]
env = {
  # Custom server configuration
  "SERVER_MODE" = "production",
  "LOG_LEVEL" = "info",
  
  # API keys for external services
  "SERVICE_API_KEY" = "your-service-api-key",
  
  # Server-specific settings
  "MAX_CONCURRENT_REQUESTS" = "10"
}
```

**Creating a Custom MCP Server:**

Custom MCP servers can be created by implementing the MCP protocol in your preferred programming language. The protocol defines:

- **Startup**: Server announces available tools and capabilities
- **Request Handling**: Server processes tool requests from agents
- **Response**: Server returns results in a standardized format

For more information on implementing custom MCP servers, refer to the MCP protocol documentation.

### MCP Server Environment Variables

Environment variables are the recommended way to pass sensitive information like API keys, database credentials, and tokens to MCP servers. This approach keeps secrets out of configuration files and allows for easy rotation of credentials.

#### Passing Environment Variables

There are three ways to pass environment variables to MCP servers:

**1. Via the `env` section in server configuration:**

```toml
[[mcp_servers]]
name = "github"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "ghp_your_github_token_here"
}
```

**2. Via system environment variables (variables are inherited by the MCP server):**

```bash
# Set environment variable in your shell
export GITHUB_TOKEN="ghp_your_github_token_here"

# The MCP server will inherit this variable
```

```toml
[[mcp_servers]]
name = "github"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "${GITHUB_TOKEN}"  # Reference system environment variable
}
```

**3. Via the `.kilocode/.env` file:**

```bash
# In ~/.kilocode/.env
GITHUB_TOKEN=ghp_your_github_token_here
DATABASE_URL=postgresql://user:password@localhost:5432/mydb
```

#### Environment Variable Usage Examples

**Multiple API Keys:**

```toml
[[mcp_servers]]
name = "github"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "ghp_your_github_token_here"
}

[[mcp_servers]]
name = "slack"
type = "custom"
command = "mcp-slack-server"
args = []
env = {
  "SLACK_API_TOKEN" = "xoxb-your-slack-token",
  "SLACK_SIGNING_SECRET" = "your-slack-signing-secret"
}
```

**Database Credentials:**

```toml
[[mcp_servers]]
name = "database"
type = "postgresql"
command = "mcp-postgresql-server"
args = ["--connection-string", "${DATABASE_URL}"]
env = {
  "DATABASE_URL" = "postgresql://user:${DB_PASSWORD}@localhost:5432/mydb"
}
```

#### Security Best Practices

Follow these security practices when managing MCP server credentials:

- **Never hardcode secrets in configuration files**: Always use environment variables for API keys, tokens, and passwords.

- **Use separate credentials for MCP servers**: Create dedicated API keys and user accounts for MCP servers, rather than reusing credentials with broader permissions.

- **Implement the principle of least privilege**: Grant only the minimum necessary permissions to MCP server credentials. For example:
  - Use read-only database accounts when possible
  - Limit GitHub token scopes to only what's needed
  - Set expiration dates on API keys when supported

- **Rotate credentials regularly**: Change API keys and passwords periodically to reduce the impact of potential leaks.

- **Use secrets management in production**: For production deployments, use a secrets management service:
  - AWS Secrets Manager
  - HashiCorp Vault
  - Azure Key Vault
  - Google Cloud Secret Manager

- **Secure the `.env` file**: Set restrictive file permissions on environment files:
  ```bash
  chmod 600 ~/.kilocode/.env  # Read/write for owner only
  ```

- **Don't commit secrets to version control**: Add `.env` files to `.gitignore`:
  ```bash
  # In .gitignore
  .env
  *.env
  .kilocode/.env
  ```

- **Audit credential usage**: Regularly review API key usage logs to identify any suspicious activity.

- **Use different credentials for different environments**: Maintain separate API keys for development, staging, and production environments.

> **Warning**: Exposed API keys can lead to unauthorized access, data breaches, and unexpected charges. Always follow security best practices when managing credentials for MCP servers.

### MCP Server Permissions

MCP server permissions control which tools agents can access and what actions they can perform. Proper permission configuration helps ensure that agents can only perform actions that are appropriate for their use case, enhancing security and preventing accidental or malicious operations.

#### Tool Access Control

You can control which tools an agent can access by configuring the `allowed_tools` field in the agent configuration. This restricts the agent to only use the specified tools from available MCP servers.

```toml
[[agent]]
name = "code-reader"
schedule = "0 0 * * * *"
prompt = "Review the code and provide insights."

# Allow only read operations from the filesystem MCP server
env = {
  "MCP_ALLOWED_TOOLS" = "filesystem:read_file,filesystem:list_directory,filesystem:file_exists"
}

[[agent]]
name = "code-writer"
schedule = "0 30 * * * *"
prompt = "Update documentation files."

# Allow both read and write operations from the filesystem MCP server
env = {
  "MCP_ALLOWED_TOOLS" = "filesystem:read_file,filesystem:write_file,filesystem:list_directory,filesystem:file_exists"
}

[[agent]]
name = "database-analyst"
schedule = "0 0 9 * * *"
prompt = "Analyze database metrics and generate reports."

# Allow database query operations only
env = {
  "MCP_ALLOWED_TOOLS" = "database:execute_query,database:list_tables,database:describe_table"
}
```

#### Permission Scoping

Permission scoping allows you to restrict the scope of operations an agent can perform. This is particularly useful for limiting agents to specific directories, repositories, or database tables.

**Directory-Based Scoping (File System MCP Server):**

```toml
[[mcp_servers]]
name = "filesystem-docs"
type = "filesystem"
command = "mcp-filesystem-server"
args = ["/path/to/docs"]
env = {
  "ALLOWED_DIRECTORIES" = "/path/to/docs",
  "ALLOW_WRITE" = "false"
}

[[mcp_servers]]
name = "filesystem-temp"
type = "filesystem"
command = "mcp-filesystem-server"
args = ["/tmp/switchboard"]
env = {
  "ALLOWED_DIRECTORIES" = "/tmp/switchboard",
  "ALLOW_WRITE" = "true"
}
```

```toml
# Agent restricted to documentation directory
[[agent]]
name = "doc-reviewer"
schedule = "0 0 * * * *"
prompt = "Review documentation."
env = {
  "MCP_SERVERS" = "filesystem-docs"
}

# Agent restricted to temporary directory with write access
[[agent]]
name = "temp-worker"
schedule = "0 */30 * * * *"
prompt = "Process files in temporary directory."
env = {
  "MCP_SERVERS" = "filesystem-temp"
}
```

**Repository-Based Scoping (GitHub MCP Server):**

```toml
[[mcp_servers]]
name = "github-public"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "ghp_readonly_token",
  "ALLOWED_REPOS" = "owner/repo1,owner/repo2",
  "ALLOW_SCOPES" = "read"
}

[[mcp_servers]]
name = "github-private"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "ghp_full_access_token",
  "ALLOWED_REPOS" = "owner/private-repo",
  "ALLOW_SCOPES" = "read,write"
}
```

**Query-Based Scoping (Database MCP Server):**

```toml
[[mcp_servers]]
name = "database-readonly"
type = "postgresql"
command = "mcp-postgresql-server"
args = ["--connection-string", "postgresql://readonly:password@localhost:5432/mydb"]
env = {
  "READ_ONLY" = "true",
  "ALLOWED_TABLES" = "public.users,public.analytics,public.reports"
}

[[mcp_servers]]
name = "database-write"
type = "postgresql"
command = "mcp-postgresql-server"
args = ["--connection-string", "postgresql://writer:password@localhost:5432/mydb"]
env = {
  "READ_ONLY" = "false",
  "ALLOWED_TABLES" = "public.users,public.analytics"
}
```

#### Examples of Restricted Access

**Read-Only Documentation Agent:**

```toml
[[mcp_servers]]
name = "filesystem-docs"
type = "filesystem"
command = "mcp-filesystem-server"
args = ["/path/to/docs"]
env = {
  "ALLOWED_DIRECTORIES" = "/path/to/docs",
  "DENIED_DIRECTORIES" = "/path/to/docs/internal,/path/to/docs/secrets",
  "ALLOW_WRITE" = "false"
}

[[agent]]
name = "doc-reviewer"
schedule = "0 0 * * * *"
prompt = "Review public documentation and suggest improvements."
env = {
  "MCP_SERVERS" = "filesystem-docs",
  "MCP_ALLOWED_TOOLS" = "filesystem:read_file,filesystem:list_directory"
}
```

**Database Query Agent (Read-Only):**

```toml
[[mcp_servers]]
name = "database-analytics"
type = "postgresql"
command = "mcp-postgresql-server"
args = ["--connection-string", "${ANALYTICS_DB_URL}"]
env = {
  "READ_ONLY" = "true",
  "ALLOWED_TABLES" = "public.analytics,public.reports",
  "MAX_ROWS" = "10000"
}

[[agent]]
name = "analytics-reporter"
schedule = "0 0 9 * * *"
prompt = "Generate daily analytics reports."
env = {
  "MCP_SERVERS" = "database-analytics",
  "MCP_ALLOWED_TOOLS" = "database:execute_query,database:describe_table"
}
```

**GitHub Issue Manager (Issue-Only Access):**

```toml
[[mcp_servers]]
name = "github-issues"
type = "github"
command = "mcp-github-server"
args = []
env = {
  "GITHUB_TOKEN" = "${GITHUB_ISSUE_TOKEN}",
  "DEFAULT_REPO" = "owner/repo",
  "ALLOWED_OPERATIONS" = "issues:list,issues:get,issues:create"
}

[[agent]]
name = "issue-triager"
schedule = "0 */2 * * * *"
prompt = "Triage new GitHub issues and assign labels."
env = {
  "MCP_SERVERS" = "github-issues",
  "MCP_ALLOWED_TOOLS" = "github:list_issues,github:get_issue,github:create_issue"
}
```

> **Note**: Always configure the most restrictive permissions that still allow the agent to perform its intended task. This principle of least privilege helps prevent accidental data loss, unauthorized access, and security vulnerabilities.

---

## Common Setup Issues and Solutions

This section covers common problems users encounter during setup and how to resolve them. Each issue includes the error you might see, the likely cause, and step-by-step solutions to fix it.

### .kilocode Directory Issues

#### Issue: ".kilocode directory not found" error

**Error:**
> Error: .kilocode directory not found

**Cause:** The `.kilocode` directory doesn't exist in your home directory or is in the wrong location.

**Solution:**

1. **Verify the expected location:**
   ```bash
   # Check if directory exists
   ls -la ~/.kilocode
   ```

2. **Create the directory if it doesn't exist:**
   ```bash
   mkdir -p ~/.kilocode
   mkdir -p ~/.kilocode/agents
   mkdir -p ~/.kilocode/logs
   ```

3. **Verify the creation:**
   ```bash
   ls -la ~/.kilocode
   ```

4. **If you previously created it in a different location:**
   - Move it to the correct location:
     ```bash
     mv /old/path/.kilocode ~/.kilocode
     ```
   - Or update your configuration to point to the custom location

#### Issue: "Permission denied" when accessing .kilocode

**Error:**
> Error: Permission denied: cannot access ~/.kilocode

**Cause:** The directory has incorrect ownership or permissions settings.

**Solution:**

1. **Check current ownership and permissions:**
   ```bash
   ls -la ~/.kilocode
   ```

2. **Fix ownership (if owned by another user):**
   ```bash
   # Change ownership to your user
   sudo chown -R $USER:$USER ~/.kilocode
   ```

3. **Fix permissions:**
   ```bash
   # Set secure permissions (owner only)
   chmod 700 ~/.kilocode
   
   # Or set standard permissions
   chmod 755 ~/.kilocode
   
   # Fix files inside
   chmod -R 600 ~/.kilocode/.env  # if it exists
   chmod -R 644 ~/.kilocode/config.toml  # if it exists
   ```

4. **Verify the fix:**
   ```bash
   ls -la ~/.kilocode
   touch ~/.kilocode/test.txt && rm ~/.kilocode/test.txt
   ```

### API Key Issues

#### Issue: "API key not found" or "invalid API key" errors

**Error:**
> Error: API key not found: OPENAI_API_KEY
> or
> Error: Invalid API key

**Cause:** API key file not found, wrong format, or expired key.

**Solution:**

1. **Verify API key file exists:**
   ```bash
   ls -la ~/.kilocode/.env
   ```

2. **Check API key file format:**
   ```bash
   cat ~/.kilocode/.env
   ```
   
   Ensure it follows this format:
   ```bash
   OPENAI_API_KEY=sk-proj-your-key-here
   ANTHROPIC_API_KEY=sk-ant-your-key-here
   ```

3. **Verify API key validity:**
   - Check that the key starts with the correct prefix:
     - OpenAI: `sk-`
     - Anthropic: `sk-ant-`
   - Ensure no extra spaces or quotes around the key
   - Verify the key hasn't expired in your provider's dashboard

4. **Create or fix the .env file:**
   ```bash
   # Create the file if it doesn't exist
   touch ~/.kilocode/.env
   
   # Add your API key (replace with your actual key)
   echo 'OPENAI_API_KEY=sk-proj-your-key-here' >> ~/.kilocode/.env
   
   # Set secure permissions
   chmod 600 ~/.kilocode/.env
   ```

5. **Test that the key is being loaded:**
   ```bash
   # Check if environment variable is set
   echo $OPENAI_API_KEY
   
   # Or load from .env file
   export $(grep -v '^#' ~/.kilocode/.env | xargs)
   echo $OPENAI_API_KEY
   ```

#### Issue: "Rate limit exceeded" errors

**Error:**
> Error: Rate limit exceeded: too many requests

**Cause:** Too many requests to the API provider within a short time period.

**Solution:**

1. **Implement rate limiting in your configuration:**
   ```toml
   [[agent]]
   name = "rate-limited-agent"
   schedule = "0 0 * * * *"  # Adjust to less frequent schedule
   prompt = "Your prompt here"
   
   env = {
     # Add rate limiting settings
     MAX_RETRIES = "3",
     RETRY_DELAY = "10",        # Wait longer between retries
     REQUEST_TIMEOUT = "120",   # Increase timeout
   }
   ```

2. **Reduce agent frequency:**
   - Change cron schedules to run less frequently
   - Spread out agent execution times

3. **Use a different model tier:**
   - Some providers have different rate limits for different models
   - Consider using models with higher rate limits for your use case

4. **Wait before retrying:**
   - Most rate limits reset after a period (e.g., 1 minute, 1 hour)
   - Check your provider's documentation for rate limit details

5. **Implement exponential backoff:**
   ```bash
   # Example: Wait with exponential backoff
   for i in {1..3}; do
     # try your command
     switchboard run --agent your-agent && break
     sleep $((10 * i))  # wait 10s, 20s, 30s
   done
   ```

#### Issue: API keys not being loaded from environment variables

**Error:**
> Error: API key not found despite being set in environment

**Cause:** Environment variables not set, incorrect syntax, or not exported to child processes.

**Solution:**

1. **Verify environment variables are set:**
   ```bash
   # Check if variable is set
   echo $OPENAI_API_KEY
   
   # List all environment variables
   env | grep API
   ```

2. **Set environment variables correctly:**
   
   **Linux/macOS (Bash/Zsh):**
   ```bash
   # Set for current session
   export OPENAI_API_KEY="sk-your-key-here"
   export ANTHROPIC_API_KEY="sk-ant-your-key-here"
   
   # Make persistent across sessions (add to ~/.bashrc or ~/.zshrc)
   echo 'export OPENAI_API_KEY="sk-your-key-here"' >> ~/.bashrc
   echo 'export ANTHROPIC_API_KEY="sk-ant-your-key-here"' >> ~/.bashrc
   source ~/.bashrc
   ```
   
   **Windows PowerShell:**
   ```powershell
   # Set for current session
   $env:OPENAI_API_KEY="sk-your-key-here"
   
   # Set permanently (User scope)
   [System.Environment]::SetEnvironmentVariable('OPENAI_API_KEY', 'sk-your-key-here', 'User')
   ```

3. **Verify .env file is being loaded:**
   ```bash
   # Check file exists and has content
   cat ~/.kilocode/.env
   
   # Ensure no syntax errors (no quotes around values, correct spacing)
   # Correct: OPENAI_API_KEY=sk-key
   # Wrong: OPENAI_API_KEY = "sk-key"  (spaces and quotes can cause issues)
   ```

4. **Test with a simple script:**
   ```bash
   # Create a test script
   cat > test_env.sh << 'EOF'
   #!/bin/bash
   source ~/.kilocode/.env
   echo "OPENAI_API_KEY is: ${OPENAI_API_KEY:0:10}..."
   EOF
   
   chmod +x test_env.sh
   ./test_env.sh
   ```

5. **Restart your terminal or shell:**
   - Sometimes environment variables don't take effect until you restart
   - Close and reopen your terminal application

### Docker Issues

#### Issue: "Docker daemon not running" error

**Error:**
> Error: Cannot connect to Docker daemon. Is the docker daemon running?

**Cause:** Docker service is not started or not installed.

**Solution:**

1. **Check if Docker is installed:**
   ```bash
   docker --version
   ```

2. **Check Docker daemon status:**
   ```bash
   # Linux
   sudo systemctl status docker
   
   # macOS
   # Check Docker Desktop is running in menu bar
   
   # Windows
   # Check Docker Desktop is running in system tray
   ```

3. **Start Docker daemon:**
   
   **Linux (systemd):**
   ```bash
   sudo systemctl start docker
   sudo systemctl enable docker  # Auto-start on boot
   ```
   
   **Linux (init.d):**
   ```bash
   sudo service docker start
   sudo chkconfig docker on  # Auto-start on boot
   ```
   
   **macOS:**
   - Open Docker Desktop from Applications
   - Wait until Docker icon shows "Docker Desktop is running"
   
   **Windows:**
   - Open Docker Desktop from Start menu
   - Wait until Docker icon shows "Docker Desktop is running"

4. **Verify Docker is running:**
   ```bash
   docker ps
   ```
   This should return a list of running containers (or an empty list if none are running).

5. **If Docker is not installed:**
   - Linux: Follow installation guide at https://docs.docker.com/engine/install/
   - macOS: Download Docker Desktop from https://www.docker.com/products/docker-desktop/
   - Windows: Download Docker Desktop from https://www.docker.com/products/docker-desktop/

#### Issue: "Permission denied while trying to connect to Docker daemon"

**Error:**
> Error: Got permission denied while trying to connect to the Docker daemon socket

**Cause:** Your user is not in the docker group.

**Solution:**

1. **Check if docker group exists:**
   ```bash
   getent group docker
   ```

2. **Check your current groups:**
   ```bash
   groups
   ```

3. **Add your user to the docker group:**
   ```bash
   sudo usermod -aG docker $USER
   ```

4. **Activate the new group membership:**
   
   **Option A: Log out and log back in**
   
   **Option B: Use newgrp (temporary for current session):**
   ```bash
   newgrp docker
   ```
   
   **Option C: Refresh groups in current shell:**
   ```bash
   exec sg docker newgrp $(id -gn)
   ```

5. **Verify the fix:**
   ```bash
   # Check that docker group is in your groups
   groups | grep docker
   
   # Try running docker without sudo
   docker ps
   ```

6. **If issues persist:**
   ```bash
   # Check docker socket permissions
   ls -la /var/run/docker.sock
   
   # Should show docker:docker ownership (or root:docker)
   # If not, fix it:
   sudo chown root:docker /var/run/docker.sock
   sudo chmod 660 /var/run/docker.sock
   ```

#### Issue: Container image build failures

**Error:**
> Error: Failed to build container image

**Cause:** Network issues, missing dependencies, syntax errors, or insufficient disk space.

**Solution:**

1. **Check the detailed error message:**
   ```bash
   # Rebuild with verbose output
   switchboard build --agent your-agent --verbose
   ```

2. **Common build issues and fixes:**

   **Network connectivity issues:**
   ```bash
   # Check internet connection
   ping -c 3 google.com
   
   # Check DNS resolution
   nslookup registry-1.docker.io
   
   # Try pulling a test image
   docker pull alpine:latest
   ```

   **Missing dependencies:**
   - Verify all required packages are listed in your Dockerfile
   - Update package lists in Dockerfile: `RUN apt-get update`
   
   **Syntax errors in Dockerfile or configuration:**
   ```bash
   # Validate your TOML configuration
   switchboard validate config
   
   # Check for syntax errors in Dockerfile (if custom)
   # Common issues: missing quotes, incorrect line continuation
   ```

   **Insufficient disk space:**
   ```bash
   # Check disk space
   df -h
   
   # Clean up unused Docker resources
   docker system prune -a
   
   # Clean up build cache
   docker builder prune
   ```

3. **Force a clean rebuild:**
   ```bash
   # Remove existing images for your agent
   docker images | grep your-agent-name
   docker rmi your-image-id
   
   # Rebuild
   switchboard build --agent your-agent --force
   ```

4. **Check Docker daemon logs:**
   ```bash
   # Linux
   sudo journalctl -u docker.service -n 50
   
   # macOS
   # Check Docker Desktop logs via menu bar > Troubleshoot > Logs
   
   # Windows
   # Check Docker Desktop logs via system tray > Troubleshoot > Logs
   ```

#### Issue: "Error: workspace path does not exist"

**Error:**
> Error: workspace path does not exist: /path/to/workspace

**Cause:** Incorrect workspace_path configuration or the path doesn't exist on the system.

**Solution:**

1. **Verify the configured workspace path:**
   ```bash
   # Check your switchboard.toml configuration
   cat ~/.kilocode/config.toml | grep workspace_path
   ```

2. **Check if the path actually exists:**
   ```bash
   # Using the path from your configuration
   ls -la /path/to/workspace
   ```

3. **Create the directory if it doesn't exist:**
   ```bash
   # Create the workspace directory
   mkdir -p /path/to/workspace
   
   # Verify it was created
   ls -la /path/to/workspace
   ```

4. **Check permissions:**
   ```bash
   # Verify you have read/write access
   touch /path/to/workspace/test.txt && rm /path/to/workspace/test.txt
   ```

5. **Use an absolute path instead of relative:**
   
   **Incorrect (relative path):**
   ```toml
   workspace_path = "./my-workspace"
   ```
   
   **Correct (absolute path):**
   ```toml
   workspace_path = "/home/username/my-workspace"
   # Or use environment variable
   workspace_path = "${HOME}/my-workspace"
   ```

6. **Update your configuration with the correct path:**
   ```bash
   # Edit your config file
   nano ~/.kilocode/config.toml
   
   # Or use a command to update it
   sed -i 's|workspace_path = "./my-workspace"|workspace_path = "/home/username/my-workspace"|g' ~/.kilocode/config.toml
   ```

7. **Validate the configuration:**
   ```bash
   switchboard validate config
   ```

### Configuration File Issues

#### Issue: "Failed to parse configuration file" error

**Error:**
> Error: Failed to parse configuration file: invalid TOML syntax

**Cause:** Invalid TOML syntax, missing required fields, or malformed data structures.

**Solution:**

1. **Validate TOML syntax:**
   ```bash
   # Use Switchboard's built-in validation
   switchboard validate config
   
   # Or use a TOML linter if available
   # Example with taplo (if installed):
   taplo check ~/.kilocode/config.toml
   ```

2. **Common TOML syntax errors and fixes:**
   
   **Mismatched brackets:**
   ```toml
   # Wrong
   [[agent]
   name = "test"
   
   # Correct
   [[agent]]
   name = "test"
   ```
   
   **Unquoted strings with special characters:**
   ```toml
   # Wrong
   prompt = Hello world!
   
   # Correct (multiline string for long text)
   prompt = """
   Hello world!
   """
   
   # Or (single line with quotes)
   prompt = "Hello world!"
   ```
   
   **Invalid boolean or number values:**
   ```toml
   # Wrong
   enabled = true
   timeout = "30"  # Numbers should not be quoted
   
   # Correct
   enabled = true
   timeout = 30
   ```
   
   **Missing commas in arrays:**
   ```toml
   # Wrong
   args = ["--port" "8080"]
   
   # Correct
   args = ["--port", "8080"]
   ```

3. **Check for required fields:**
   ```bash
   # Compare with sample configuration
   diff ~/.kilocode/config.toml switchboard.sample.toml
   ```

4. **Use a TOML validator online:**
   - https://www.toml-lint.com/
   - https://play.toml-lang.org/
   
   Paste your configuration to identify syntax errors.

5. **Check for invisible characters:**
   ```bash
   # Show all characters including whitespace
   cat -A ~/.kilocode/config.toml | head -20
   
   # Remove carriage returns (common on Windows)
   sed -i 's/\r$//' ~/.kilocode/config.toml
   ```

6. **Rebuild from sample if corrupted:**
   ```bash
   # Backup current config
   cp ~/.kilocode/config.toml ~/.kilocode/config.toml.backup
   
   # Copy sample
   cp switchboard.sample.toml ~/.kilocode/config.toml
   
   # Edit with your settings
   nano ~/.kilocode/config.toml
   ```

#### Issue: "Invalid cron expression" error

**Error:**
> Error: Invalid cron expression: syntax error

**Cause:** Malformed cron schedule syntax, incorrect number of fields, or invalid values.

**Solution:**

1. **Understand cron expression format:**
   
   Switchboard uses 6-field cron format (seconds, minutes, hours, day of month, month, day of week):
   
   ```
   ┌───────────── second (0 - 59)
   │ ┌───────────── minute (0 - 59)
   │ │ ┌───────────── hour (0 - 23)
   │ │ │ ┌───────────── day of month (1 - 31)
   │ │ │ │ ┌───────────── month (1 - 12)
   │ │ │ │ │ ┌───────────── day of week (0 - 7, where 0 and 7 are Sunday)
   │ │ │ │ │ │
   │ │ │ │ │ │
   * * * * * *
   ```

2. **Common cron expressions and what they mean:**
   
   ```toml
   # Run every minute
   schedule = "0 * * * * *"
   
   # Run every hour at minute 0
   schedule = "0 0 * * * *"
   
   # Run every day at 9:00 AM
   schedule = "0 0 9 * * *"
   
   # Run every Monday at 9:00 AM
   schedule = "0 0 9 * * 1"
   
   # Run every 5 minutes
   schedule = "0 */5 * * * *"
   
   # Run at midnight on the 1st of every month
   schedule = "0 0 0 1 * *"
   ```

3. **Common mistakes and fixes:**
   
   **Too few or too many fields:**
   ```toml
   # Wrong (5 fields - traditional cron without seconds)
   schedule = "0 * * * *"
   
   # Correct (6 fields - including seconds)
   schedule = "0 * * * * *"
   ```
   
   **Invalid values:**
   ```toml
   # Wrong
   schedule = "0 0 25 * * *"  # Hour 25 doesn't exist
   schedule = "0 0 0 32 * *"  # Day 32 doesn't exist
   
   # Correct
   schedule = "0 0 0 1 * *"   # First day of month
   ```
   
   **Incorrect syntax for ranges/lists:**
   ```toml
   # Wrong
   schedule = "0 0 9,10 * * *"  # Spaces around commas
   
   # Correct
   schedule = "0 0 9-10 * * *"  # Use hyphen for range
   schedule = "0 0 9,10 * * *"  # No spaces for list
   ```

4. **Validate cron expressions:**
   ```bash
   # Use Switchboard validation
   switchboard validate config
   
   # Use online cron expression validator
   # https://crontab.guru/ (note: only has 5 fields)
   # https://crontab.cronhub.io/ (supports 6 fields)
   ```

5. **Test cron expressions:**
   ```bash
   # Calculate next run times using a tool
   # Example with cronexpr (if installed):
   cronexpr next "0 0 9 * * 1"
   ```

#### Issue: Config validation passes but agent doesn't run

**Cause:** Prompt file not found, missing environment variables, or permission issues.

**Solution:**

1. **Check agent status and logs:**
   ```bash
   # List all agents and their status
   switchboard list
   
   # View logs for a specific agent
   switchboard logs --agent your-agent-name
   ```

2. **Verify prompt file exists:**
   ```bash
   # Check if the prompt file referenced in config exists
   ls -la ~/.kilocode/agents/your-prompt-file.md
   
   # If using inline prompt, verify it's properly formatted
   cat ~/.kilocode/config.toml | grep -A 5 "prompt ="
   ```

3. **Check required environment variables:**
   ```bash
   # List environment variables required by agent
   cat ~/.kilocode/config.toml | grep -A 10 "env = {"
   
   # Verify they are set
   echo $OPENAI_API_KEY
   echo $ANTHROPIC_API_KEY
   echo $GITHUB_TOKEN
   ```

4. **Check agent configuration:**
   ```bash
   # Ensure agent is enabled
   cat ~/.kilocode/config.toml | grep -A 5 "enabled"
   
   # Should show:
   # enabled = true
   ```

5. **Verify MCP servers (if configured):**
   ```bash
   # Check if MCP servers are listed in config
   cat ~/.kilocode/config.toml | grep "MCP_SERVERS"
   
   # Verify MCP server commands are executable
   which mcp-filesystem-server
   ```

6. **Check file permissions:**
   ```bash
   # Check config file permissions
   ls -la ~/.kilocode/config.toml
   
   # Should be readable by your user
   # If not, fix:
   chmod 644 ~/.kilocode/config.toml
   
   # Check prompt file permissions
   ls -la ~/.kilocode/agents/
   chmod 644 ~/.kilocode/agents/*.md
   ```

7. **Try running the agent manually:**
   ```bash
   # Run agent once to test
   switchboard run --agent your-agent-name --once
   
   # Check output for errors
   ```

8. **Verify cron schedule:**
   ```bash
   # Calculate when the next run should be
   switchboard list --agent your-agent-name
   
   # Check if the schedule is correct for your needs
   ```

**Checklist for agent not running:**
- [ ] Agent is enabled in configuration
- [ ] Prompt file exists at specified path
- [ ] Cron expression is valid and reasonable
- [ ] Required environment variables are set
- [ ] MCP server commands (if any) are installed
- [ ] Docker is running and accessible
- [ ] Workspace path exists and is accessible
- [ ] Configuration file is valid TOML
- [ ] API keys are valid and not expired

### MCP Server Issues

#### Issue: "MCP server failed to start" error

**Error:**
> Error: MCP server 'mcp-filesystem' failed to start

**Cause:** Incorrect command, missing dependencies, or port conflicts.

**Solution:**

1. **Check MCP server configuration:**
   ```bash
   # View MCP server configuration
   cat ~/.kilocode/config.toml | grep -A 10 "\[\[mcp_servers\]\]"
   ```

2. **Verify the MCP server is installed:**
   ```bash
   # Check if the command exists
   which mcp-filesystem-server
   which mcp-postgresql-server
   
   # If not found, install it
   npm install -g @modelcontextprotocol/server-filesystem
   # or
   pip install mcp-filesystem-server
   ```

3. **Test the MCP server command manually:**
   ```bash
   # Run the command directly to see errors
   mcp-filesystem-server /path/to/directory
   ```

4. **Check for port conflicts:**
   ```bash
   # List what's using ports
   netstat -tuln | grep LISTEN
   
   # Check specific port (if using custom port)
   netstat -tuln | grep 3000
   
   # Kill process using the port if needed
   lsof -ti:3000 | xargs kill -9
   ```

5. **Check MCP server logs:**
   ```bash
   # Switchboard logs MCP server activity
   switchboard logs --agent your-agent-name | grep MCP
   ```

6. **Verify MCP server arguments:**
   ```toml
   # Ensure arguments are correct
   [[mcp_servers]]
   name = "filesystem"
   type = "filesystem"
   command = "mcp-filesystem-server"
   args = ["/path/to/workspace"]  # Check this path exists
   ```

7. **Test with a simple MCP server:**
   ```bash
   # Create a test filesystem server configuration
   [[mcp_servers]]
   name = "test-filesystem"
   type = "filesystem"
   command = "mcp-filesystem-server"
   args = ["/tmp"]
   
   # Validate and test
   switchboard validate config
   ```

#### Issue: "MCP server not accessible" or "connection refused"

**Error:**
> Error: MCP server not accessible: connection refused
> or
> Error: Failed to connect to MCP server: Connection refused

**Cause:** Server not running, wrong port, or firewall blocking the connection.

**Solution:**

1. **Check if MCP server is running:**
   ```bash
   # Check for MCP server processes
   ps aux | grep mcp
   
   # Check if the specific MCP server is running
   ps aux | grep mcp-filesystem-server
   ```

2. **Test MCP server connectivity:**
   ```bash
   # Try connecting directly (if using HTTP)
   curl http://localhost:3000/health
   
   # Or test the command
   mcp-filesystem-server --help
   ```

3. **Check firewall settings:**
   ```bash
   # Check if firewall is enabled
   sudo ufw status  # Ubuntu/Debian
   sudo firewall-cmd --list-all  # RHEL/CentOS
   
   # If firewall is blocking, allow the port
   sudo ufw allow 3000/tcp  # Ubuntu/Debian
   sudo firewall-cmd --add-port=3000/tcp --permanent  # RHEL/CentOS
   ```

4. **Verify MCP server is started by Switchboard:**
   ```bash
   # Check Switchboard logs for MCP server startup
   switchboard logs --agent your-agent-name | grep "MCP server"
   
   # Restart Switchboard to ensure MCP servers are initialized
   switchboard stop
   switchboard start
   ```

5. **Check MCP server configuration in agent:**
   ```toml
   # Ensure agent references correct MCP server
   [[agent]]
   name = "my-agent"
   schedule = "0 0 * * * *"
   prompt = "Your prompt here"
   
   env = {
     # Must match the name in [[mcp_servers]] section
     MCP_SERVERS = "filesystem,database"
   }
   ```

6. **Test with a local connection:**
   ```bash
   # Create a simple test script
   cat > test_mcp.sh << 'EOF'
   #!/bin/bash
   echo "Testing MCP server connection..."
   mcp-filesystem-server /tmp &
   SERVER_PID=$!
   sleep 2
   
   # Test if process is running
   if ps -p $SERVER_PID > /dev/null; then
     echo "MCP server started successfully (PID: $SERVER_PID)"
     kill $SERVER_PID
   else
     echo "MCP server failed to start"
   fi
   EOF
   
   chmod +x test_mcp.sh
   ./test_mcp.sh
   ```

#### Issue: MCP server tools not available to agent

**Error:**
> Error: MCP tools not available for agent 'my-agent'

**Cause:** Server not listed in agent configuration or permission denied.

**Solution:**

1. **Verify MCP server is listed in agent configuration:**
   ```bash
   # Check agent's MCP_SERVERS environment variable
   cat ~/.kilocode/config.toml | grep -A 20 "\[\[agent\]\]" | grep MCP_SERVERS
   ```

2. **Ensure MCP server is configured globally:**
   ```bash
   # Check if MCP server is defined in [[mcp_servers]] section
   cat ~/.kilocode/config.toml | grep -A 10 "\[\[mcp_servers\]\]"
   ```

3. **Match MCP server names:**
   ```toml
   # Ensure names match exactly
   [[mcp_servers]]
   name = "filesystem"  # This name
   type = "filesystem"
   command = "mcp-filesystem-server"
   args = ["/path/to/workspace"]
   
   [[agent]]
   name = "my-agent"
   schedule = "0 0 * * * *"
   prompt = "Your prompt here"
   
   env = {
     # Must match the MCP server name above
     MCP_SERVERS = "filesystem"  # References "filesystem" above
   }
   ```

4. **Check MCP tool permissions:**
   ```toml
   # Some MCP servers support tool-level restrictions
   [[agent]]
   name = "my-agent"
   schedule = "0 0 * * * *"
   prompt = "Your prompt here"
   
   env = {
     MCP_SERVERS = "filesystem",
     # Allow specific tools only
     MCP_ALLOWED_TOOLS = "filesystem:read_file,filesystem:list_directory"
   }
   ```

5. **Verify MCP server is providing tools:**
   ```bash
   # Check Switchboard logs for tool discovery
   switchboard logs --agent your-agent-name | grep "MCP tool"
   
   # Should see messages like:
   # Discovered MCP tools: read_file, write_file, list_directory
   ```

6. **Test with minimal MCP configuration:**
   ```toml
   # Start with a simple MCP server
   [[mcp_servers]]
   name = "simple-filesystem"
   type = "filesystem"
   command = "mcp-filesystem-server"
   args = ["/tmp"]
   
   [[agent]]
   name = "test-agent"
   schedule = "0 0 * * * *"
   prompt = "Test prompt"
   
   env = {
     MCP_SERVERS = "simple-filesystem"
   }
   ```

7. **Check MCP server capabilities:**
   ```bash
   # Some MCP servers may not support tools
   # Check the server documentation for supported features
   
   # Test the server directly
   mcp-filesystem-server --help
   ```

### Getting Help

If you encounter issues not covered in this section, here are additional resources to help you resolve them:

#### Documentation Resources

- **README.md** - General troubleshooting section with common issues
  - [README.md](../README.md) - Check the troubleshooting section near the end

- **docs/troubleshooting.md** - Detailed troubleshooting guide
  - [docs/troubleshooting.md](./troubleshooting.md) - Comprehensive troubleshooting guide

- **docs/INSTALLATION_TROUBLESHOOTING.md** - Installation-specific issues
  - [docs/INSTALLATION_TROUBLESHOOTING.md](./INSTALLATION_TROUBLESHOOTING.md) - Installation and setup troubleshooting

#### Community Support

- **GitHub Issues** - Search for existing issues or create a new one
  - Check if someone else has encountered the same issue
  - Search using keywords from your error message
  - When creating an issue, include:
    - Switchboard version
    - Operating system and version
    - Error messages (full output)
    - Configuration details (sanitize sensitive data)

#### Collecting Diagnostic Information

When seeking help, collect the following information to expedite troubleshooting:

1. **Switchboard version:**
   ```bash
   switchboard --version
   ```

2. **System information:**
   ```bash
   # Linux/macOS
   uname -a
   cat /etc/os-release  # Linux
   sw_vers  # macOS
   
   # Windows
   systeminfo | findstr /B /C:"OS Name" /C:"OS Version"
   ```

3. **Docker information:**
   ```bash
   docker --version
   docker info
   docker ps -a
   ```

4. **Configuration (sanitize sensitive data):**
   ```bash
   # Show configuration without API keys
   sed 's/=.*/=***REDACTED***/' ~/.kilocode/config.toml
   sed 's/=.*/=***REDACTED***/' ~/.kilocode/.env
   ```

5. **Recent logs:**
   ```bash
   # Get last 50 lines of logs
   switchboard logs --agent your-agent-name | tail -50
   
   # Or check log files directly
   tail -50 ~/.kilocode/logs/your-agent-name.log
   ```

6. **Validation output:**
   ```bash
   # Validate configuration
   switchboard validate config
   ```

7. **Environment variables:**
   ```bash
   # List relevant environment variables (sanitize)
   env | grep -E "(API|KEY|TOKEN|MODEL)" | sed 's/=.*/=***REDACTED***/'
   ```

#### Creating a Bug Report

If you believe you've found a bug, create a bug report with:

```markdown
## Description
Brief description of the issue

## Steps to Reproduce
1. Do this
2. Then do that
3. See error

## Expected Behavior
What should happen

## Actual Behavior
What actually happened (include error messages)

## Environment
- Switchboard version: ...
- OS: ...
- Docker version: ...
- Rust version: ... (if applicable)

## Configuration
```toml
# Sanitized configuration
...
```

## Logs
```
# Relevant log output
...
```
```

#### Additional Tips

- **Search before asking:** Use the search function in GitHub issues and documentation
- **Be specific:** Include exact error messages and steps to reproduce
- **Sanitize sensitive data:** Never include real API keys, tokens, or passwords
- **Check dependencies:** Ensure Docker, Rust, and other dependencies are up to date
- **Try minimal reproduction:** Simplify your configuration to isolate the issue
- **Read error messages:** Error messages often contain clues about what went wrong
- **Check permissions:** Many issues are related to file/directory permissions
- **Verify paths:** Absolute paths are often more reliable than relative paths
- **Test in isolation:** Try running commands manually before using them in Switchboard

For the most up-to-date information and community discussions, visit the [GitHub repository](https://github.com/your-repo/switchboard) and check the [Issues](https://github.com/your-repo/switchboard/issues) section.
