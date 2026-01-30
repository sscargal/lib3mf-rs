use lib3mf_async::loader::load_model_async;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let path = "models/Benchy.3mf";

    if !std::path::Path::new(path).exists() {
        println!("Please run this example from the repo root and ensure 'models/Benchy.3mf' exists.");
        return Ok(());
    }

    println!("Asynchronously loading {}...", path);
    
    let start = std::time::Instant::now();
    let model = load_model_async(path).await?;
    let duration = start.elapsed();

    println!("Successfully loaded model in {:?}", duration);
    println!("  Unit: {:?}", model.unit);
    println!("  Objects: {}", model.resources.iter_objects().count());
    println!("  Build Items: {}", model.build.items.len());

    Ok(())
}
