use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;

#[test]
fn test_roundtrip_benchy() -> anyhow::Result<()> {
    // 1. Read Original
    let file = File::open("../../models/Benchy.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(Cursor::new(model_data))?;

    let _stats_orig = model.compute_stats(&mut archiver)?;

    // 2. Write to memory buffer
    let mut buffer = Cursor::new(Vec::new());
    model.write(&mut buffer)?;

    // 3. Read back from memory buffer
    let mut buffer_archiver = ZipArchiver::new(Cursor::new(buffer.into_inner()))?;
    let model_path_new = find_model_path(&mut buffer_archiver)?;
    let model_data_new = buffer_archiver.read_entry(&model_path_new)?;
    let model_new = parse_model(Cursor::new(model_data_new))?;

    // 4. Compare
    println!(
        "ORIG Objects: {}, Items: {}",
        model.resources.iter_objects().count(),
        model.build.items.len()
    );
    println!(
        "NEW Objects: {}, Items: {}",
        model_new.resources.iter_objects().count(),
        model_new.build.items.len()
    );

    // Verify root part structure
    assert_eq!(
        model_new.resources.iter_objects().count(),
        model.resources.iter_objects().count()
    );
    assert_eq!(model_new.build.items.len(), model.build.items.len());

    // Validate that the root object (ID 8) was persisted correctly
    let obj8 = model_new
        .resources
        .get_object(lib3mf_core::model::ResourceId(8))
        .expect("Object 8 missing");

    // Verify Production Extension UUID was persisted
    if let Some(uuid) = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(8))
        .and_then(|o| o.uuid)
    {
        assert_eq!(obj8.uuid, Some(uuid));
    }

    Ok(())
}
