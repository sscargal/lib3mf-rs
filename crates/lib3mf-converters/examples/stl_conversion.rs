use lib3mf_converters::stl::{StlExporter, StlImporter};
use lib3mf_core::model::{Model, Unit};
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() -> anyhow::Result<()> {
    println!("--- STL Conversion Example ---");

    // 1. Create a simple model to export to STL
    let mut model = Model {
        unit: Unit::Millimeter, // Default but let's be explicit
        ..Default::default()
    };
    let mut mesh = lib3mf_core::model::Mesh::new();

    // Add a simple triangle
    let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v3 = mesh.add_vertex(5.0, 10.0, 0.0);
    mesh.add_triangle(v1, v2, v3);

    let object = lib3mf_core::model::Object {
        id: lib3mf_core::model::ResourceId(1),
        name: Some("Test Triangle".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        thumbnail: None,
        pindex: None,
        object_type: lib3mf_core::model::ObjectType::Model,
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

    // 2. Export to STL
    let stl_path = "triangle.stl";
    println!("Exporting to {}...", stl_path);
    let stl_file = File::create(stl_path)?;
    StlExporter::write(&model, BufWriter::new(stl_file))?;

    // 3. Import back from STL
    println!("Importing back from {}...", stl_path);
    let stl_input = File::open(stl_path)?;
    let imported_model = StlImporter::read(BufReader::new(stl_input))?;

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
