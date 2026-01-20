//! Git repository fetching operations with caching.
//!
//! Uses a Cargo-like caching structure:
//! - `~/.skilo/git/db/` - Bare git repositories (fetch targets)
//! - `~/.skilo/git/checkouts/` - Working trees at specific commits

use crate::cache::{
    checkout_name, checkouts_dir, db_dir, db_name, ensure_dir, is_offline, parse_owner_repo,
};
use crate::git::source::GitSource;
use crate::SkiloError;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Result of a successful fetch operation.
pub struct FetchResult {
    /// The path to the root of the repository (or subdir if specified).
    pub root: PathBuf,
    /// The temporary directory (only used for non-cached fetches).
    /// Kept for backward compatibility - will be None when using cache.
    pub temp_dir: Option<TempDir>,
    /// The checkout directory (when using cache).
    pub checkout_dir: Option<PathBuf>,
    /// Whether the result came from cache.
    pub from_cache: bool,
    /// The commit hash of the checkout.
    pub commit: Option<String>,
}

/// Fetch a git repository, using cache when possible.
///
/// Caching strategy:
/// 1. Clone/fetch bare repo to `~/.skilo/git/db/{owner}-{repo}/`
/// 2. Checkout specific revision to `~/.skilo/git/checkouts/{owner}-{repo}-{rev}/`
/// 3. Return the checkout path
pub fn fetch(source: &GitSource) -> Result<FetchResult, SkiloError> {
    // Try to use cache if we can parse owner/repo
    if let Some((owner, repo)) = parse_owner_repo(&source.url) {
        return fetch_cached(source, &owner, &repo);
    }

    // Fall back to temporary directory for non-standard URLs
    fetch_to_temp(source)
}

/// Fetch using the cache directory structure.
fn fetch_cached(source: &GitSource, owner: &str, repo: &str) -> Result<FetchResult, SkiloError> {
    let db = db_dir()
        .ok_or_else(|| SkiloError::Config("Could not determine cache directory".to_string()))?;
    let checkouts = checkouts_dir()
        .ok_or_else(|| SkiloError::Config("Could not determine checkouts directory".to_string()))?;

    ensure_dir(&db).map_err(SkiloError::Io)?;
    ensure_dir(&checkouts).map_err(SkiloError::Io)?;

    let db_path = db.join(db_name(owner, repo));

    // Clone or fetch the bare repository
    let bare_repo = if db_path.exists() {
        // Open existing bare repo and fetch updates
        let repo = Repository::open_bare(&db_path).map_err(|e| SkiloError::Git {
            message: format!("Failed to open cached repo: {}", e),
        })?;

        if !is_offline() {
            fetch_updates(&repo, &source.url)?;
        }

        repo
    } else {
        if is_offline() {
            return Err(SkiloError::Network {
                message: "Repository not in cache and offline mode is enabled".to_string(),
            });
        }

        // Clone as bare repository
        clone_bare(&source.url, &db_path)?
    };

    // Resolve the reference to a commit
    let commit_id = resolve_reference(&bare_repo, source.reference())?;
    let short_commit = &commit_id[..7.min(commit_id.len())];

    // Check if we already have this checkout
    let checkout_path = checkouts.join(checkout_name(owner, repo, &commit_id));

    if !checkout_path.exists() {
        // Create the checkout from the bare repo
        checkout_from_bare(&bare_repo, &commit_id, &checkout_path)?;
    }

    // Determine the root path (may be a subdirectory)
    let root = if let Some(ref subdir) = source.subdir {
        checkout_path.join(subdir)
    } else {
        checkout_path.clone()
    };

    if !root.exists() {
        return Err(SkiloError::InvalidSource(
            source.url.clone(),
            format!(
                "Subdirectory '{}' not found in repository",
                source.subdir.as_deref().unwrap_or("")
            ),
        ));
    }

    Ok(FetchResult {
        root,
        temp_dir: None,
        checkout_dir: Some(checkout_path),
        from_cache: true,
        commit: Some(short_commit.to_string()),
    })
}

/// Fall back to fetching to a temporary directory.
fn fetch_to_temp(source: &GitSource) -> Result<FetchResult, SkiloError> {
    if is_offline() {
        return Err(SkiloError::Network {
            message: "Cannot fetch non-cached repository in offline mode".to_string(),
        });
    }

    let temp_dir = TempDir::new().map_err(SkiloError::Io)?;

    let repo = clone_repo(&source.url, source.reference(), temp_dir.path())?;

    // Get the HEAD commit
    let commit = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| c.id().to_string()[..7].to_string());

    // Determine the root path (may be a subdirectory)
    let root = if let Some(ref subdir) = source.subdir {
        temp_dir.path().join(subdir)
    } else {
        temp_dir.path().to_path_buf()
    };

    if !root.exists() {
        return Err(SkiloError::InvalidSource(
            source.url.clone(),
            format!(
                "Subdirectory '{}' not found in repository",
                source.subdir.as_deref().unwrap_or("")
            ),
        ));
    }

    Ok(FetchResult {
        root,
        temp_dir: Some(temp_dir),
        checkout_dir: None,
        from_cache: false,
        commit,
    })
}

/// Clone a bare repository.
fn clone_bare(url: &str, dest: &Path) -> Result<Repository, SkiloError> {
    let mut builder = RepoBuilder::new();
    builder.bare(true);

    let mut callbacks = RemoteCallbacks::new();
    setup_credentials(&mut callbacks);

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    builder.fetch_options(fetch_opts);

    builder.clone(url, dest).map_err(|e| map_git_error(e, url))
}

/// Fetch updates to an existing bare repository.
fn fetch_updates(repo: &Repository, url: &str) -> Result<(), SkiloError> {
    let mut remote = repo
        .find_remote("origin")
        .or_else(|_| repo.remote_anonymous(url))
        .map_err(|e| SkiloError::Git {
            message: format!("Failed to get remote: {}", e),
        })?;

    let mut callbacks = RemoteCallbacks::new();
    setup_credentials(&mut callbacks);

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote
        .fetch(&["refs/heads/*:refs/heads/*"], Some(&mut fetch_opts), None)
        .map_err(|e| SkiloError::Git {
            message: format!("Failed to fetch updates: {}", e),
        })?;

    Ok(())
}

/// Resolve a reference (branch, tag, or HEAD) to a commit ID.
fn resolve_reference(repo: &Repository, reference: Option<&str>) -> Result<String, SkiloError> {
    let commit = if let Some(ref_name) = reference {
        // Try as local branch first
        repo.find_branch(ref_name, git2::BranchType::Local)
            .ok()
            .and_then(|b| b.get().target())
            .or_else(|| {
                // Try as direct ref (refs/heads/...)
                repo.find_reference(&format!("refs/heads/{}", ref_name))
                    .ok()
                    .and_then(|r| r.target())
            })
            .or_else(|| {
                // Try as tag
                repo.find_reference(&format!("refs/tags/{}", ref_name))
                    .ok()
                    .and_then(|r| r.target())
            })
            .or_else(|| {
                // Try as direct ref
                repo.find_reference(ref_name).ok().and_then(|r| r.target())
            })
            .or_else(|| {
                // Try as commit hash
                git2::Oid::from_str(ref_name).ok()
            })
            .ok_or_else(|| SkiloError::Git {
                message: format!("Reference '{}' not found", ref_name),
            })?
    } else {
        // Default to main or master branch
        // For bare repos, HEAD is usually a symbolic ref, so we resolve it
        let head_ref = repo
            .find_reference("HEAD")
            .or_else(|_| repo.find_reference("refs/heads/main"))
            .or_else(|_| repo.find_reference("refs/heads/master"))
            .map_err(|e| SkiloError::Git {
                message: format!("Failed to find HEAD: {}", e),
            })?;

        // Resolve symbolic refs
        let resolved = head_ref.resolve().map_err(|e| SkiloError::Git {
            message: format!("Failed to resolve HEAD: {}", e),
        })?;

        resolved.target().ok_or_else(|| SkiloError::Git {
            message: "HEAD has no target".to_string(),
        })?
    };

    Ok(commit.to_string())
}

/// Checkout a specific commit from a bare repository to a working directory.
fn checkout_from_bare(
    bare_repo: &Repository,
    commit_id: &str,
    checkout_path: &Path,
) -> Result<(), SkiloError> {
    // Create the checkout directory
    std::fs::create_dir_all(checkout_path).map_err(SkiloError::Io)?;

    // Clone from the bare repo to the checkout path
    let mut builder = RepoBuilder::new();

    // Use local clone from the bare repo
    builder
        .clone(
            bare_repo.path().to_str().ok_or_else(|| SkiloError::Git {
                message: "Invalid bare repo path".to_string(),
            })?,
            checkout_path,
        )
        .map_err(|e| SkiloError::Git {
            message: format!("Failed to checkout: {}", e),
        })?;

    // Checkout the specific commit
    let checkout_repo = Repository::open(checkout_path).map_err(|e| SkiloError::Git {
        message: format!("Failed to open checkout: {}", e),
    })?;

    let oid = git2::Oid::from_str(commit_id).map_err(|e| SkiloError::Git {
        message: format!("Invalid commit ID: {}", e),
    })?;

    let commit = checkout_repo
        .find_commit(oid)
        .map_err(|e| SkiloError::Git {
            message: format!("Commit not found: {}", e),
        })?;

    checkout_repo
        .checkout_tree(commit.as_object(), None)
        .map_err(|e| SkiloError::Git {
            message: format!("Failed to checkout tree: {}", e),
        })?;

    checkout_repo
        .set_head_detached(oid)
        .map_err(|e| SkiloError::Git {
            message: format!("Failed to set HEAD: {}", e),
        })?;

    Ok(())
}

/// Clone a repository to a destination (for non-cached fetches).
fn clone_repo(url: &str, reference: Option<&str>, dest: &Path) -> Result<Repository, SkiloError> {
    let mut builder = RepoBuilder::new();
    let mut callbacks = RemoteCallbacks::new();

    setup_credentials(&mut callbacks);

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // Only use shallow clone when not specifying a branch/tag
    if reference.is_none() {
        fetch_opts.depth(1);
    }

    builder.fetch_options(fetch_opts);

    if let Some(ref_name) = reference {
        builder.branch(ref_name);
    }

    builder.clone(url, dest).map_err(|e| map_git_error(e, url))
}

/// Set up credential callbacks.
fn setup_credentials(callbacks: &mut RemoteCallbacks) {
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        // Try SSH agent first for SSH URLs
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            if let Some(username) = username_from_url {
                return Cred::ssh_key_from_agent(username);
            }
        }

        // Try default credentials (git credential helper)
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            return Cred::credential_helper(
                &git2::Config::open_default()?,
                _url,
                username_from_url,
            );
        }

        // Fall back to default for public repos
        if allowed_types.contains(git2::CredentialType::DEFAULT) {
            return Cred::default();
        }

        Err(git2::Error::from_str("no valid credentials available"))
    });
}

/// Map git2 errors to SkiloError.
fn map_git_error(e: git2::Error, url: &str) -> SkiloError {
    let message = e.message().to_string();
    let code = e.code();

    if message.contains("Could not resolve host")
        || message.contains("network")
        || message.contains("connection")
    {
        SkiloError::Network { message }
    } else if code == git2::ErrorCode::NotFound {
        SkiloError::RepoNotFound {
            url: url.to_string(),
        }
    } else {
        SkiloError::Git { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_nonexistent_repo() {
        let source = GitSource {
            url: "https://github.com/nonexistent-owner-xyz/nonexistent-repo-xyz.git".to_string(),
            branch: None,
            tag: None,
            subdir: None,
        };

        let result = fetch(&source);
        assert!(result.is_err());
    }
}
