use super::OutputFormatter;
use crate::skill::{DiagnosticCode, ValidationResult};
use serde::Serialize;

pub struct SarifFormatter {
    quiet: bool,
}

impl SarifFormatter {
    pub fn new(quiet: bool) -> Self {
        Self { quiet }
    }
}

#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifConfiguration,
}

#[derive(Serialize)]
struct SarifConfiguration {
    level: &'static str,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn", skip_serializing_if = "Option::is_none")]
    start_column: Option<usize>,
}

fn get_rule_description(code: DiagnosticCode) -> &'static str {
    match code {
        DiagnosticCode::E001 => "Invalid skill name format",
        DiagnosticCode::E002 => "Skill name exceeds maximum length",
        DiagnosticCode::E003 => "Skill name does not match directory name",
        DiagnosticCode::E004 => "Missing skill description",
        DiagnosticCode::E005 => "Skill description exceeds maximum length",
        DiagnosticCode::E006 => "Compatibility field exceeds maximum length",
        DiagnosticCode::E007 => "Invalid YAML in frontmatter",
        DiagnosticCode::E008 => "Missing SKILL.md file",
        DiagnosticCode::E009 => "Referenced file not found",
        DiagnosticCode::W001 => "Skill body exceeds recommended length",
        DiagnosticCode::W002 => "Script is not executable",
        DiagnosticCode::W003 => "Script missing shebang line",
        DiagnosticCode::W004 => "Empty optional directory",
    }
}

impl OutputFormatter for SarifFormatter {
    fn format_validation(&self, results: &[(String, ValidationResult)]) -> String {
        // Collect all unique rules
        let mut rules: Vec<SarifRule> = Vec::new();
        let mut seen_codes = std::collections::HashSet::new();

        for (_, result) in results {
            for diag in result.errors.iter().chain(result.warnings.iter()) {
                if seen_codes.insert(diag.code) {
                    rules.push(SarifRule {
                        id: diag.code.to_string(),
                        short_description: SarifMessage {
                            text: get_rule_description(diag.code).to_string(),
                        },
                        default_configuration: SarifConfiguration {
                            level: if diag.code.is_error() {
                                "error"
                            } else {
                                "warning"
                            },
                        },
                    });
                }
            }
        }

        // Collect all results
        let mut sarif_results: Vec<SarifResult> = Vec::new();

        for (path, result) in results {
            for diag in result.errors.iter().chain(result.warnings.iter()) {
                sarif_results.push(SarifResult {
                    rule_id: diag.code.to_string(),
                    level: if diag.code.is_error() {
                        "error"
                    } else {
                        "warning"
                    },
                    message: SarifMessage {
                        text: diag.message.clone(),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation { uri: path.clone() },
                            region: diag.line.map(|line| SarifRegion {
                                start_line: line,
                                start_column: diag.column,
                            }),
                        },
                    }],
                });
            }
        }

        let log = SarifLog {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
            version: "2.1.0",
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "skilo",
                        version: env!("CARGO_PKG_VERSION"),
                        information_uri: "https://github.com/example/skilo",
                        rules,
                    },
                },
                results: sarif_results,
            }],
        };

        serde_json::to_string_pretty(&log).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_message(&self, message: &str) {
        if !self.quiet {
            eprintln!("{}", message);
        }
    }

    fn format_error(&self, message: &str) {
        eprintln!("error: {}", message);
    }

    fn format_success(&self, message: &str) {
        if !self.quiet {
            eprintln!("{}", message);
        }
    }
}
