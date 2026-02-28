# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 2
> Focus Area: Formatting and Documentation
> Last Updated: 2026-02-28T12:41:28Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:

- Cargo.toml (build manifest)
- src/lib.rs (module root)
- src/scheduler/mod.rs (for formatting task)
- src/docker/run/run.rs, src/discord/config.rs, src/commands/validate.rs (for test organization task)

## Tasks

[Tasks in the format below, ordered safe → risky]

- [ ] [FIND-CONV-005] Fix Formatting Issue in scheduler/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Code should be formatted per `cargo fmt` conventions. Lines 1067-1073 in scheduler/mod.rs should have proper formatting with the closure spread across multiple lines.
  - 📂 Files: `src/scheduler/mod.rs`
  - 🧭 Context: The file has a formatting issue at lines 1067-1073. Current broken code:
    ```rust
    let schedule = cron::Schedule::from_str(&cron_helper::convert_to_6_field_cron(
        &agent.config.schedule,
    ))
    .map_err(|e| SchedulerError::InvalidCronSchedule {
        schedule: agent.config.schedule.clone(),
        error: e.to_string(),
    })?;
    ```
    Run `cargo fmt` to auto-fix this issue.
  - ⚡ Pre-check: `cargo fmt --check` fails (shows formatting issues)
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent of other tasks)

- [ ] [FIND-CONV-003] Improve Test Organization
  - 📚 SKILLS: `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Tests currently scattered inline should be reviewed and organized. Consider moving to dedicated test modules or the `tests/` directory where appropriate.
  - 📂 Files: `src/docker/run/run.rs`, `src/discord/config.rs`, `src/commands/validate.rs`
  - 🧭 Context: Tests are scattered inline within source files rather than in dedicated test modules. The evidence shows `#[cfg(test)]` modules mixed into source files.
  - ⚡ Pre-check: Build and tests pass before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent of other tasks)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
