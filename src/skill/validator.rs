use crate::config::LintConfig;
use crate::skill::manifest::Manifest;
use crate::skill::rules::{
    BodyLengthRule, CompatibilityLengthRule, DescriptionLengthRule, DescriptionRequiredRule,
    NameDirectoryRule, NameFormatRule, NameLengthRule, ReferencesExistRule, Rule,
    ScriptExecutableRule, ScriptShebangRule,
};

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_ok_strict(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: DiagnosticCode,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    // Errors
    E001, // Invalid name format
    E002, // Name too long
    E003, // Name mismatch with directory
    E004, // Missing description
    E005, // Description too long
    E006, // Compatibility too long
    E007, // Invalid YAML
    E008, // Missing SKILL.md
    E009, // Referenced file not found

    // Warnings
    W001, // Body exceeds max lines
    W002, // Script not executable
    W003, // Script missing shebang
    W004, // Empty optional directory
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E001 => write!(f, "E001"),
            Self::E002 => write!(f, "E002"),
            Self::E003 => write!(f, "E003"),
            Self::E004 => write!(f, "E004"),
            Self::E005 => write!(f, "E005"),
            Self::E006 => write!(f, "E006"),
            Self::E007 => write!(f, "E007"),
            Self::E008 => write!(f, "E008"),
            Self::E009 => write!(f, "E009"),
            Self::W001 => write!(f, "W001"),
            Self::W002 => write!(f, "W002"),
            Self::W003 => write!(f, "W003"),
            Self::W004 => write!(f, "W004"),
        }
    }
}

impl DiagnosticCode {
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::E001
                | Self::E002
                | Self::E003
                | Self::E004
                | Self::E005
                | Self::E006
                | Self::E007
                | Self::E008
                | Self::E009
        )
    }
}

pub struct Validator {
    rules: Vec<Box<dyn Rule>>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new(&LintConfig::default())
    }
}

impl Validator {
    pub fn new(config: &LintConfig) -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();

        if config.rules.name_format {
            rules.push(Box::new(NameFormatRule));
        }
        if let Some(max) = config.rules.name_length.resolve(64) {
            rules.push(Box::new(NameLengthRule::new(max)));
        }
        if config.rules.name_directory {
            rules.push(Box::new(NameDirectoryRule));
        }
        if config.rules.description_required {
            rules.push(Box::new(DescriptionRequiredRule));
        }
        if let Some(max) = config.rules.description_length.resolve(1024) {
            rules.push(Box::new(DescriptionLengthRule::new(max)));
        }
        if let Some(max) = config.rules.compatibility_length.resolve(500) {
            rules.push(Box::new(CompatibilityLengthRule::new(max)));
        }
        if config.rules.references_exist {
            rules.push(Box::new(ReferencesExistRule));
        }
        if let Some(max) = config.rules.body_length.resolve(500) {
            rules.push(Box::new(BodyLengthRule::new(max)));
        }
        if config.rules.script_executable {
            rules.push(Box::new(ScriptExecutableRule));
        }
        if config.rules.script_shebang {
            rules.push(Box::new(ScriptShebangRule));
        }

        Self { rules }
    }

    pub fn validate(&self, manifest: &Manifest) -> ValidationResult {
        let mut result = ValidationResult::default();

        for rule in &self.rules {
            let diagnostics = rule.check(manifest);
            for diag in diagnostics {
                if diag.code.is_error() {
                    result.errors.push(diag);
                } else {
                    result.warnings.push(diag);
                }
            }
        }

        result
    }
}
