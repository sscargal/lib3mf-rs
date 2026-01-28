use lib3mf_core::archive::{find_model_path, ArchiveReader, ZipArchiver};
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
    
    let stats_orig = model.compute_stats(&mut archiver)?;
    
    // 2. Write to memory buffer
    let mut buffer = Cursor::new(Vec::new());
    model.write(&mut buffer)?;
    
    // 3. Read back from memory buffer
    let mut buffer_archiver = ZipArchiver::new(Cursor::new(buffer.into_inner()))?;
    let model_path_new = find_model_path(&mut buffer_archiver)?;
    let model_data_new = buffer_archiver.read_entry(&model_path_new)?;
    let model_new = parse_model(Cursor::new(model_data_new))?;
    
    let stats_new = model_new.compute_stats(&mut buffer_archiver)?;

    // 4. Compare
    println!("ORIG Stats: Tri: {}, Vert: {}", stats_orig.geometry.triangle_count, stats_orig.geometry.vertex_count);
    println!("NEW Stats: Tri: {}, Vert: {}", stats_new.geometry.triangle_count, stats_new.geometry.vertex_count);

    assert_eq!(stats_orig.geometry.object_count, stats_new.geometry.object_count);
    assert_eq!(stats_orig.geometry.instance_count, stats_new.geometry.instance_count);
    assert_eq!(stats_orig.geometry.triangle_count, stats_new.geometry.triangle_count); // Both should be 0 for this file
    assert_eq!(stats_new.geometry.instance_count, 1);

    Ok(())
}
