# Publishing to crates.io

This document describes the complete process for publishing the `switchboard` package to crates.io.

## Package Details

- **Name:** switchboard
- **Current version:** 0.1.0
- **Binary:** switchboard

## Prerequisites

Before publishing to crates.io, ensure you have:

- Cargo installed and configured
- crates.io account (https://crates.io/)
- API token from crates.io account settings

## Setup

1. **Login to crates.io:**
   ```bash
   cargo login <api-token>
   ```
   This stores the API token in `~/.cargo/credentials`.

## Pre-publishing Checklist

Before publishing, complete the following checklist:

- [ ] Update version in `Cargo.toml` to the desired release version
- [ ] Ensure all dependencies have compatible licenses
- [ ] Run `cargo publish --dry-run` to verify package is ready
- [ ] Review package description, keywords, and categories
- [ ] Ensure `README.md` exists and is well-formatted (used on crates.io)
- [ ] Ensure `LICENSE` file exists (used on crates.io)
- [ ] Review [Platform Compatibility Testing Results](PLATFORM_COMPATIBILITY.md) for target platforms

## Publishing

1. **Run the publish command:**
   ```bash
   cargo publish
   ```
   This uploads the package to crates.io.

2. **Wait for the build to complete** on crates.io (may take several minutes)

3. **Verify the publication** at https://crates.io/crates/switchboard

## After Publishing

After successful publication, complete these steps:

1. **Tag the release in git:**
   ```bash
   git tag v0.1.0
   ```

2. **Push tags to remote:**
   ```bash
   git push --tags
   ```

3. **Create a GitHub release** (optional but recommended)

## Yanking a Version (if needed)

If you need to remove or hide a published version:

**To yank (remove) a published version:**
```bash
cargo yank --vers 0.1.0
```

**To restore a yanked version:**
```bash
cargo yank --vers 0.1.0 --undo
```

**Important note:** You cannot yank the most recent version if there are no newer versions. You must publish a newer version first.

## Common Issues

### "error: failed to publish to registry"

**Cause:** Invalid or missing crates.io API token

**Solution:** 
- Verify your API token is valid in crates.io account settings
- Run `cargo login <api-token>` again to update credentials

### "cannot package a filename with a special character"

**Cause:** Files with special characters in filenames are not allowed by crates.io

**Solution:**
- Use `package.exclude` in `Cargo.toml` to exclude non-essential files
- Example:
  ```toml
  [package]
  exclude = [
      "*.md",
      ".gitignore",
      ".cargoignore",
      "scripts/*",
      "tests/*",
  ]
  ```

### Build failures on crates.io

**Cause:** The package doesn't compile on all supported platforms

**Solution:**
- Ensure the package compiles with `cargo build --release`
- Check that all dependencies are compatible with the target platforms
- Run `cargo test` to verify tests pass
- Consider platform-specific dependencies with proper conditional compilation

## Additional Resources

- [Platform Compatibility Testing Results](PLATFORM_COMPATIBILITY.md) - Test results for various platforms
- [crates.io documentation](https://doc.rates.io/)
- [Cargo book on publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
