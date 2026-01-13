mod body_length;
mod compatibility;
mod description;
mod name;
mod references;
mod scripts;

pub use body_length::BodyLengthRule;
pub use compatibility::CompatibilityLengthRule;
pub use description::{DescriptionLengthRule, DescriptionRequiredRule};
pub use name::{NameDirectoryRule, NameFormatRule, NameLengthRule};
pub use references::ReferencesExistRule;
pub use scripts::{ScriptExecutableRule, ScriptShebangRule};

use crate::skill::manifest::Manifest;
use crate::skill::validator::Diagnostic;

/// A lint rule that checks a manifest for issues.
pub trait Rule: Send + Sync {
    /// Human-readable name for this rule (e.g., "name-format")
    fn name(&self) -> &'static str;

    /// Check the manifest and return any diagnostics found.
    fn check(&self, manifest: &Manifest) -> Vec<Diagnostic>;
}
