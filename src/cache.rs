//! Cache directory management for git repositories.
//!
//! Provides a Cargo-like caching structure:
//! ```text
//! ~/.skilo/
//! ├── config.toml
//! └── git/
//!     ├── checkouts/    # Working trees at specific commits
//!     └── db/           # Bare git repositories (fetch targets)
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Get the skilo home directory.
///
/// Resolution order:
/// 1. `SKILO_HOME` environment variable
/// 2. `~/.skilo/`
pub fn skilo_home() -> Option<PathBuf> {
    env::var_os("SKILO_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".skilo")))
}

/// Get the git cache directory.
///
/// Resolution order:
/// 1. `SKILO_CACHE` environment variable
/// 2. `~/.skilo/git/`
pub fn git_dir() -> Option<PathBuf> {
    env::var_os("SKILO_CACHE")
        .map(PathBuf::from)
        .or_else(|| skilo_home().map(|h| h.join("git")))
}

/// Get the bare repositories directory (`~/.skilo/git/db/`).
pub fn db_dir() -> Option<PathBuf> {
    git_dir().map(|g| g.join("db"))
}

/// Get the checkouts directory (`~/.skilo/git/checkouts/`).
pub fn checkouts_dir() -> Option<PathBuf> {
    git_dir().map(|g| g.join("checkouts"))
}

/// Generate db directory name for a repo.
///
/// Format: `{owner}-{repo}`
pub fn db_name(owner: &str, repo: &str) -> String {
    format!("{}-{}", owner, repo)
}

/// Generate checkout directory name for a repo at a specific revision.
///
/// Format: `{owner}-{repo}-{short_rev}`
pub fn checkout_name(owner: &str, repo: &str, rev: &str) -> String {
    let short_rev = &rev[..7.min(rev.len())];
    format!("{}-{}-{}", owner, repo, short_rev)
}

/// Parse owner and repo from a git URL.
///
/// Supports:
/// - `https://github.com/owner/repo.git`
/// - `git@github.com:owner/repo.git`
pub fn parse_owner_repo(url: &str) -> Option<(String, String)> {
    let url = url.trim_end_matches(".git");

    // SSH format: git@github.com:owner/repo
    if url.starts_with("git@") {
        let path = url.split(':').nth(1)?;
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // HTTPS format: https://github.com/owner/repo
    if let Some(idx) = url.find("://") {
        let path = &url[idx + 3..];
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 3 {
            return Some((parts[1].to_string(), parts[2].to_string()));
        }
    }

    None
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Check if offline mode is enabled via `SKILO_OFFLINE` environment variable.
pub fn is_offline() -> bool {
    env::var("SKILO_OFFLINE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Information about a cached repository in db/.
#[derive(Debug)]
pub struct CachedRepo {
    /// Repository name (owner-repo format).
    pub name: String,
    /// Path to the bare repository.
    pub path: PathBuf,
    /// Size in bytes.
    pub size: u64,
}

/// Information about a checkout in checkouts/.
#[derive(Debug)]
pub struct CachedCheckout {
    /// Checkout name (owner-repo-rev format).
    pub name: String,
    /// Path to the checkout.
    pub path: PathBuf,
    /// Size in bytes.
    pub size: u64,
    /// Last modified time.
    pub modified: Option<SystemTime>,
}

/// Get cache statistics.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Repositories in db/.
    pub repos: Vec<CachedRepo>,
    /// Checkouts in checkouts/.
    pub checkouts: Vec<CachedCheckout>,
    /// Total size of db/ in bytes.
    pub db_size: u64,
    /// Total size of checkouts/ in bytes.
    pub checkouts_size: u64,
}

impl CacheStats {
    /// Collect cache statistics.
    pub fn collect() -> Self {
        let mut stats = CacheStats::default();

        // Collect db stats
        if let Some(db) = db_dir() {
            if db.exists() {
                if let Ok(entries) = fs::read_dir(&db) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_dir() {
                            let size = dir_size(&path);
                            stats.db_size += size;
                            stats.repos.push(CachedRepo {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path,
                                size,
                            });
                        }
                    }
                }
            }
        }

        // Collect checkout stats
        if let Some(checkouts) = checkouts_dir() {
            if checkouts.exists() {
                if let Ok(entries) = fs::read_dir(&checkouts) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_dir() {
                            let size = dir_size(&path);
                            let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
                            stats.checkouts_size += size;
                            stats.checkouts.push(CachedCheckout {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path,
                                size,
                                modified,
                            });
                        }
                    }
                }
            }
        }

        // Sort by name
        stats.repos.sort_by(|a, b| a.name.cmp(&b.name));
        stats.checkouts.sort_by(|a, b| a.name.cmp(&b.name));

        stats
    }

    /// Total cache size in bytes.
    pub fn total_size(&self) -> u64 {
        self.db_size + self.checkouts_size
    }
}

/// Calculate directory size recursively.
fn dir_size(path: &PathBuf) -> u64 {
    let mut size = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path);
            } else if let Ok(meta) = entry.metadata() {
                size += meta.len();
            }
        }
    }

    size
}

/// Format bytes as human-readable string.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Clean checkouts older than the given age in days.
pub fn clean_old_checkouts(max_age_days: u32) -> std::io::Result<(usize, u64)> {
    let checkouts = match checkouts_dir() {
        Some(c) => c,
        None => return Ok((0, 0)),
    };

    if !checkouts.exists() {
        return Ok((0, 0));
    }

    let max_age = std::time::Duration::from_secs(max_age_days as u64 * 24 * 60 * 60);
    let now = SystemTime::now();
    let mut removed = 0;
    let mut freed = 0;

    if let Ok(entries) = fs::read_dir(&checkouts) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            let size = dir_size(&path);
                            if fs::remove_dir_all(&path).is_ok() {
                                removed += 1;
                                freed += size;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((removed, freed))
}

/// Clean all cache (db + checkouts).
pub fn clean_all() -> std::io::Result<(usize, usize, u64)> {
    let mut repos_removed = 0;
    let mut checkouts_removed = 0;
    let mut freed = 0;

    // Clean checkouts
    if let Some(checkouts) = checkouts_dir() {
        if checkouts.exists() {
            if let Ok(entries) = fs::read_dir(&checkouts) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let size = dir_size(&path);
                        if fs::remove_dir_all(&path).is_ok() {
                            checkouts_removed += 1;
                            freed += size;
                        }
                    }
                }
            }
        }
    }

    // Clean db
    if let Some(db) = db_dir() {
        if db.exists() {
            if let Ok(entries) = fs::read_dir(&db) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let size = dir_size(&path);
                        if fs::remove_dir_all(&path).is_ok() {
                            repos_removed += 1;
                            freed += size;
                        }
                    }
                }
            }
        }
    }

    Ok((repos_removed, checkouts_removed, freed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_owner_repo_https() {
        let (owner, repo) = parse_owner_repo("https://github.com/anthropics/skills.git").unwrap();
        assert_eq!(owner, "anthropics");
        assert_eq!(repo, "skills");
    }

    #[test]
    fn test_parse_owner_repo_ssh() {
        let (owner, repo) = parse_owner_repo("git@github.com:anthropics/skills.git").unwrap();
        assert_eq!(owner, "anthropics");
        assert_eq!(repo, "skills");
    }

    #[test]
    fn test_db_name() {
        assert_eq!(db_name("anthropics", "skills"), "anthropics-skills");
    }

    #[test]
    fn test_checkout_name() {
        assert_eq!(
            checkout_name("anthropics", "skills", "abc1234def"),
            "anthropics-skills-abc1234"
        );
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
