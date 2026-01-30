use lib3mf_async::loader::load_model_async;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("--- Async Model Loading Example ---");

    // We'll try to load Benchy.3mf from the models directory.
    let path = "models/Benchy.3mf";

    if !Path::new(path).exists() {
        println!(
            "Please run this example from the repo root and ensure '{}' exists.",
            path
        );
        return Ok(());
    }

    println!("Loading {} asynchronously using tokio...", path);

    // load_model_async handles opening the file, unzipping (async),
    // and parsing (spawn_blocking) internally.
    let model = load_model_async(path).await?;

    println!("SUCCESS: Model loaded asynchronously.");
    println!("Statistics:");
    println!("  Unit:     {:?}", model.unit);
    println!("  Metadata: {} entries", model.metadata.len());
    println!("  Objects:  {}", model.resources.iter_objects().count());
    println!("  Items:    {}", model.build.items.len());

    Ok(())
}
