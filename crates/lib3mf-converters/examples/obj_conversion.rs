use lib3mf_converters::obj::{ObjExporter, ObjImporter};
use lib3mf_core::model::{Model, Unit};
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() -> anyhow::Result<()> {
    println!("--- OBJ Conversion Example ---");

    // 1. Create a simple model to export to OBJ
    let mut model = Model {
        unit: Unit::Millimeter, // Default but let's be explicit
        ..Default::default()
    };
    let mut mesh = lib3mf_core::model::Mesh::new();

    // Add a simple quad (2 triangles)
    let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v3 = mesh.add_vertex(10.0, 10.0, 0.0);
    let v4 = mesh.add_vertex(0.0, 10.0, 0.0);
    mesh.add_triangle(v1, v2, v3);
    mesh.add_triangle(v1, v3, v4);

    let object = lib3mf_core::model::Object {
        id: lib3mf_core::model::ResourceId(1),
        name: Some("Test Quad".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        thumbnail: None,
        pindex: None,
        geometry: lib3mf_core::model::Geometry::Mesh(mesh),
    };
    model.resources.add_object(object)?;

    model.build.items.push(lib3mf_core::model::BuildItem {
        object_id: lib3mf_core::model::ResourceId(1),
        uuid: None,
        path: None,
        part_number: None,
        transform: glam::Mat4::IDENTITY,
    });

    // 2. Export to OBJ
    let obj_path = "quad.obj";
    println!("Exporting to {}...", obj_path);
    let obj_file = File::create(obj_path)?;
    ObjExporter::write(&model, BufWriter::new(obj_file))?;

    // 3. Import back from OBJ
    println!("Importing back from {}...", obj_path);
    let obj_input = File::open(obj_path)?;
    let imported_model = ObjImporter::read(BufReader::new(obj_input))?;

    println!("Imported model stats:");
    for object in imported_model.resources.iter_objects() {
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &object.geometry {
            println!("  Object ID: {}", object.id.0);
            println!("  Triangles: {}", mesh.triangles.len());
            println!("  Vertices:  {}", mesh.vertices.len());
        }
    }

    Ok(())
}
