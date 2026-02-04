use lib3mf_core::model::{CapMode, ClippingMode, Geometry};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

/// Test basic beam lattice parsing with all three cap modes
#[test]
fn test_parse_beam_lattice_cap_modes() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="10" y="0" z="0" />
                    <vertex x="0" y="10" z="0" />
                    <vertex x="0" y="0" z="10" />
                </vertices>
                <beamlattice minlength="0.1" precision="0.01" clippingmode="inside">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" r2="1.0" cap="hemisphere" />
                        <beam v1="0" v2="2" r1="1.5" />
                        <beam v1="0" v2="3" r1="0.5" cap="butt" />
                    </beams>
                    <beamsets>
                        <beamset name="Set1" identifier="BS1">
                            <ref index="0" />
                            <ref index="1" />
                        </beamset>
                    </beamsets>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .expect("Object missing");

    if let Geometry::Mesh(mesh) = &obj.geometry {
        assert_eq!(mesh.vertices.len(), 4);

        let lattice = mesh.beam_lattice.as_ref().expect("Beam lattice missing");

        // Check lattice attributes
        assert_eq!(lattice.min_length, 0.1);
        assert_eq!(lattice.precision, 0.01);
        assert_eq!(lattice.clipping_mode, ClippingMode::Inside);

        // Check Beams
        assert_eq!(lattice.beams.len(), 3);

        // Beam 0: hemisphere cap
        let b0 = &lattice.beams[0];
        assert_eq!(b0.v1, 0);
        assert_eq!(b0.v2, 1);
        assert_eq!(b0.r1, 1.0);
        assert_eq!(b0.r2, 1.0);
        assert_eq!(b0.cap_mode, CapMode::Hemisphere);

        // Beam 1: default sphere cap (not specified)
        let b1 = &lattice.beams[1];
        assert_eq!(b1.v1, 0);
        assert_eq!(b1.v2, 2);
        assert_eq!(b1.r1, 1.5);
        assert_eq!(b1.r2, 1.5); // r2 defaults to r1
        assert_eq!(b1.cap_mode, CapMode::Sphere); // Default

        // Beam 2: butt cap
        let b2 = &lattice.beams[2];
        assert_eq!(b2.cap_mode, CapMode::Butt);

        // Check BeamSets
        assert_eq!(lattice.beam_sets.len(), 1);
        let bs = &lattice.beam_sets[0];
        assert_eq!(bs.name, Some("Set1".to_string()));
        assert_eq!(bs.identifier, Some("BS1".to_string()));
        assert_eq!(bs.refs.len(), 2);
        assert_eq!(bs.refs[0], 0);
        assert_eq!(bs.refs[1], 1);
    } else {
        panic!("Geometry is not a mesh");
    }

    Ok(())
}

/// Test all three clipping modes
#[test]
fn test_beam_lattice_clipping_modes() -> anyhow::Result<()> {
    // Test clipping mode: none (default)
    let xml_none = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml_none))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();
    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lattice.clipping_mode, ClippingMode::None);
    }

    // Test clipping mode: inside
    let xml_inside = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1" clippingmode="inside">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml_inside))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();
    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lattice.clipping_mode, ClippingMode::Inside);
    }

    // Test clipping mode: outside
    let xml_outside = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1" clippingmode="outside">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml_outside))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();
    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lattice.clipping_mode, ClippingMode::Outside);
    }

    Ok(())
}

/// Test beam property indices (pid, p1, p2)
#[test]
fn test_beam_lattice_property_indices() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="10" y="0" z="0" />
                    <vertex x="0" y="10" z="0" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" p1="5" p2="10" />
                        <beam v1="1" v2="2" r1="1.5" p1="7" />
                        <beam v1="0" v2="2" r1="0.8" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lattice.beams.len(), 3);

        // Beam 0: both p1 and p2 specified
        assert_eq!(lattice.beams[0].p1, Some(5));
        assert_eq!(lattice.beams[0].p2, Some(10));

        // Beam 1: only p1 specified
        assert_eq!(lattice.beams[1].p1, Some(7));
        assert_eq!(lattice.beams[1].p2, None);

        // Beam 2: no property indices
        assert_eq!(lattice.beams[2].p1, None);
        assert_eq!(lattice.beams[2].p2, None);
    } else {
        panic!("Expected mesh geometry");
    }

    Ok(())
}

/// Test multiple beam sets with various configurations
#[test]
fn test_beam_lattice_multiple_beam_sets() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                    <vertex x="0" y="0" z="1" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" />
                        <beam v1="0" v2="2" r1="1.0" />
                        <beam v1="0" v2="3" r1="1.0" />
                        <beam v1="1" v2="2" r1="1.0" />
                    </beams>
                    <beamsets>
                        <beamset name="Horizontal" identifier="H1">
                            <ref index="0" />
                        </beamset>
                        <beamset name="Vertical">
                            <ref index="1" />
                            <ref index="2" />
                        </beamset>
                        <beamset identifier="DIAG">
                            <ref index="3" />
                        </beamset>
                        <beamset />
                    </beamsets>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lattice.beam_sets.len(), 4);

        // BeamSet 0: has both name and identifier
        assert_eq!(lattice.beam_sets[0].name, Some("Horizontal".to_string()));
        assert_eq!(lattice.beam_sets[0].identifier, Some("H1".to_string()));
        assert_eq!(lattice.beam_sets[0].refs, vec![0]);

        // BeamSet 1: has name only
        assert_eq!(lattice.beam_sets[1].name, Some("Vertical".to_string()));
        assert_eq!(lattice.beam_sets[1].identifier, None);
        assert_eq!(lattice.beam_sets[1].refs, vec![1, 2]);

        // BeamSet 2: has identifier only
        assert_eq!(lattice.beam_sets[2].name, None);
        assert_eq!(lattice.beam_sets[2].identifier, Some("DIAG".to_string()));
        assert_eq!(lattice.beam_sets[2].refs, vec![3]);

        // BeamSet 3: empty set (no name, identifier, or refs)
        assert_eq!(lattice.beam_sets[3].name, None);
        assert_eq!(lattice.beam_sets[3].identifier, None);
        assert_eq!(lattice.beam_sets[3].refs.len(), 0);
    } else {
        panic!("Expected mesh geometry");
    }

    Ok(())
}

/// Test edge case: beam lattice with no beams or beam sets
#[test]
fn test_beam_lattice_empty() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.5" precision="0.001">
                    <beams />
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lattice.min_length, 0.5);
        assert_eq!(lattice.precision, 0.001);
        assert_eq!(lattice.beams.len(), 0);
        assert_eq!(lattice.beam_sets.len(), 0);
    } else {
        panic!("Expected mesh geometry");
    }

    Ok(())
}

/// Test r2 defaults to r1 when not specified
#[test]
fn test_beam_r2_defaults_to_r1() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="2" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>
                        <beam v1="0" v2="1" r1="2.5" />
                        <beam v1="1" v2="2" r1="1.0" r2="3.0" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .unwrap();

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lattice = mesh.beam_lattice.as_ref().unwrap();

        // Beam 0: r2 not specified, should default to r1
        assert_eq!(lattice.beams[0].r1, 2.5);
        assert_eq!(lattice.beams[0].r2, 2.5);

        // Beam 1: r2 explicitly specified
        assert_eq!(lattice.beams[1].r1, 1.0);
        assert_eq!(lattice.beams[1].r2, 3.0);
    } else {
        panic!("Expected mesh geometry");
    }

    Ok(())
}

// ============================================================================
// ERROR PATH TESTS
// ============================================================================

/// Test EOF during beamlattice element parsing
#[test]
fn test_beam_lattice_truncated_eof_in_beamlattice() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>"##; // Truncated here - EOF in beamlattice

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in beamlattice element"
    );
}

/// Test EOF during beams element parsing
#[test]
fn test_beam_lattice_truncated_eof_in_beams() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" />"##; // Truncated here - EOF in beams

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in beams element"
    );
}

/// Test EOF during beamsets element parsing
#[test]
fn test_beam_lattice_truncated_eof_in_beamsets() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                </vertices>
                <beamlattice minlength="0.1">
                    <beams>
                        <beam v1="0" v2="1" r1="1.0" />
                    </beams>
                    <beamsets>
                        <beamset name="Set1">
                            <ref index="0" />"##; // Truncated here - EOF in beamsets

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in beamsets element"
    );
}
