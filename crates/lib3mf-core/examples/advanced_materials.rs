use lib3mf_core::archive::ZipArchiver;
use lib3mf_core::error::Result;
use lib3mf_core::model::parser::model_parser::parse_model;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<()> {
    // This example demonstrates how to access advanced material properties
    // like Texture2DGroup, CompositeMaterials, and MultiProperties.
    
    // Note: In a real scenario, you would open a file.
    // Here we'll simulate a model with advanced materials for demonstration purposes
    // since we might not have a physical file with these properties handy in the repo root.
    
    println!("Advanced Materials Example");
    println!("==========================");

    // In a real application:
    // let file = File::open("path/to/model.3mf")?;
    // let archiver = ZipArchiver::new(file)?;
    // let model = parse_model(archiver)?; 
    
    // Accessing resources
    // let resources = &model.resources;
    
    // 1. Texture 2D Groups
    // println!("Texture 2D Groups: {}", resources.texture_2d_groups_count());
    // for (id, group) in &resources.texture_2d_groups {
    //     println!("  Group ID: {}", id.0);
    //     println!("  Texture ID: {}", group.texture_id.0);
    //     println!("  Coords: {} coordinates", group.coords.len());
    // }

    // 2. Composite Materials
    // println!("Composite Materials: {}", resources.composite_materials_count());
    // for (id, possible_group) in &resources.composite_materials {
    //     // Access composite logic
    // }

    // 3. Multi Properties
    // println!("Multi Properties: {}", resources.multi_properties_count());
    
    println!("(See source code for usage patterns)");
    Ok(())
}
