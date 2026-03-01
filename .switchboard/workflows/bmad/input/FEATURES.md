# Switchboard OSS — Issues to Address

## 1. Broken & Placeholder References

The repo has several references that point to the wrong place or were never updated from the internal project:

- **README badge and clone URL** reference `github.com/switchboard-ai/switchboard`, not `kkingsbe/switchboard-rs-oss`
- **Cargo.toml** has `repository = "https://github.com/yourusername/switchboard"` — a literal placeholder
- **README examples** use `kilosynth/prompter:latest` as the Docker image, which is unexplained and likely private. It's unclear how this relates to `switchboard build`
- **CI badge** links to a workflow in the wrong repo, so it will never render correctly

**Fix:** Do a full find-and-replace for all org/repo/image references and verify every link resolves.

## 2. Missing License & Contributor Basics

- No `LICENSE` file exists despite "MIT" being declared in both README and Cargo.toml. Without the actual file, the license grant is legally ambiguous.
- No `CONTRIBUTING.md` or code of conduct for an open-source project.

**Fix:** Add a standard MIT LICENSE file and a minimal CONTRIBUTING guide.

## 3. Committed Artifacts That Don't Belong

The repo contains files and directories that are clearly runtime output or internal project management materials, not source code:

| Path | What it is | Size |
|---|---|---|
| `coverage/html/` | Generated HTML coverage reports | 3.5 MB |
| `summarizer-narratives/` | 40+ AI-generated narrative outputs from actual agent runs | 100 KB |
| `comms/outbox/` | Runtime communication artifacts | 28 KB |
| `logs/combined.log` | Actual runtime log output | 5 KB |
| `plans/` | Internal planning docs | 11 KB |
| `BLOCKERS.md`, `FRONTEND_PRD.md`, `PRD.md` | Internal project management | — |
| `switchboard.toml` | Author's live working config (references private prompt files) | — |
| `invalid.toml`, `test-*.toml` (at root) | Test fixtures that belong in `tests/fixtures/` | — |
| `skills-lock.json` | Runtime lock file | — |

**Fix:** Add these to `.gitignore`, remove from git history (or at minimum from the current tree), and move root-level test TOML files into `tests/fixtures/`.

## 4. Kilo Code Dependency Is Poorly Explained

The entire tool is a wrapper around the Kilo Code CLI, but the README treats this as a minor prerequisite rather than the core dependency. A new user will have questions:

- What is Kilo Code? Is it free or paid?
- Why are the only available models `z-ai/glm-4.7` and `minimax/minimax-m2.5`? Can I use Claude, GPT-4, etc.?
- What does the Kilo Code token give me access to?

**Fix:** Add a clear "How It Works" section explaining the Kilo Code dependency, its cost model, and supported models. If the tool is meant to be general-purpose, clarify that. If it's Kilo Code-specific, own that positioning.

## 5. Oversized Files Need Decomposition

Two files are far too large for maintainability:

- `src/docker/run/run.rs` — **5,115 lines**
- `src/config/mod.rs` — **3,511 lines**

The `lib.rs` doc comment actually cites line counts per module as if they're a feature. These files should be broken into focused sub-modules.

**Fix:** Decompose each into logical sub-modules (e.g., `config/parsing.rs`, `config/validation.rs`, `config/types.rs`; `docker/run/create.rs`, `docker/run/execute.rs`, `docker/run/lifecycle.rs`).

## 6. 608 `.unwrap()` Calls

For a tool that manages Docker containers with access to user project files, unchecked unwraps are a reliability risk. Panics during container operations could leave orphaned containers or corrupted state.

**Fix:** Audit and replace `.unwrap()` calls with proper error handling (`?`, `unwrap_or_else`, `context()`). Prioritize the Docker execution and scheduler paths.

## 7. Clippy & Formatting Not Enforced

The CI workflow installs `rustfmt` and `clippy` components but **never runs them**. There are no `cargo clippy` or `cargo fmt --check` steps.

**Fix:** Add these steps to CI:
```yaml
- name: Check formatting
  run: cargo fmt --all -- --check

- name: Run clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

## 8. Empty Feature Flags

`Cargo.toml` declares `integration`, `scheduler`, and `streams` features as empty arrays with no conditional compilation that uses them (only `discord` is real). This is confusing for contributors trying to understand the build.

**Fix:** Either wire these features to actual conditional compilation or remove them.

## 9. Commit History Reads as AI-Generated

Every commit follows a pattern like `refactor(refactor1): [FIND-002A] remove unused imports` and `chore(auditor): audit complete`. This is clearly the output of the tool's own agents doing self-maintenance. While dogfooding is great, the commit history provides no human-readable narrative of the project's development.

**Fix:** Consider squashing the AI-generated maintenance commits before the OSS release, or at minimum add conventional human-authored commits for feature milestones.