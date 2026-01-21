//! Installation scope handling (project vs global).

use crate::agent::Agent;
use std::path::{Path, PathBuf};

/// Installation scope for skills.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    /// Project-level installation (relative to project root).
    #[default]
    Project,
    /// Global installation (user home directory).
    Global,
}

impl Scope {
    /// Returns true if this is a global scope.
    pub fn is_global(&self) -> bool {
        matches!(self, Scope::Global)
    }

    /// Returns true if this is a project scope.
    pub fn is_project(&self) -> bool {
        matches!(self, Scope::Project)
    }

    /// Resolve the skills directory for this scope and agent.
    pub fn resolve_skills_dir(&self, agent: Agent, project_root: &Path) -> Option<PathBuf> {
        match self {
            Scope::Project => Some(agent.resolve_project_skills_dir(project_root)),
            Scope::Global => agent.resolve_global_skills_dir(),
        }
    }

    /// Get the display name for this scope.
    pub fn display_name(&self) -> &'static str {
        match self {
            Scope::Project => "project",
            Scope::Global => "global",
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Installed skill information.
#[derive(Debug, Clone)]
pub struct InstalledSkill {
    /// The skill name.
    pub name: String,
    /// The skill description.
    pub description: String,
    /// Path to the skill directory.
    pub path: PathBuf,
    /// The agent this skill is installed for (None if using generic ./skills/).
    pub agent: Option<Agent>,
    /// Installation scope.
    pub scope: Scope,
}

/// List installed skills at a given scope.
pub fn list_skills(agent: Agent, scope: Scope, project_root: &Path) -> Vec<InstalledSkill> {
    let Some(skills_dir) = scope.resolve_skills_dir(agent, project_root) else {
        return Vec::new();
    };

    list_skills_from_path(&skills_dir, Some(agent), scope)
}

/// List installed skills from a specific directory path.
pub fn list_skills_from_path(
    skills_dir: &Path,
    agent: Option<Agent>,
    scope: Scope,
) -> Vec<InstalledSkill> {
    if !skills_dir.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();

    if let Ok(entries) = std::fs::read_dir(skills_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    if let Some(info) = read_skill_info(&path) {
                        skills.push(InstalledSkill {
                            name: info.0,
                            description: info.1,
                            path,
                            agent,
                            scope,
                        });
                    }
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// List all installed skills (project + global) for an agent.
pub fn list_all_skills(agent: Agent, project_root: &Path) -> Vec<InstalledSkill> {
    let mut skills = Vec::new();
    skills.extend(list_skills(agent, Scope::Project, project_root));
    skills.extend(list_skills(agent, Scope::Global, project_root));
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Check if a skill exists at a scope.
pub fn skill_exists(name: &str, agent: Agent, scope: Scope, project_root: &Path) -> bool {
    let Some(skills_dir) = scope.resolve_skills_dir(agent, project_root) else {
        return false;
    };
    skills_dir.join(name).join("SKILL.md").exists()
}

/// Check if a skill exists at the opposite scope (for shadow warnings).
pub fn skill_exists_other_scope(
    name: &str,
    agent: Agent,
    scope: Scope,
    project_root: &Path,
) -> Option<Scope> {
    let other = match scope {
        Scope::Project => Scope::Global,
        Scope::Global => Scope::Project,
    };

    if skill_exists(name, agent, other, project_root) {
        Some(other)
    } else {
        None
    }
}

/// Read basic skill info (name, description) from a skill directory.
fn read_skill_info(skill_dir: &Path) -> Option<(String, String)> {
    let skill_md = skill_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md).ok()?;

    // Parse frontmatter
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return None;
    }

    let frontmatter = parts[1];

    let mut name = None;
    let mut description = None;

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("name:") {
            name = Some(
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
        } else if let Some(value) = line.strip_prefix("description:") {
            description = Some(
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
        }
    }

    match (name, description) {
        (Some(n), Some(d)) => Some((n, d)),
        (Some(n), None) => Some((n, String::new())),
        _ => {
            // Fall back to directory name
            let dir_name = skill_dir.file_name()?.to_str()?.to_string();
            Some((dir_name, String::new()))
        }
    }
}

/// Get the global skills directory, creating it if necessary.
pub fn ensure_global_dir(agent: Agent) -> std::io::Result<PathBuf> {
    let Some(path) = agent.resolve_global_skills_dir() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ));
    };

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

/// Get the project skills directory, creating it if necessary.
pub fn ensure_project_dir(agent: Agent, project_root: &Path) -> std::io::Result<PathBuf> {
    let path = agent.resolve_project_skills_dir(project_root);

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(path)
}

/// Ensure the skills directory exists for the given scope.
pub fn ensure_skills_dir(
    agent: Agent,
    scope: Scope,
    project_root: &Path,
) -> std::io::Result<PathBuf> {
    match scope {
        Scope::Project => ensure_project_dir(agent, project_root),
        Scope::Global => ensure_global_dir(agent),
    }
}
