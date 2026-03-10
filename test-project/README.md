# test-project

This is a Switchboard project. It contains AI coding agents that run on scheduled intervals.

## Getting Started

1. Edit `switchboard.toml` to configure your agents
2. Add prompts to the `prompts/` directory
3. Run `switchboard up` to start the scheduler

## Project Structure

```
./
├── switchboard.toml    # Main configuration file
├── .switchboard/       # Local data (logs, PID files)
├── skills/            # Project-level skills
├── prompts/           # Agent prompt files
├── .gitignore         # Git ignore rules
└── README.md          # This file
```

## Available Commands

```bash
# Start the scheduler
switchboard up

# List configured agents
switchboard list

# View logs
switchboard logs

# Stop the scheduler
switchboard down
```

For more information, see the [documentation](https://github.com/switchboard/switchboard-rs-oss/docs).
