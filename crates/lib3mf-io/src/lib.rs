pub mod stl;
pub mod obj;

use lib3mf_core::error::Result;
use lib3mf_core::model::Model;

/// Trait for importing a model from a format.
pub trait ModelImporter {
    fn import<R: std::io::Read>(&self, reader: R) -> Result<Model>;
}

/// Trait for exporting a model to a format.
pub trait ModelExporter {
    fn export<W: std::io::Write>(&self, model: &Model, writer: W) -> Result<()>;
}
