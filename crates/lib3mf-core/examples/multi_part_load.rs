use lib3mf_core::archive::model_locator::find_model_path;
use lib3mf_core::archive::{ArchiveReader, ZipArchiver};
use lib3mf_core::model::resolver::PartResolver;
use lib3mf_core::parser::model_parser::parse_model;
use std::fs::File;
use std::io::Cursor;

fn main() -> anyhow::Result<()> {
    let path = "models/Benchy.3mf";
    println!("Loading multi-part model: {}", path);

    let file = File::open(path)?;
    let mut archiver = ZipArchiver::new(file)?;

    // 1. Find root model path
    let model_path = find_model_path(&mut archiver)?;
    println!("Root model path: {}", model_path);

    // 2. Parse root model
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(Cursor::new(model_data))?;

    // 3. Initialize Resolver
    let mut resolver = PartResolver::new(&mut archiver, model);

    // 4. Explore Build Items
    // We clone the items to avoid holding a borrow to the root model while using the resolver
    let items: Vec<_> = resolver.get_root_model().build.items.clone();
    println!("Build Items: {}", items.len());

    for (i, item) in items.iter().enumerate() {
        println!("Item {}: Object ID {}", i, item.object_id.0);

        // Resolve object (it might be in another part)
        match resolver.resolve_object(item.object_id, None)? {
            Some((_model, obj)) => {
                println!("  Resolved: name={:?}, id={}", obj.name, obj.id.0);
                match &obj.geometry {
                    lib3mf_core::model::Geometry::Mesh(mesh) => {
                        println!(
                            "  Mesh: {} vertices, {} triangles",
                            mesh.vertices.len(),
                            mesh.triangles.len()
                        );
                    }
                    lib3mf_core::model::Geometry::Components(comps) => {
                        println!("  Components: {}", comps.components.len());
                        let comps_list = comps.components.clone();
                        for (j, comp) in comps_list.iter().enumerate() {
                            println!(
                                "    Comp {}: Object ID {}, path={:?}",
                                j, comp.object_id.0, comp.path
                            );
                            // Resolve nested component
                            if let Some((_sub_model, sub_obj)) =
                                resolver.resolve_object(comp.object_id, comp.path.as_deref())?
                            {
                                println!(
                                    "      Resolved nested: name={:?}, id={}",
                                    sub_obj.name, sub_obj.id.0
                                );
                            }
                        }
                    }
                    _ => println!("  Other geometry type"),
                }
            }
            None => println!("  Failed to resolve object!"),
        }
    }

    Ok(())
}
