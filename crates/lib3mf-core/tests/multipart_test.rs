use lib3mf_core::archive::{ArchiveReader, ZipArchiver};
use lib3mf_core::parser::model_parser::parse_model;
use std::fs::File;
use std::io::Cursor;

#[test]
fn test_multipart_stats_recursive() {
    let path = "../../models/Benchy.3mf";
    let file = File::open(path).expect("Failed to open Benchy.3mf");
    let mut archiver = ZipArchiver::new(file).expect("Failed to open ZIP");
    
    // Manual discovery of root part (usually fixed for Benchy)
    let root_data = archiver.read_entry("3D/3dmodel.model").expect("Failed to read root model");
    let model = parse_model(Cursor::new(root_data)).expect("Failed to parse model");
    
    let stats = model.compute_stats(&mut archiver).expect("Failed to compute stats");
    
    println!("Benchy Stats: {:?}", stats.geometry);
    
    // Validate against user-provided benchmarks from 3dviewer.net
    // Triangles: 226,654
    // Vertices: 113,331
    assert!(stats.geometry.triangle_count > 220000, "Triangle count too low: {}", stats.geometry.triangle_count);
    assert!(stats.geometry.vertex_count > 110000, "Vertex count too low: {}", stats.geometry.vertex_count);
    
    // Check Bounding Box (approximately matching Size X:45.67 Size Y:68.52 Size Z:54.62)
    if let Some(aabb) = stats.geometry.bounding_box {
        let size_x = aabb.max[0] - aabb.min[0];
        let size_y = aabb.max[1] - aabb.min[1];
        let size_z = aabb.max[2] - aabb.min[2];
        
        assert!((size_x - 45.67).abs() < 2.0, "Size X mismatch: {}", size_x);
        assert!((size_y - 68.52).abs() < 2.0, "Size Y mismatch: {}", size_y);
        assert!((size_z - 54.62).abs() < 2.0, "Size Z mismatch: {}", size_z);
    }
}
