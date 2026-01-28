pub mod archive;
pub mod error;
pub mod model;
pub mod parser;
pub mod writer;
pub mod validation;

pub use error::{Lib3mfError, Result};
pub use model::*;

#[cfg(test)]
mod tests {
    use super::*;
    // use glam::Vec3; // Removed unused import


    #[test]
    fn test_model_default() {
        let model = Model::default();
        assert_eq!(model.unit, Unit::Millimeter);
        assert!(model.metadata.is_empty());
    }

    #[test]
    fn test_mesh_construction() {
        let mut mesh = Mesh::new();
        let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
        let v2 = mesh.add_vertex(1.0, 0.0, 0.0);
        let v3 = mesh.add_vertex(0.0, 1.0, 0.0);
        
        mesh.add_triangle(v1, v2, v3);
        
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangles.len(), 1);
        
        let t = &mesh.triangles[0];
        assert_eq!(t.v1, 0);
        assert_eq!(t.v2, 1);
        assert_eq!(t.v3, 2);
    }

    #[test]
    fn test_resource_collection() {
        let mut resources = ResourceCollection::new();
        let mesh = Mesh::new();
        let object = Object {
            id: ResourceId(1),
            name: Some("Test Object".to_string()),
            part_number: None,
            geometry: Geometry::Mesh(mesh),
        };

        assert!(resources.add_object(object.clone()).is_ok());
        assert!(resources.add_object(object).is_err()); // Duplicate ID
        
        assert!(resources.get_object(ResourceId(1)).is_some());
        assert!(resources.get_object(ResourceId(2)).is_none());
    }
}
