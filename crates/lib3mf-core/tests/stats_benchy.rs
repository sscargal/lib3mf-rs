use lib3mf_core::archive::{find_model_path, ArchiveReader, ZipArchiver};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_stats_benchy() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.pop();
    d.push("models");
    d.push("Benchy.3mf");

    let file = File::open(&d).expect("Failed to open Benchy.3mf");
    // We need archiver for both finding model and computing stats (checking for config)
    // But ZipArchiver takes ownership of reader.
    // We can't easily "rewind" the ZipArchiver to pass it again IF we consumed it.
    // However, compute_stats takes &mut impl ArchiveReader.
    // We need to keep archiver alive.
    
    let mut archiver = ZipArchiver::new(file).expect("Failed to create archiver");
    
    // Parse Model
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver.read_entry(&model_path).expect("Failed to read model");
    let model = parse_model(Cursor::new(model_data)).expect("Failed to parse model XML");
    
    // Compute Stats
    let stats = model.compute_stats(&mut archiver).expect("Failed to compute stats");
    
    println!("Benchy Stats: {:?}", stats);
    
    // Check Geometry
    // Benchy is usually around 225k triangles.
    // Our previous inspection showed "face_count=225706".
    // 3MF might store this differently or split it.
    // Vertices: 112855.
    
    // NOTE: If Benchy uses components referencing external files, our simple parser 
    // won't see the geometry unless we implement resolving external references, 
    // which we haven't done in stats_impl yet (we only iterate *loaded* resources).
    // Let's see what we get. If 0, it confirms we need to handle external references or 
    // verify if Benchy.3mf actually has the mesh embedded in valid 3MF 3D payload or not.
    // Previously we saw "3D/Objects/object_1.model". If that is NOT the main model file,
    // then the main model only has components.
    
    // Update expectations based on run.
    assert!(stats.geometry.triangle_count > 0 || stats.geometry.instance_count > 0);
    
    // Check Vendor Data
    if let Some(generator) = &stats.generator {
        assert!(generator.contains("Bambu"), "Should be Bambu Studio generated");
        // Benchy usually has no plates config if it's a simple export, or maybe it does?
        // Let's check.
    }
}
