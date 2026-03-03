use serde::{Deserialize, Serialize};

/// Severity level of a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// A spec violation or structural error that prevents correct interpretation.
    Error,
    /// A non-fatal issue that may affect interoperability or quality.
    Warning,
    /// Informational note (advisory, not a problem).
    Info,
}

/// A single validation finding with severity, code, message, and optional context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationItem {
    /// Severity level of this finding.
    pub severity: ValidationSeverity,
    /// Unique numeric code identifying the type of validation failure.
    pub code: u32, // Unique error code
    /// Human-readable description of the issue.
    pub message: String,
    /// Optional suggested fix or remediation.
    pub suggestion: Option<String>,
    /// Optional context string identifying which object/resource is affected (e.g., `"Object 5"`).
    pub context: Option<String>, // e.g., "Object 5"
}

/// Collection of validation findings produced by `Model::validate()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    /// All findings from the validation run.
    pub items: Vec<ValidationItem>,
}

impl ValidationReport {
    /// Creates a new empty `ValidationReport`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an error-severity finding to the report.
    pub fn add_error(&mut self, code: u32, msg: impl Into<String>) {
        self.items.push(ValidationItem {
            severity: ValidationSeverity::Error,
            code,
            message: msg.into(),
            suggestion: None,
            context: None,
        });
    }

    /// Adds a warning-severity finding to the report.
    pub fn add_warning(&mut self, code: u32, msg: impl Into<String>) {
        self.items.push(ValidationItem {
            severity: ValidationSeverity::Warning,
            code,
            message: msg.into(),
            suggestion: None,
            context: None,
        });
    }

    /// Adds an info-severity finding to the report.
    pub fn add_info(&mut self, code: u32, msg: impl Into<String>) {
        self.items.push(ValidationItem {
            severity: ValidationSeverity::Info,
            code,
            message: msg.into(),
            suggestion: None,
            context: None,
        });
    }

    /// Returns `true` if the report contains any error-severity findings.
    pub fn has_errors(&self) -> bool {
        self.items
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }
}
