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

    println!("Model: Benchy.3mf");
    println!("  Unit: {:?}", model.unit);
    println!("  Triangles: {}", stats.geometry.triangle_count);
    println!("  Vertices: {}", stats.geometry.vertex_count);

    Ok(())
}
