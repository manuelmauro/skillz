# ADR 0007: Skilo Cache Directory

## Status

Proposed

## Context

Currently, `skilo add` clones repositories into a temporary directory, copies the requested skills, and discards the clone. This has several drawbacks:

1. **Repeated downloads**: Adding multiple skills from the same repo requires re-cloning each time
2. **No offline support**: Cannot install previously-used skills without network access
3. **Slow operations**: Full clone for each `skilo add` invocation
4. **No update mechanism**: Cannot efficiently check for or apply updates

Cargo solves this elegantly with `~/.cargo/git/`:

```
~/.cargo/
├── git/
│   ├── checkouts/    # Working trees at specific commits
│   └── db/           # Bare git repositories (fetch targets)
├── registry/         # Crates.io index and crate caches
└── bin/              # Installed binaries
```

## Decision

We will create a skilo-specific cache directory using the `dirs` crate for cross-platform support.

### Directory Structure

```
~/.skilo/                    # SKILO_HOME
├── config.toml              # User-level configuration
└── git/
    ├── checkouts/           # Working trees at specific commits
    │   ├── anthropics-skills-a1b2c3d/
    │   │   ├── code-review/
    │   │   └── run-tests/
    │   └── my-org-skills-e4f5g6h/
    │       └── ...
    └── db/                  # Bare git repositories (fetch targets)
        ├── anthropics-skills/
        └── my-org-skills/
```

### Location Resolution

Using `dirs` crate for cross-platform paths:

| Platform | Default Path              | Environment Override |
| -------- | ------------------------- | -------------------- |
| Linux    | `~/.skilo/`               | `SKILO_HOME`         |
| macOS    | `~/.skilo/`               | `SKILO_HOME`         |
| Windows  | `%USERPROFILE%\.skilo\`   | `SKILO_HOME`         |

Alternative XDG-compliant location (Linux):

```
~/.cache/skilo/checkouts/    # XDG_CACHE_HOME
~/.config/skilo/config.toml  # XDG_CONFIG_HOME
```

For simplicity and discoverability, we'll use `~/.skilo/` consistently across platforms, matching patterns from tools like `~/.cargo/`, `~/.npm/`, `~/.rustup/`.

### Checkout Naming

Checkouts are named using the pattern: `{owner}-{repo}-{short-hash}`

```
anthropics-skills-a1b2c3d/   # github.com/anthropics/skills @ a1b2c3d
my-org-my-skills-main/       # Branch checkout (when no specific commit)
```

This allows multiple versions of the same repo to coexist.

### Workflow

**First install:**

```bash
$ skilo add anthropics/skills/code-review
Cloning anthropics/skills...
Caching to ~/.skilo/git/db/anthropics-skills/
Checking out to ~/.skilo/git/checkouts/anthropics-skills-a1b2c3d/
Installing code-review... done
```

**Subsequent installs (same repo):**

```bash
$ skilo add anthropics/skills/run-tests
Using cached anthropics/skills (a1b2c3d)
Installing run-tests... done
```

**With update check:**

```bash
$ skilo add anthropics/skills/code-review --update
Fetching updates for anthropics/skills...
Updated: a1b2c3d -> b2c3d4e
Installing code-review... done
```

### Cache Management Commands

```bash
# Show cache status
skilo cache

Cache directory: ~/.skilo/git/
  db/: 3 repositories, 12 MB
    anthropics-skills
    my-org-tools
    examples-repo
  checkouts/: 5 checkouts, 45 MB
    anthropics-skills-a1b2c3d (2 days ago)
    anthropics-skills-f8e9d0c (1 week ago)
    my-org-tools-e4f5g6h (1 week ago)
    examples-repo-i7j8k9l (3 weeks ago)

# Clean old checkouts (keeps db for fast re-checkout)
skilo cache clean
Removing checkouts older than 30 days...
Removed: examples-repo-i7j8k9l (33 MB freed)

# Clean everything (db + checkouts)
skilo cache clean --all
Removing all cached data...
Removed 3 repositories, 5 checkouts (57 MB freed)

# Show cache path
skilo cache path
/Users/user/.skilo/git
```

### Configuration

In `~/.skilo/config.toml`:

```toml
[cache]
# Git cache directory (default: ~/.skilo/git)
dir = "~/.skilo/git"

# Maximum checkout age in days (0 = never expire)
max_age = 30

# Maximum cache size in MB (0 = unlimited)
max_size = 500
```

### Environment Variables

| Variable        | Purpose                              |
| --------------- | ------------------------------------ |
| `SKILO_HOME`    | Override base skilo directory        |
| `SKILO_CACHE`   | Override git cache directory         |
| `SKILO_OFFLINE` | Disable network, use cache only      |

### Offline Mode

```bash
$ SKILO_OFFLINE=1 skilo add anthropics/skills/code-review
Using cached anthropics/skills (a1b2c3d)
Installing code-review... done

$ SKILO_OFFLINE=1 skilo add new-org/skills/something
Error: Repository not in cache. Run without SKILO_OFFLINE to fetch.
```

## Consequences

### Positive

- Faster repeated installs from same repository
- Offline support for cached repositories
- Foundation for `skilo update` command
- Reduced network traffic and GitHub API usage
- Familiar pattern from Cargo, npm, pip

### Negative

- Disk space usage grows over time
- Cache invalidation complexity
- Another directory to manage (`~/.skilo/`)
- Need to handle corrupted cache gracefully

### Neutral

- Requires `dirs` crate (already a dependency)
- Cache is optional - fresh clones still work
- No breaking changes to existing behavior

## Implementation Notes

### Module Structure

```
src/
├── cache/
│   ├── mod.rs        # Cache trait and resolution
│   ├── checkout.rs   # Checkout management
│   └── clean.rs      # Cache cleanup logic
└── ...
```

### Key Functions

```rust
/// Get the skilo home directory.
pub fn skilo_home() -> Option<PathBuf> {
    env::var_os("SKILO_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".skilo")))
}

/// Get the git cache directory.
pub fn git_dir() -> Option<PathBuf> {
    env::var_os("SKILO_CACHE")
        .map(PathBuf::from)
        .or_else(|| skilo_home().map(|h| h.join("git")))
}

/// Get the bare repositories directory.
pub fn db_dir() -> Option<PathBuf> {
    git_dir().map(|g| g.join("db"))
}

/// Get the checkouts directory.
pub fn checkouts_dir() -> Option<PathBuf> {
    git_dir().map(|g| g.join("checkouts"))
}

/// Generate db directory name for a repo.
pub fn db_name(owner: &str, repo: &str) -> String {
    format!("{}-{}", owner, repo)
}

/// Generate checkout directory name for a repo at a specific revision.
pub fn checkout_name(owner: &str, repo: &str, rev: &str) -> String {
    format!("{}-{}-{}", owner, repo, &rev[..7.min(rev.len())])
}
```

### Cache Lookup Flow

```
1. Parse source (e.g., "anthropics/skills/code-review")
2. Check ~/.skilo/git/db/ for bare repo
3. If found: fetch updates to db
4. If not found: clone bare repo to db/
5. Check ~/.skilo/git/checkouts/ for matching revision
6. If found: use cached checkout
7. If not found: checkout from db to checkouts/
8. Copy skill from checkout to target directory
```

## Future Extensions

- **Partial clones**: Use git sparse-checkout for large repos
- **Registry**: Central skill registry with versioned packages
- **Lockfiles**: Pin exact versions for reproducible installs
- **Shared cache**: Team-shared cache via network mount or S3

## References

- [Cargo source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html)
- [Cargo git dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories)
- [dirs crate](https://docs.rs/dirs/latest/dirs/)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
