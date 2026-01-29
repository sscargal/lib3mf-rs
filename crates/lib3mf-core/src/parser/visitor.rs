use crate::error::Result;
use crate::model::{BaseMaterialsGroup, BuildItem, ColorGroup, ResourceId};

/// Trait for receiving callback events during streaming parsing of a 3MF model.
/// This allows for processing massive files with constant memory usage.
pub trait ModelVisitor {
    /// Called when the model element starts.
    fn on_start_model(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the model element ends.
    fn on_end_model(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called upon encountering metadata.
    fn on_metadata(&mut self, _name: &str, _value: &str) -> Result<()> {
        Ok(())
    }

    /// Called when the resources container starts.
    fn on_start_resources(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the resources container ends.
    fn on_end_resources(&mut self) -> Result<()> {
        Ok(())
    }

    // --- Resources ---

    /// Called when a BaseMaterials group is fully parsed.
    /// Since these are typically small, we pass the full object.
    fn on_base_materials(&mut self, _id: ResourceId, _group: &BaseMaterialsGroup) -> Result<()> {
        Ok(())
    }

    /// Called when a ColorGroup is fully parsed.
    fn on_color_group(&mut self, _id: ResourceId, _group: &ColorGroup) -> Result<()> {
        Ok(())
    }

    // --- Mesh (Streaming) ---

    /// Called when a Mesh object starts.
    fn on_start_mesh(&mut self, _id: ResourceId) -> Result<()> {
        Ok(())
    }

    /// Called for each vertex in the current mesh.
    fn on_vertex(&mut self, _x: f32, _y: f32, _z: f32) -> Result<()> {
        Ok(())
    }

    /// Called for each triangle in the current mesh.
    fn on_triangle(&mut self, _v1: u32, _v2: u32, _v3: u32) -> Result<()> {
        Ok(())
    }

    /// Called when a Mesh object ends.
    fn on_end_mesh(&mut self) -> Result<()> {
        Ok(())
    }

    // --- Build ---

    /// Called when the build container starts.
    fn on_start_build(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the build container ends.
    fn on_end_build(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called for each item in the build.
    fn on_build_item(&mut self, _item: &BuildItem) -> Result<()> {
        Ok(())
    }
}
