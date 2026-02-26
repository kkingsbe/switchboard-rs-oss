# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Skills feature: Integrate with skills.sh for agent skill management
  - Added `switchboard skills` CLI subcommand family for browsing, installing, and managing skills
  - Added `switchboard skills list` command to browse available skills from skills.sh registry
  - Added `switchboard skills install <source>` command to install skills from GitHub/npm/local
  - Added `switchboard skills installed` command to list installed skills with agent assignments
  - Added `switchboard skills remove <name>` command to remove installed skills
  - Added `switchboard skills update [<skill-name>]` command to update installed skills
  - Added per-agent `skills` field in `[[agent]]` configuration for declaring skills
  - Added automatic skill installation inside Docker containers at agent startup
  - Added container skill installation failure detection and logging with `[SKILL INSTALL]` prefix
  - Added skill-related metrics tracking (Total Skills Installed, Total Skills Failed, Avg Install Time)
  - Added skills validation in `switchboard validate` (format validation, empty list warnings, duplicate detection)
  - Added comprehensive troubleshooting section for common skill-related issues
  - Added container skill installation behavior documentation
  - Added skill installation failure handling documentation with metrics tracking

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.1.0]

### Added

#### Skills Feature

New `switchboard skills` CLI commands for managing agent capabilities:

- `switchboard skills list` - List available skills from npm registry
- `switchboard skills install` - Install skills into agent containers
- `switchboard skills installed` - Show skills installed in agent containers
- `switchboard skills remove` - Remove installed skills
- `switchboard skills update` - Update installed skills
- `switchboard skills validate` - Validate skill configurations

**Technical Implementation:**

- SKILL.md frontmatter parser for extracting skill metadata (AC-04)
- Container entrypoint script generation for skill loading (AC-08)
- Docker-based skill installation with proper container networking (AC-08)
- Error handling for failed installations with rollback support (AC-09)
- Configuration validation for skill definitions (AC-10)

**Acceptance Criteria Met:**

- AC-04: SKILL.md Frontmatter Parser
- AC-05: `switchboard skills remove` command
- AC-06: `switchboard skills update` command
- AC-08: Container Entrypoint Script Generation
- AC-09: Failed Skill Install Handling
- AC-10: Config Validation for skills
