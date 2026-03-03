use crate::model::Model;
use std::collections::HashMap;

/// Represents a 3MF Package, which can contain multiple model parts.
#[derive(Debug, Clone, Default)]
pub struct Package {
    /// The main model part (usually /3D/3dmodel.model).
    pub main_model: Model,

    /// Additional model parts keyed by their package path.
    pub parts: HashMap<String, Model>,
}

impl Package {
    /// Creates a new `Package` with the given main model and no additional parts.
    pub fn new(main_model: Model) -> Self {
        Self {
            main_model,
            parts: HashMap::new(),
        }
    }

    /// Adds an additional model part to the package, keyed by its package path.
    pub fn add_part(&mut self, path: String, model: Model) {
        self.parts.insert(path, model);
    }
}
