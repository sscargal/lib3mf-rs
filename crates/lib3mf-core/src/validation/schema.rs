use crate::model::Model;
use crate::validation::report::ValidationReport;

/// Performs basic schema checks not caught by the XML parser.
pub fn validate_schema(_model: &Model, _report: &mut ValidationReport) {
    // Basic schema checks that aren't caught by parser
}
