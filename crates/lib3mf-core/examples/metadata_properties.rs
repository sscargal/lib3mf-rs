use lib3mf_core::model::{Model, Unit};
use lib3mf_core::writer::package_writer::PackageWriter;
use std::fs::File;

fn main() -> anyhow::Result<()> {
    println!("Creating model with metadata...");

    let mut model = Model {
        unit: Unit::Micron,
        language: Some("en-US".to_string()),
        ..Default::default()
    };

    // Standard Metadata (Dublin Core)
    model
        .metadata
        .insert("Title".to_string(), "Metadata Example".to_string());
    model
        .metadata
        .insert("Designer".to_string(), "Lib3mf User".to_string());
    model.metadata.insert(
        "Description".to_string(),
        "Demonstrates metadata handling".to_string(),
    );
    model
        .metadata
        .insert("License".to_string(), "MIT".to_string());
    model
        .metadata
        .insert("Copyright".to_string(), "2024".to_string());

    // Custom Metadata
    // Keys usually namespaced, e.g. "namespace:key"
    model
        .metadata
        .insert("custom:version".to_string(), "1.0.0-beta".to_string());
    model
        .metadata
        .insert("custom:category".to_string(), "Prototypes".to_string());

    // Write to file
    let file = File::create("metadata.3mf")?;
    let writer = PackageWriter::new(file);
    writer.write(&model)?;

    println!("Written to metadata.3mf");

    // Demonstrate reading back (sanity check)
    // Note: To read back we would use Reader, but here we just show creation.
    println!("Metadata count: {}", model.metadata.len());

    Ok(())
}
