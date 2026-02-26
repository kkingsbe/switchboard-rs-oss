# DEFERRED_TASKS.md

> Last Updated: 2026-02-16T02:30:00Z
> Purpose: Track future work planned for post-v0.1.0 releases

## Overview

This file tracks tasks that are intentionally deferred to future releases and do NOT block current development progress. These are planned enhancements or documentation tasks that will be addressed after v0.1.0.

---

## Post-v0.1.0 Release Tasks

### Documentation Tasks

#### 1. Document cargo install from crates.io
- **Status:** Deferred until v0.1.0 release
- **Reasoning:** Cannot document installation from crates.io until the package is published
- **Planned For:** Immediately after v0.1.0 release
- **Dependencies:** v0.1.0 milestone completion
- **Related:** PRD §14 Success Criterion #1, docs/CRATES_IO_PUBLISHING.md

#### 2. Document binary download and installation
- **Status:** Deferred until after v0.1.0 release
- **Reasoning:** Binary distribution infrastructure planned for post-v0.1.0
- **Planned For:** Post-v0.1.0 enhancement
- **Dependencies:** v0.1.0 release, binary build pipeline setup
- **Related:** PRD §12 Future Considerations

---

## Known Limitations for v0.1.0

### Platform Testing

#### 1. macOS Apple Silicon (aarch64) Testing
- **Status:** Known limitation for v0.1.0
- **Reasoning:** Current environment is Linux 6.6, cannot test on macOS aarch64 without Apple Silicon hardware
- **Impact:** Success Criterion #1 (`cargo install` from repo) cannot be fully verified on macOS aarch64
- **Workaround:** Document limitation in docs/PLATFORM_COMPATIBILITY.md, proceed with v0.1.0 release
- **Related:** docs/PLATFORM_COMPATIBILITY.md, BLOCKERS.md #1
- **Planned For:** Post-v0.1.0 (when Apple Silicon testing environment available)

---

## Future Enhancements (Post-v0.1.0)

### Infrastructure
- Automated release workflow (`.github/workflows/release.yml`)
- Binary distribution pipeline
- Cross-platform CI testing on macOS and Windows

### Documentation
- Advanced usage examples
- Troubleshooting guides expansion
- Video tutorials

### Features
- Web UI / dashboard
- Remote management API
- Advanced scheduling features

---

## Notes

- Items in this file do NOT block current development
- These are tracked for future planning and prioritization
- When moving from DEFERRED_TASKS.md to BACKLOG.md or TODO.md, remove from this file
- Reference PRD sections and related documentation for context
