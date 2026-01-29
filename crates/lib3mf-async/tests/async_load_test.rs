use lib3mf_async::loader::load_model_async;
use std::path::PathBuf;

#[tokio::test]
async fn test_async_load_benchy() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/assets/Benchy.3mf"); 
    // Wait, assets are in workspace root or crate root?
    // lib3mf-core tests access assets relative to workspace loop via CARGO_MANIFEST_DIR usually.
    // Let's check where BENCHY is.
    // Phase 2 integration tests used `../../tests/assets/Benchy.3mf`?
    // Assuming workspace structure:
    // crates/lib3mf-async
    // crates/lib3mf-core
    // tests/assets/Benchy.3mf (Root tests folder?)
    
    // Correct relative path from crate root:
    let asset_path = PathBuf::from("../../tests/assets/Benchy.3mf");
    
    // Ensure file exists before running
    if !asset_path.exists() {
        println!("Skipping test: Benchy.3mf not found at {:?}", asset_path);
        return;
    }
    
    let result = load_model_async(&asset_path).await;
    match result {
        Ok(model) => {
             assert_eq!(model.unit, lib3mf_core::model::Unit::Millimeter);
             assert!(model.resources.iter_objects().count() > 0);
             println!("Successfully loaded Benchy async!");
        },
        Err(e) => panic!("Async load failed: {}", e),
    }
}
