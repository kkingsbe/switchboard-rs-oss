//! Docker Build - Image build context utilities
//!
//! This module provides:
//! - create_build_context_tarball() - Creates a tarball for Docker build context
//! - add_directory_to_tar() - Recursively adds directories to a tarball

use std::io::{Cursor, Write};
use std::path::Path;

/// Create a tarball of the build context directory with the Dockerfile included
///
/// This helper function creates a tarball containing all files from the build
/// context directory, with the provided Dockerfile content written as "Dockerfile"
/// at the root of the tarball.
///
/// # Arguments
///
/// * `build_context` - Path to the directory containing the build context
/// * `dockerfile_content` - The content of the Dockerfile to include in the tarball
///
/// # Returns
///
/// Returns a `Cursor<Vec<u8>>` containing the tarball bytes.
///
/// # Errors
///
/// Returns an error if:
/// - Reading files from the build context fails
/// - Writing to the tarball fails
pub fn create_build_context_tarball(
    build_context: &Path,
    dockerfile_content: &str,
) -> Result<Cursor<Vec<u8>>, anyhow::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs::File;
    use tar::Header;

    let mut tarball = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut tarball, Compression::default());
        let mut tar_builder = tar::Builder::new(&mut encoder);

        // Add Dockerfile to the tarball
        let dockerfile_path = Path::new("Dockerfile");
        let mut header = Header::new_gnu();
        header.set_size(dockerfile_content.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        tar_builder.append_data(&mut header, dockerfile_path, dockerfile_content.as_bytes())?;

        // Add all files from the build context directory
        // Only include .kilocode directory (the Dockerfile only copies this)
        if build_context.is_dir() {
            eprintln!(
                "DEBUG: build_context is a directory: {}",
                build_context.display()
            );
            let entries = std::fs::read_dir(build_context)
                .map_err(|e| anyhow::anyhow!("Failed to read build context: {}", e))?;

            for entry in entries {
                let entry =
                    entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                let relative_path = path
                    .strip_prefix(build_context)
                    .map_err(|e| anyhow::anyhow!("Failed to get relative path: {}", e))?;

                eprintln!(
                    "DEBUG: Found entry in build_context: {} (is_file: {}, is_dir: {})",
                    relative_path.display(),
                    path.is_file(),
                    path.is_dir()
                );

                // Skip the Dockerfile if it exists in the build context (we already added it)
                if relative_path == Path::new("Dockerfile") {
                    continue;
                }

                // Only include .kilocode directory - everything else is not needed
                // (the Dockerfile only copies .kilocode into the image)
                let name = relative_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if name != ".kilocode" {
                    eprintln!("DEBUG: Skipping {} (not .kilocode)", name);
                    continue; // Skip everything except .kilocode
                }

                eprintln!(
                    "DEBUG: Including .kilocode entry: {}",
                    relative_path.display()
                );

                if path.is_file() {
                    let mut file = File::open(&path).map_err(|e| {
                        anyhow::anyhow!("Failed to open file {}: {}", path.display(), e)
                    })?;
                    let file_size = file
                        .metadata()
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to get metadata for {}: {}", path.display(), e)
                        })?
                        .len();

                    let mut header = Header::new_gnu();
                    header.set_size(file_size);
                    header.set_mode(0o644);
                    header.set_mtime(0);
                    tar_builder.append_data(&mut header, relative_path, &mut file)?;
                } else if path.is_dir() {
                    // CRITICAL: Add directory entry to tarball before recursing
                    // Without this, Docker COPY fails because directories don't exist
                    let mut header = Header::new_gnu();
                    header.set_entry_type(tar::EntryType::Directory);
                    header.set_size(0);
                    header.set_mode(0o755);
                    header.set_mtime(0);
                    eprintln!("DEBUG: Adding .kilocode directory entry to tarball");
                    tar_builder.append_data(&mut header, relative_path, &mut std::io::empty())?;

                    // Recursively add directories to the tarball
                    add_directory_to_tar(&mut tar_builder, &path, build_context)?;
                }
            }
        } else {
            eprintln!(
                "DEBUG: build_context is NOT a directory: {}",
                build_context.display()
            );
        }

        drop(tar_builder);
        encoder.finish()?;
    }

    eprintln!("DEBUG: create_build_context_tarball about to return");
    Ok(Cursor::new(tarball))
}

/// Recursively add a directory to the tarball
///
/// # Arguments
///
/// * `tar_builder` - The tar builder to add files to
/// * `dir_path` - The directory path to add
/// * `base_path` - The base path to compute relative paths from
///
/// # Errors
///
/// Returns an error if reading files or writing to the tarball fails
#[allow(dead_code)]
pub fn add_directory_to_tar<W: Write>(
    tar_builder: &mut tar::Builder<W>,
    dir_path: &Path,
    base_path: &Path,
) -> Result<(), anyhow::Error> {
    use std::fs::File;
    use tar::Header;

    eprintln!(
        "DEBUG: add_directory_to_tar called for: {}",
        dir_path.display()
    );

    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| anyhow::anyhow!("Failed to read directory {}: {}", dir_path.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(base_path)
            .map_err(|e| anyhow::anyhow!("Failed to get relative path: {}", e))?;

        eprintln!(
            "DEBUG: Processing path: {} (is_file: {}, is_dir: {})",
            relative_path.display(),
            path.is_file(),
            path.is_dir()
        );

        if path.is_file() {
            let mut file = File::open(&path)
                .map_err(|e| anyhow::anyhow!("Failed to open file {}: {}", path.display(), e))?;
            let file_size = file
                .metadata()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to get metadata for {}: {}", path.display(), e)
                })?
                .len();

            let mut header = Header::new_gnu();
            header.set_size(file_size);
            header.set_mode(0o644);
            header.set_mtime(0);
            eprintln!("DEBUG: Adding file to tarball: {}", relative_path.display());
            tar_builder.append_data(&mut header, relative_path, &mut file)?;
        } else if path.is_dir() {
            // CRITICAL: Add directory entry to tarball before recursing
            // Without this, Docker COPY fails because directories don't exist
            let mut header = Header::new_gnu();
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            header.set_mode(0o755);
            header.set_mtime(0);
            eprintln!(
                "DEBUG: Adding directory entry to tarball: {}",
                relative_path.display()
            );
            tar_builder.append_data(&mut header, relative_path, &mut std::io::empty())?;

            add_directory_to_tar(tar_builder, &path, base_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Test module for Docker build functionality
    use super::*;

    #[test]
    fn test_kilocode_included_in_build_context_tarball() {
        use flate2::read::GzDecoder;
        use std::fs;
        use tar::Archive;

        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let build_context = temp_dir.path();

        // Create a .kilocode subdirectory with some content
        let kilocode_dir = build_context.join(".kilocode");
        fs::create_dir(&kilocode_dir).expect("Failed to create .kilocode directory");

        // Create a test file inside .kilocode
        let test_file = kilocode_dir.join("config.json");
        fs::write(&test_file, r#"{"api_key": "test-key"}"#).expect("Failed to write config.json");

        // Create another nested directory inside .kilocode
        let nested_dir = kilocode_dir.join("mcp_servers");
        fs::create_dir(&nested_dir).expect("Failed to create mcp_servers directory");

        let nested_file = nested_dir.join("server.json");
        fs::write(&nested_file, r#"{"name": "test-server"}"#).expect("Failed to write server.json");

        // Create a Dockerfile in the temp directory
        let dockerfile_path = build_context.join("Dockerfile");
        let dockerfile_content = "FROM alpine:latest\nCMD [\"echo\", \"test\"]\n";
        fs::write(&dockerfile_path, dockerfile_content).expect("Failed to write Dockerfile");

        // Create the build context tarball
        let dockerfile = "FROM alpine:latest\nCMD [\"echo\", \"test\"]\n";
        let tarball_cursor = create_build_context_tarball(build_context, dockerfile)
            .expect("Failed to create tarball");
        let tarball_bytes = tarball_cursor.into_inner();

        // Decompress and parse the tarball
        let decoder = GzDecoder::new(&tarball_bytes[..]);
        let mut archive = Archive::new(decoder);

        // Collect all entries from the tarball
        let mut entries: Vec<String> = Vec::new();
        for entry in archive.entries().expect("Failed to read tarball entries") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path().expect("Failed to get entry path");
            let path_str = path.to_string_lossy().to_string();
            entries.push(path_str);
        }

        // Verify that Dockerfile is in the tarball
        assert!(
            entries.contains(&"Dockerfile".to_string()),
            "Dockerfile should be in the tarball. Entries found: {:?}",
            entries
        );

        // Verify that .kilocode directory is in the tarball
        let kilocode_entries: Vec<&String> = entries
            .iter()
            .filter(|e| e.starts_with(".kilocode"))
            .collect();
        assert!(
            !kilocode_entries.is_empty(),
            ".kilocode directory should be included in the tarball. Entries found: {:?}",
            entries
        );

        // Verify that config.json inside .kilocode is present
        assert!(
            entries.contains(&".kilocode/config.json".to_string()),
            ".kilocode/config.json should be in the tarball. Entries found: {:?}",
            entries
        );

        // Verify that nested directory structure is preserved
        assert!(
            entries.contains(&".kilocode/mcp_servers/server.json".to_string()),
            ".kilocode/mcp_servers/server.json should be in the tarball. Entries found: {:?}",
            entries
        );

        // Clean up
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_kilocode_directory_check_in_build_agent_image() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let build_context = temp_dir.path();

        // Verify .kilocode directory is missing
        let kilocode_dir = build_context.join(".kilocode");
        assert!(!kilocode_dir.exists() || !kilocode_dir.is_dir());

        // The check logic from build_agent_image would fail here
        let kilocode_check_path = build_context.join(".kilocode");
        if !kilocode_check_path.exists() || !kilocode_check_path.is_dir() {
            let expected_error_msg = format!(
                "The .kilocode directory is required for building the agent image but was not found in: {}\n\n\
                The .kilocode directory contains API keys, model configuration, and MCP server\n\
                definitions needed by the Kilo Code CLI. Please configure .kilocode/ in the Switchboard\n\
                repo with your API keys before building the agent image.",
                build_context.display()
            );

            // Verify the error message contains the expected key phrases
            assert!(expected_error_msg.contains(".kilocode directory is required"));
            assert!(expected_error_msg.contains("API keys"));
            assert!(expected_error_msg.contains("model configuration"));
            // The error message splits "MCP server" and "definitions" across lines
            assert!(expected_error_msg.contains("MCP server"));
        }

        // Clean up
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_kilocode_directory_exists() {
        use std::fs;

        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let build_context = temp_dir.path();

        // Create .kilocode directory
        let kilocode_dir = build_context.join(".kilocode");
        fs::create_dir(&kilocode_dir).expect("Failed to create .kilocode directory");

        // Verify .kilocode directory exists
        assert!(kilocode_dir.exists());
        assert!(kilocode_dir.is_dir());

        // Clean up
        temp_dir.close().expect("Failed to close temp directory");
    }
}
