//! The `self update` command implementation.

use crate::cli::{Cli, SelfUpdateArgs};
use crate::config::Config;
use crate::error::{Result, SkiloError};
use colored::Colorize;
use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};

const GITHUB_API_URL: &str = "https://api.github.com/repos/manuelmauro/skilo/releases/latest";
const USER_AGENT: &str = concat!("skilo/", env!("CARGO_PKG_VERSION"));

/// GitHub release response structure.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub asset structure.
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Get the current version of skilo.
fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Fetch the latest release information from GitHub.
fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| SkiloError::Network {
            message: format!("Failed to create HTTP client: {}", e),
        })?;

    let response = client
        .get(GITHUB_API_URL)
        .send()
        .map_err(|e| SkiloError::Network {
            message: format!("Failed to fetch release info: {}", e),
        })?;

    if !response.status().is_success() {
        return Err(SkiloError::Network {
            message: format!(
                "GitHub API returned status {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            ),
        });
    }

    response
        .json::<GitHubRelease>()
        .map_err(|e| SkiloError::Network {
            message: format!("Failed to parse release info: {}", e),
        })
}

/// Detect the current platform's target triple.
fn detect_target() -> Option<&'static str> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Some("aarch64-apple-darwin")
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Some("x86_64-unknown-linux-gnu")
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        Some("x86_64-pc-windows-msvc")
    }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    {
        None
    }
}

/// Find the asset URL for the current platform.
fn find_asset_url<'a>(release: &'a GitHubRelease, target: &str) -> Option<&'a str> {
    let expected_name = if cfg!(windows) {
        format!("skilo-{}.zip", target)
    } else {
        format!("skilo-{}.tar.gz", target)
    };

    release
        .assets
        .iter()
        .find(|a| a.name == expected_name)
        .map(|a| a.browser_download_url.as_str())
}

/// Download the binary from the given URL.
fn download_binary(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| SkiloError::Network {
            message: format!("Failed to create HTTP client: {}", e),
        })?;

    let response = client.get(url).send().map_err(|e| SkiloError::Network {
        message: format!("Failed to download binary: {}", e),
    })?;

    if !response.status().is_success() {
        return Err(SkiloError::Network {
            message: format!("Download failed with status {}", response.status()),
        });
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| SkiloError::Network {
            message: format!("Failed to read download: {}", e),
        })
}

/// Extract the binary from a tar.gz archive.
#[cfg(not(windows))]
fn extract_binary(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let cursor = Cursor::new(data);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(SkiloError::Io)? {
        let mut entry = entry.map_err(SkiloError::Io)?;
        let path = entry.path().map_err(SkiloError::Io)?;

        if path.file_name().map(|n| n == "skilo").unwrap_or(false) {
            let mut binary = Vec::new();
            entry.read_to_end(&mut binary).map_err(SkiloError::Io)?;
            return Ok(binary);
        }
    }

    Err(SkiloError::Network {
        message: "Binary not found in archive".to_string(),
    })
}

/// Extract the binary from a zip archive (Windows).
#[cfg(windows)]
fn extract_binary(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| SkiloError::Network {
        message: format!("Failed to open zip archive: {}", e),
    })?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| SkiloError::Network {
            message: format!("Failed to read zip entry: {}", e),
        })?;

        if file.name().ends_with("skilo.exe") || file.name() == "skilo" {
            let mut binary = Vec::new();
            file.read_to_end(&mut binary).map_err(SkiloError::Io)?;
            return Ok(binary);
        }
    }

    Err(SkiloError::Network {
        message: "Binary not found in archive".to_string(),
    })
}

/// Check if the executable appears to be installed via cargo.
fn is_cargo_installed() -> bool {
    let Ok(current_exe) = env::current_exe() else {
        return false;
    };

    // Check if the executable is in ~/.cargo/bin/
    if let Some(home) = dirs::home_dir() {
        let cargo_bin = home.join(".cargo").join("bin");
        if let Some(exe_dir) = current_exe.parent() {
            return exe_dir == cargo_bin;
        }
    }

    false
}

/// Replace the current executable with the new binary.
fn replace_binary(new_binary: &[u8]) -> Result<()> {
    let current_exe = env::current_exe().map_err(SkiloError::Io)?;

    // Create temp file in the same directory as the executable
    let exe_dir = current_exe.parent().ok_or_else(|| {
        SkiloError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Cannot find executable directory",
        ))
    })?;

    let temp_path = exe_dir.join(".skilo-update-tmp");
    let backup_path = exe_dir.join(".skilo-backup");

    // Write new binary to temp file
    {
        let mut file = File::create(&temp_path).map_err(SkiloError::Io)?;
        file.write_all(new_binary).map_err(SkiloError::Io)?;
    }

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)
            .map_err(SkiloError::Io)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms).map_err(SkiloError::Io)?;
    }

    // Backup current executable
    if let Err(e) = fs::rename(&current_exe, &backup_path) {
        // On Windows, the running executable might be locked
        // Try to copy instead
        fs::copy(&current_exe, &backup_path).map_err(|_| SkiloError::Io(e))?;
    }

    // Move new binary to current executable location
    if let Err(e) = fs::rename(&temp_path, &current_exe) {
        // Restore backup if move fails
        let _ = fs::rename(&backup_path, &current_exe);
        let _ = fs::remove_file(&temp_path);
        return Err(SkiloError::Io(e));
    }

    // Clean up backup
    let _ = fs::remove_file(&backup_path);

    Ok(())
}

/// Parse version string, removing 'v' prefix if present.
fn parse_version(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

/// Compare versions to determine if an update is available.
fn is_newer_version(current: &str, latest: &str) -> bool {
    let current = parse_version(current);
    let latest = parse_version(latest);

    // Simple semver comparison
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    for (c, l) in current_parts.iter().zip(latest_parts.iter()) {
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

/// Run the self update command.
pub fn run(args: SelfUpdateArgs, _config: &Config, cli: &Cli) -> Result<i32> {
    let current_version = get_current_version();
    let target = detect_target().ok_or_else(|| SkiloError::Network {
        message: "Unsupported platform for self-update".to_string(),
    })?;

    // Check for cargo installation
    let cargo_installed = is_cargo_installed();
    if cargo_installed && !args.check {
        println!(
            "{} skilo appears to be installed via cargo",
            "Warning:".yellow().bold()
        );
        println!(
            "  Consider using {} instead to avoid version conflicts.",
            "cargo install skilo".cyan()
        );
        println!();

        if !args.yes {
            print!("Continue with self-update anyway? [y/N] ");
            io::stdout().flush().map_err(SkiloError::Io)?;

            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(SkiloError::Io)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Update cancelled.");
                return Ok(0);
            }
            println!();
        }
    }

    if !cli.quiet {
        println!("Current version: {}", current_version.cyan());
        println!("Platform: {}", target);
        println!();
        println!("Checking for updates...");
    }

    let release = fetch_latest_release()?;
    let latest_version = parse_version(&release.tag_name);

    if !is_newer_version(current_version, &release.tag_name) {
        if !cli.quiet {
            println!(
                "\n{} skilo is already up to date (v{})",
                "✓".green(),
                current_version
            );
        }
        return Ok(0);
    }

    if !cli.quiet {
        println!(
            "\n{} New version available: {} → {}",
            "→".blue(),
            current_version.yellow(),
            latest_version.green()
        );
    }

    if args.check {
        return Ok(0);
    }

    let asset_url = find_asset_url(&release, target).ok_or_else(|| SkiloError::Network {
        message: format!("No binary available for platform: {}", target),
    })?;

    // Confirm update unless --yes is specified
    if !args.yes {
        print!("\nDo you want to update? [y/N] ");
        io::stdout().flush().map_err(SkiloError::Io)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(SkiloError::Io)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            if !cli.quiet {
                println!("Update cancelled.");
            }
            return Ok(0);
        }
    }

    if !cli.quiet {
        println!("\nDownloading skilo v{}...", latest_version);
    }

    let archive_data = download_binary(asset_url)?;

    if !cli.quiet {
        println!("Extracting...");
    }

    let binary_data = extract_binary(&archive_data)?;

    if !cli.quiet {
        println!("Installing...");
    }

    replace_binary(&binary_data)?;

    if !cli.quiet {
        println!(
            "\n{} Successfully updated skilo to v{}",
            "✓".green(),
            latest_version
        );
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("v1.0.0"), "1.0.0");
        assert_eq!(parse_version("1.0.0"), "1.0.0");
        assert_eq!(parse_version("v0.5.0"), "0.5.0");
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("0.5.0", "v0.6.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("2.0.0", "1.0.0"));
    }

    #[test]
    fn test_detect_target() {
        // This should return Some on supported platforms
        let target = detect_target();
        #[cfg(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64"),
        ))]
        assert!(target.is_some());
    }
}
