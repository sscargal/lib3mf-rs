use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationItem {
    pub severity: ValidationSeverity,
    pub code: u32, // Unique error code
    pub message: String,
    pub suggestion: Option<String>,
    pub context: Option<String>, // e.g., "Object 5"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    pub items: Vec<ValidationItem>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, code: u32, msg: impl Into<String>) {
        self.items.push(ValidationItem {
            severity: ValidationSeverity::Error,
            code,
            message: msg.into(),
            suggestion: None,
            context: None,
        });
    }

    pub fn add_warning(&mut self, code: u32, msg: impl Into<String>) {
        self.items.push(ValidationItem {
            severity: ValidationSeverity::Warning,
            code,
            message: msg.into(),
            suggestion: None,
            context: None,
        });
    }

    pub fn has_errors(&self) -> bool {
        self.items
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }
}
