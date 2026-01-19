//! Git repository fetching operations.

use crate::git::source::GitSource;
use crate::SkiloError;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Result of a successful fetch operation.
pub struct FetchResult {
    /// The temporary directory containing the cloned repository.
    pub temp_dir: TempDir,
    /// The path to the root of the repository (or subdir if specified).
    pub root: PathBuf,
}

/// Fetch a git repository to a temporary directory.
pub fn fetch(source: &GitSource) -> Result<FetchResult, SkiloError> {
    let temp_dir = TempDir::new().map_err(SkiloError::Io)?;

    clone_repo(&source.url, source.reference(), temp_dir.path())?;

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

    Ok(FetchResult { temp_dir, root })
}

fn clone_repo(url: &str, reference: Option<&str>, dest: &Path) -> Result<Repository, SkiloError> {
    let mut builder = RepoBuilder::new();
    let mut callbacks = RemoteCallbacks::new();

    // Set up credential handling
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

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // Only use shallow clone when not specifying a branch/tag
    // (git2 has issues with shallow clone + specific refs)
    if reference.is_none() {
        fetch_opts.depth(1);
    }

    builder.fetch_options(fetch_opts);

    if let Some(ref_name) = reference {
        builder.branch(ref_name);
    }

    builder.clone(url, dest).map_err(|e| {
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
    })
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
