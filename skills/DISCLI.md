# DISCLI Skill - Discord Communication

This skill enables sending progress updates and notifications to Discord.

## Overview

The system uses an outbox-based approach for Discord messaging. Messages are written to markdown files in the `comms/outbox/` directory, which are then processed by a background poller that sends them to Discord.

## Usage

### Sending a Discord Update

To send a Discord message:

1. Create a new markdown file in `comms/outbox/`
2. Filename format: `summarizer-update-YYYY-MM-DD-HHMM.md`
3. Content should be brief and distilled - this will be formatted with a prefix

### Message Format

The outbox poller automatically formats messages as:
```
📬 **Agent Update** — `{filename}`

{content}
```

### Example

```markdown
# Summarizer Update - 14:00 UTC

**Commit Range:** abc123 → def456

3 new commits recorded. Feature implementation completed.

**Status:** Active development in progress.

Full narrative: `summarizer-narratives/2026-02-28-1400.md`
```

### Processing

- The outbox poller runs every 60 seconds
- After successfully sending to Discord, files are moved to `comms/archive/`
- Failed sends remain in the outbox for retry

## Requirements

- Markdown file must be placed in `comms/outbox/`
- File must have `.md` extension
- Content should be concise and informative
