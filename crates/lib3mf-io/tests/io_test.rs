use lib3mf_core::model::{Model, Mesh, Vertex, Triangle, BuildItem};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_io::stl::{StlImporter, StlExporter};
use lib3mf_io::obj::{ObjImporter, ObjExporter};
use std::io::Cursor;

fn create_cube_model() -> Model {
    let mut mesh = Mesh::default();
    
    // Simple cube-like shape (One triangle for simplicity)
    let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v3 = mesh.add_vertex(0.0, 10.0, 0.0);
    mesh.add_triangle(v1, v2, v3);
    
    let mut model = Model::default();
    let id = ResourceId(1);
    
    let object = lib3mf_core::model::Object {
        id,
        name: Some("TestCube".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
        geometry: lib3mf_core::model::Geometry::Mesh(mesh),
    };
    
    let _ = model.resources.add_object(object);
    
    model.build.items.push(BuildItem {
        object_id: id,
        transform: glam::Mat4::IDENTITY,
        part_number: None,
        uuid: None,
        path: None,
    });
    
    model
}

#[test]
fn test_stl_roundtrip() {
    let original = create_cube_model();
    
    // Write to memory
    let mut buffer = Vec::new();
    StlExporter::write(&original, &mut buffer).expect("STL export failed");
    
    // Verify Header size
    assert!(buffer.len() > 84); // 80 header + 4 count + 50 bytes per triangle
    
    // Read back
    let cursor = Cursor::new(buffer);
    let imported = StlImporter::read(cursor).expect("STL import failed");
    
    // Verify
    assert_eq!(imported.build.items.len(), 1);
    
    let obj = imported.resources.get_object(imported.build.items[0].object_id).unwrap();
    if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
        assert_eq!(mesh.triangles.len(), 1);
        // Vertices might be 3 (since loose triangle)
        assert_eq!(mesh.vertices.len(), 3);
    } else {
        panic!("Imported object is not a mesh");
    }
}

#[test]
fn test_obj_roundtrip() {
    let original = create_cube_model();
    
    // Write
    let mut buffer = Vec::new();
    ObjExporter::write(&original, &mut buffer).expect("OBJ export failed");
    
    let s = String::from_utf8(buffer.clone()).unwrap();
    println!("Exported OBJ:\n{}", s);
    assert!(s.contains("v 0 0 0"));
    assert!(s.contains("f 1 2 3"));
    
    // Read
    let cursor = Cursor::new(buffer);
    let imported = ObjImporter::read(cursor).expect("OBJ import failed");
    
    let obj = imported.resources.get_object(imported.build.items[0].object_id).unwrap();
    if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
        assert_eq!(mesh.triangles.len(), 1);
        assert_eq!(mesh.vertices.len(), 3);
    } else {
         panic!("Imported object is not a mesh");
    }
}
