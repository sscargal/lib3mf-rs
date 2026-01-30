// Note: To use lib3mf-cli as a library, we might need to expose its commands.
// Currently, most logic is in crates/lib3mf-cli/src/commands.rs which is a private mod.
// Let's see if we can expose it.
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("--- Programmatic CLI Usage Example ---");

    // In this example, we demonstrate how one might wrap the CLI logic
    // to build custom tools or automated pipelines.

    let model_path = PathBuf::from("models/Benchy.3mf");
    if !model_path.exists() {
        println!("Please run this from repo root and ensure 'models/Benchy.3mf' exists.");
        return Ok(());
    }

    // Since commands are currently mod private in main.rs,
    // we would ideally refactor cli to have a pub lib.rs.
    // For now, this example serves as a template for what that would look like.

    /*
    use lib3mf_cli::commands::{stats, OutputFormat};
    stats(model_path, OutputFormat::Text)?;
    */

    println!("Feature: CLI logic is being refactored for library use.");
    println!("To run the CLI normally:");
    println!("  $ cargo run -p lib3mf-cli -- stats models/Benchy.3mf");

    Ok(())
}
