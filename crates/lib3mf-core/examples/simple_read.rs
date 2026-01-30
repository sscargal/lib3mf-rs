use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let path = "models/Benchy.3mf";

    if !std::path::Path::new(path).exists() {
        println!(
            "Please run this example from the repo root and ensure 'models/Benchy.3mf' exists."
        );
        return Ok(());
    }

    println!("Opening {}...", path);
    let file = File::open(path)?;
    let mut archiver = ZipArchiver::new(file)?;

    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;

    let model = parse_model(std::io::Cursor::new(model_data))?;
    let stats = model.compute_stats(&mut archiver)?;

    println!("Model Statistics:");
    println!("  Unit: {:?}", model.unit);
    println!("  Objects: {}", stats.geometry.object_count);
    println!("  Build Items: {}", stats.geometry.instance_count);
    println!("  Triangles: {}", stats.geometry.triangle_count);
    println!("  Vertices: {}", stats.geometry.vertex_count);

    if stats.geometry.object_count > 0 && stats.geometry.triangle_count == 0 {
        println!(
            "\nNote: This model appears to use components. Vertex/Triangle counts currently only reflect unique mesh geometry."
        );
    }

    Ok(())
}
