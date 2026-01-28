use crate::model::Model;
use crate::validation::report::ValidationReport;

pub fn validate_schema(_model: &Model, _report: &mut ValidationReport) {
    // Basic schema checks that aren't caught by parser
}
