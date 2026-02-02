//! Tests for ObjectType enum and type-specific behavior

use lib3mf_core::model::{Geometry, Mesh, Object, ObjectType, ResourceId};
use lib3mf_core::parser::parse_model;

#[test]
fn test_object_type_default() {
    let obj_type = ObjectType::default();
    assert_eq!(obj_type, ObjectType::Model);
}

#[test]
fn test_object_type_display() {
    assert_eq!(ObjectType::Model.to_string(), "model");
    assert_eq!(ObjectType::Support.to_string(), "support");
    assert_eq!(ObjectType::SolidSupport.to_string(), "solidsupport");
    assert_eq!(ObjectType::Surface.to_string(), "surface");
    assert_eq!(ObjectType::Other.to_string(), "other");
}

#[test]
fn test_requires_manifold() {
    assert!(ObjectType::Model.requires_manifold());
    assert!(ObjectType::SolidSupport.requires_manifold());
    assert!(!ObjectType::Support.requires_manifold());
    assert!(!ObjectType::Surface.requires_manifold());
    assert!(!ObjectType::Other.requires_manifold());
}

#[test]
fn test_can_be_in_build() {
    assert!(ObjectType::Model.can_be_in_build());
    assert!(ObjectType::Support.can_be_in_build());
    assert!(ObjectType::SolidSupport.can_be_in_build());
    assert!(ObjectType::Surface.can_be_in_build());
    assert!(!ObjectType::Other.can_be_in_build());
}

#[test]
fn test_parse_all_object_types() {
    // Create XML snippets for each type and verify parsing
    let test_cases = [
        ("model", ObjectType::Model),
        ("support", ObjectType::Support),
        ("solidsupport", ObjectType::SolidSupport),
        ("surface", ObjectType::Surface),
        ("other", ObjectType::Other),
    ];

    for (type_str, expected) in test_cases {
        let xml = format!(
            r#"<?xml version="1.0"?>
            <model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
                <resources>
                    <object id="1" type="{}">
                        <mesh>
                            <vertices>
                                <vertex x="0" y="0" z="0"/>
                                <vertex x="1" y="0" z="0"/>
                                <vertex x="0" y="1" z="0"/>
                                <vertex x="0" y="0" z="1"/>
                            </vertices>
                            <triangles>
                                <triangle v1="0" v2="1" v3="2"/>
                                <triangle v1="0" v2="2" v3="3"/>
                                <triangle v1="0" v2="3" v3="1"/>
                                <triangle v1="1" v2="3" v3="2"/>
                            </triangles>
                        </mesh>
                    </object>
                </resources>
                <build/>
            </model>"#,
            type_str
        );

        let model = parse_model(std::io::Cursor::new(xml.as_bytes()))
            .unwrap_or_else(|_| panic!("Failed to parse type: {}", type_str));

        let objects: Vec<_> = model.resources.iter_objects().collect();
        assert_eq!(objects.len(), 1, "Expected 1 object for type {}", type_str);
        assert_eq!(
            objects[0].object_type, expected,
            "Type mismatch for {}",
            type_str
        );
    }
}

#[test]
fn test_parse_missing_type_defaults_to_model() {
    let xml = r#"<?xml version="1.0"?>
        <model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
            <resources>
                <object id="1">
                    <mesh>
                        <vertices>
                            <vertex x="0" y="0" z="0"/>
                            <vertex x="1" y="0" z="0"/>
                            <vertex x="0" y="1" z="0"/>
                        </vertices>
                        <triangles>
                            <triangle v1="0" v2="1" v3="2"/>
                        </triangles>
                    </mesh>
                </object>
            </resources>
            <build/>
        </model>"#;

    let model = parse_model(std::io::Cursor::new(xml.as_bytes()))
        .expect("Failed to parse model without type attribute");

    let objects: Vec<_> = model.resources.iter_objects().collect();
    assert_eq!(objects[0].object_type, ObjectType::Model);
}

#[test]
fn test_roundtrip_preserves_object_type() {
    use lib3mf_core::model::Model;

    for test_type in [
        ObjectType::Model,
        ObjectType::Support,
        ObjectType::SolidSupport,
        ObjectType::Surface,
        ObjectType::Other,
    ] {
        let mut model = Model::default();
        let mut mesh = Mesh::new();
        mesh.add_vertex(0.0, 0.0, 0.0);
        mesh.add_vertex(1.0, 0.0, 0.0);
        mesh.add_vertex(0.0, 1.0, 0.0);
        mesh.add_triangle(0, 1, 2);

        let obj = Object {
            id: ResourceId(1),
            object_type: test_type,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(mesh),
        };
        model.resources.add_object(obj).unwrap();

        // Write to XML
        let mut xml_output = Vec::new();
        model
            .write_xml(&mut xml_output, None)
            .expect("Write failed");

        // Parse back
        let parsed = parse_model(std::io::Cursor::new(&xml_output)).expect("Parse failed");

        let objects: Vec<_> = parsed.resources.iter_objects().collect();
        assert_eq!(
            objects[0].object_type, test_type,
            "Round-trip failed for {:?}",
            test_type
        );
    }
}

#[test]
fn test_type_counts_in_stats() {
    // Create a model with multiple object types
    let xml = r#"<?xml version="1.0"?>
        <model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
            <resources>
                <object id="1" type="model">
                    <mesh>
                        <vertices>
                            <vertex x="0" y="0" z="0"/>
                            <vertex x="1" y="0" z="0"/>
                            <vertex x="0" y="1" z="0"/>
                        </vertices>
                        <triangles>
                            <triangle v1="0" v2="1" v3="2"/>
                        </triangles>
                    </mesh>
                </object>
                <object id="2" type="support">
                    <mesh>
                        <vertices>
                            <vertex x="0" y="0" z="0"/>
                            <vertex x="1" y="0" z="0"/>
                            <vertex x="0" y="1" z="0"/>
                        </vertices>
                        <triangles>
                            <triangle v1="0" v2="1" v3="2"/>
                        </triangles>
                    </mesh>
                </object>
                <object id="3" type="other">
                    <mesh>
                        <vertices>
                            <vertex x="0" y="0" z="0"/>
                            <vertex x="1" y="0" z="0"/>
                            <vertex x="0" y="1" z="0"/>
                        </vertices>
                        <triangles>
                            <triangle v1="0" v2="1" v3="2"/>
                        </triangles>
                    </mesh>
                </object>
            </resources>
            <build>
                <item objectid="1"/>
                <item objectid="2"/>
            </build>
        </model>"#;

    let model = parse_model(std::io::Cursor::new(xml.as_bytes())).expect("Failed to parse model");

    // Create a dummy archive reader for stats
    struct NoArchive;
    impl std::io::Read for NoArchive {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Ok(0)
        }
    }
    impl std::io::Seek for NoArchive {
        fn seek(&mut self, _: std::io::SeekFrom) -> std::io::Result<u64> {
            Ok(0)
        }
    }
    impl lib3mf_core::archive::ArchiveReader for NoArchive {
        fn read_entry(&mut self, _: &str) -> lib3mf_core::error::Result<Vec<u8>> {
            Err(lib3mf_core::error::Lib3mfError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No archive",
            )))
        }
        fn entry_exists(&mut self, _: &str) -> bool {
            false
        }
        fn list_entries(&mut self) -> lib3mf_core::error::Result<Vec<String>> {
            Ok(vec![])
        }
    }

    let stats = model.compute_stats(&mut NoArchive).expect("Stats failed");

    // Check type counts - only objects referenced in build are counted
    // Build items reference object 1 (model) and object 2 (support)
    // Object 3 (other) is not in build, so it won't be counted
    assert_eq!(stats.geometry.type_counts.get("model"), Some(&1));
    assert_eq!(stats.geometry.type_counts.get("support"), Some(&1));
    // Object 3 is not in build, so "other" should not appear in counts
    assert_eq!(stats.geometry.type_counts.get("other"), None);
}
