use crate::archive::ArchiveReader;
use crate::error::Result;
use crate::model::{Model, Object, ResourceId};
use crate::parser::model_parser::parse_model;
use std::collections::HashMap;
use std::io::Cursor;

const ROOT_PATH: &str = "ROOT";
const MAIN_MODEL_PART: &str = "3D/3dmodel.model";

/// Resolves resources across multiple model parts in a 3MF package.
pub struct PartResolver<'a, A: ArchiveReader> {
    archive: &'a mut A,
    models: HashMap<String, Model>,
}

impl<'a, A: ArchiveReader> PartResolver<'a, A> {
    pub fn new(archive: &'a mut A, root_model: Model) -> Self {
        let mut models = HashMap::new();
        models.insert(ROOT_PATH.to_string(), root_model);
        Self { archive, models }
    }

    pub fn resolve_object(
        &mut self,
        id: ResourceId,
        path: Option<&str>,
    ) -> Result<Option<(&Model, &Object)>> {
        let part_path = match path {
            Some(p) => {
                let p = p.trim_start_matches('/');
                if p.is_empty() || p.eq_ignore_ascii_case(MAIN_MODEL_PART) {
                    ROOT_PATH
                } else {
                    p
                }
            }
            None => ROOT_PATH,
        };

        if !self.models.contains_key(part_path) {
            // Try normalized version
            let data = self.archive.read_entry(part_path).or_else(|_| {
                let alt = format!("/{}", part_path);
                self.archive.read_entry(&alt)
            })?;

            let model = parse_model(Cursor::new(data))?;
            self.models.insert(part_path.to_string(), model);
        }

        let model = self.models.get(part_path).unwrap();
        Ok(model.resources.get_object(id).map(|obj| (model, obj)))
    }

    pub fn get_root_model(&self) -> &Model {
        self.models.get("ROOT").unwrap()
    }

    pub fn archive_mut(&mut self) -> &mut A {
        self.archive
    }
}
