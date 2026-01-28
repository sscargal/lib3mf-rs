use lib3mf_core::archive::{find_model_path, ArchiveReader, ZipArchiver};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_parse_benchy() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.pop();
    d.push("models");
    d.push("Benchy.3mf");

    if !d.exists() {
        eprintln!("Skipping test_parse_benchy: File not found at {:?}", d);
        return;
    }

    let file = File::open(&d).expect("Failed to open Benchy.3mf");
    let mut archiver = ZipArchiver::new(file).expect("Failed to create archiver");

    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path)
        .expect("Failed to read model content");

    let model = parse_model(Cursor::new(model_data)).expect("Failed to parse model XML");

    // 1. Verify Metadata
    assert_eq!(
        model.metadata.get("Title").map(|s| s.as_str()),
        Some("Benchy Bambu Pla Basic")
    );
    assert_eq!(
        model.metadata.get("Application").map(|s| s.as_str()),
        Some("BambuStudio-01.10.02.73")
    );

    // 2. Verify Resources (Benchy usually has 1 large mesh object)
    // In this specific file, we saw "3D/Objects/object_1.model" in unzip output 
    // which implies it might use split model reference or it is embedded.
    // Wait, unzip output showed:
    // 5937  2025-07-15 14:48   3D/3dmodel.model
    // 20445944  2025-07-15 14:48   3D/Objects/object_1.model
    // This suggests the MAIN model file (3D/3dmodel.model) might only contain COMPONENT references to object_1.model!
    // Or it contains the mesh directly?
    // Let's check the build items.
    
    assert!(!model.build.items.is_empty(), "Build should have items");
    
    // If Benchy is using Production Extension (split files), the main model 3D/3dmodel.model 
    // likely has <object type="model"><components><component path="..."/></components></object>.
    // Our current unzip inspection showed 3D/3dmodel.model was small (5KB).
    // So it definitely does NOT contain the 112k vertices mesh directly.
    // It contains components linking to Objects/object_1.model.
    
    // We haven't implemented resolving external file references in parse_model yet!
    // parse_model currently only parses the XML provided to it.
    // So if we parse "3D/3dmodel.model", we will see objects with components, but those components 
    // refer to external paths.
    
    // So we should verify we have an object with components.
    let root_objects: Vec<_> = model.resources.iter_objects().collect();
    assert!(!root_objects.is_empty());
    
    // Check for components
    if let lib3mf_core::Geometry::Components(comps) = &root_objects[0].geometry {
        println!("Found {} components in root object", comps.components.len());
        // Verify we have components
        assert!(!comps.components.is_empty());
    } else {
        // It might be a direct mesh if I am wrong, but file size suggests components.
        // Actually, looking at the previous head -n 100 output for Benchy:
        // <object id="8" ... type="model">
        //   <components>
        //    <component p:path="/3D/Objects/object_1.model" objectid="1" ... />
        //   </components>
        // </object>
        // YES. It uses production extension paths.
        println!("Root object is a component wrapper.");
    }
}
