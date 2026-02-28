use lib3mf_core::error::Result;
use lib3mf_core::model::{Beam, DisplacementTriangle, ResourceId};
use lib3mf_core::parser::streaming::parse_model_streaming;
use lib3mf_core::parser::visitor::ModelVisitor;
use std::io::Cursor;

struct TestVisitor {
    events: Vec<String>,
    beam_count: usize,
    disp_vertex_count: usize,
    disp_triangle_count: usize,
    disp_normal_count: usize,
    vertex_count: usize,
    triangle_count: usize,
}

impl TestVisitor {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            beam_count: 0,
            disp_vertex_count: 0,
            disp_triangle_count: 0,
            disp_normal_count: 0,
            vertex_count: 0,
            triangle_count: 0,
        }
    }
}

impl ModelVisitor for TestVisitor {
    fn on_vertex(&mut self, _x: f32, _y: f32, _z: f32) -> Result<()> {
        self.vertex_count += 1;
        Ok(())
    }

    fn on_triangle(&mut self, _v1: u32, _v2: u32, _v3: u32) -> Result<()> {
        self.triangle_count += 1;
        Ok(())
    }

    fn on_start_beam_lattice(&mut self, id: ResourceId) -> Result<()> {
        self.events.push(format!("start_beam_lattice:{}", id.0));
        Ok(())
    }

    fn on_beam(&mut self, _beam: &Beam) -> Result<()> {
        self.beam_count += 1;
        Ok(())
    }

    fn on_end_beam_lattice(&mut self) -> Result<()> {
        self.events.push("end_beam_lattice".to_string());
        Ok(())
    }

    fn on_start_displacement_mesh(&mut self, id: ResourceId) -> Result<()> {
        self.events
            .push(format!("start_displacement_mesh:{}", id.0));
        Ok(())
    }

    fn on_displacement_vertex(&mut self, _x: f32, _y: f32, _z: f32) -> Result<()> {
        self.disp_vertex_count += 1;
        Ok(())
    }

    fn on_displacement_triangle(&mut self, _triangle: &DisplacementTriangle) -> Result<()> {
        self.disp_triangle_count += 1;
        Ok(())
    }

    fn on_displacement_normal(&mut self, _nx: f32, _ny: f32, _nz: f32) -> Result<()> {
        self.disp_normal_count += 1;
        Ok(())
    }

    fn on_end_displacement_mesh(&mut self) -> Result<()> {
        self.events.push("end_displacement_mesh".to_string());
        Ok(())
    }
}

/// Helper to build a complete 3MF model XML string with the given resources content
fn make_model_xml(resources: &str, build_items: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        {}
    </resources>
    <build>
        {}
    </build>
</model>"#,
        resources, build_items
    )
}

// --- Tests ---

/// Beam lattice visitor callbacks fire for each beam and lifecycle methods fire in order.
#[test]
fn test_beam_lattice_streaming() {
    let resources = r#"
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
                <beamlattice radius="0.5" minlength="0.001">
                    <beams>
                        <beam v1="0" v2="1" r1="0.3" r2="0.4" />
                        <beam v1="1" v2="2" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    "#;
    let build = r#"<item objectid="1" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // on_start_beam_lattice fired with ResourceId(1)
    assert!(
        visitor.events.contains(&"start_beam_lattice:1".to_string()),
        "Expected start_beam_lattice:1 in events: {:?}",
        visitor.events
    );
    // Two beams processed
    assert_eq!(visitor.beam_count, 2);
    // on_end_beam_lattice fired
    assert!(
        visitor.events.contains(&"end_beam_lattice".to_string()),
        "Expected end_beam_lattice in events: {:?}",
        visitor.events
    );
    // Order: start_beam_lattice comes before end_beam_lattice
    let start_pos = visitor
        .events
        .iter()
        .position(|e| e == "start_beam_lattice:1")
        .unwrap();
    let end_pos = visitor
        .events
        .iter()
        .position(|e| e == "end_beam_lattice")
        .unwrap();
    assert!(
        start_pos < end_pos,
        "start_beam_lattice must come before end_beam_lattice"
    );
}

/// BeamSets are silently skipped — no crash, no beamset-related events.
#[test]
fn test_beam_lattice_with_beamsets_skipped() {
    let resources = r#"
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
                <beamlattice radius="0.5" minlength="0.001">
                    <beams>
                        <beam v1="0" v2="1" />
                        <beam v1="1" v2="2" />
                    </beams>
                    <beamsets>
                        <beamset name="set1">
                            <ref index="0"/>
                            <ref index="1"/>
                        </beamset>
                    </beamsets>
                </beamlattice>
            </mesh>
        </object>
    "#;
    let build = r#"<item objectid="1" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    // Must not crash
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // Beam callbacks still fire
    assert_eq!(visitor.beam_count, 2);
    // No beamset-related events in visitor
    let beamset_events: Vec<&String> = visitor
        .events
        .iter()
        .filter(|e| e.contains("beamset"))
        .collect();
    assert!(
        beamset_events.is_empty(),
        "Unexpected beamset events: {:?}",
        beamset_events
    );
}

/// Displacement mesh visitor callbacks fire for each vertex, triangle, and normal.
#[test]
fn test_displacement_mesh_streaming() {
    let resources = r#"
        <object id="2" type="model">
            <displacementmesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
                <normvectors>
                    <normvector nx="0" ny="0" nz="1" />
                </normvectors>
            </displacementmesh>
        </object>
    "#;
    let build = r#"<item objectid="2" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // on_start_displacement_mesh fired with ResourceId(2)
    assert!(
        visitor
            .events
            .contains(&"start_displacement_mesh:2".to_string()),
        "Expected start_displacement_mesh:2 in events: {:?}",
        visitor.events
    );
    assert_eq!(visitor.disp_vertex_count, 3);
    assert_eq!(visitor.disp_triangle_count, 1);
    assert_eq!(visitor.disp_normal_count, 1);
    // on_end_displacement_mesh fired
    assert!(
        visitor
            .events
            .contains(&"end_displacement_mesh".to_string()),
        "Expected end_displacement_mesh in events: {:?}",
        visitor.events
    );
}

/// disp2dgroups are silently skipped — no crash, vertices/triangles/normals still parse correctly.
#[test]
fn test_displacement_mesh_disp2dgroups_skipped() {
    let resources = r#"
        <object id="3" type="model">
            <displacementmesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
                <normvectors>
                    <normvector nx="0" ny="0" nz="1" />
                </normvectors>
                <disp2dgroups>
                    <disp2dgroup id="1">
                        <gradient gu="0.1" gv="0.2" />
                    </disp2dgroup>
                </disp2dgroups>
            </displacementmesh>
        </object>
    "#;
    let build = r#"<item objectid="3" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    // Must not crash from disp2dgroups
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // Vertices, triangles, normals still parsed correctly
    assert_eq!(visitor.disp_vertex_count, 3);
    assert_eq!(visitor.disp_triangle_count, 1);
    assert_eq!(visitor.disp_normal_count, 1);
}

/// Displacement mesh with no normvectors section is handled correctly.
#[test]
fn test_displacement_mesh_no_normals() {
    let resources = r#"
        <object id="4" type="model">
            <displacementmesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </displacementmesh>
        </object>
    "#;
    let build = r#"<item objectid="4" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    assert_eq!(visitor.disp_vertex_count, 3);
    assert_eq!(visitor.disp_triangle_count, 1);
    assert_eq!(
        visitor.disp_normal_count, 0,
        "Expected 0 normals when normvectors absent"
    );
}

/// Mesh with vertices/triangles AND beamlattice — both callbacks fire correctly.
#[test]
fn test_mesh_and_beam_lattice_same_object() {
    let resources = r#"
        <object id="5" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                    <vertex x="0" y="0" z="1" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                    <triangle v1="0" v2="1" v3="3" />
                    <triangle v1="0" v2="2" v3="3" />
                    <triangle v1="1" v2="2" v3="3" />
                </triangles>
                <beamlattice radius="0.2" minlength="0.01">
                    <beams>
                        <beam v1="0" v2="1" />
                        <beam v1="1" v2="2" />
                        <beam v1="2" v2="3" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    "#;
    let build = r#"<item objectid="5" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // Both mesh AND beam lattice callbacks fire
    assert_eq!(visitor.vertex_count, 4, "Mesh vertices counted");
    assert_eq!(visitor.triangle_count, 4, "Mesh triangles counted");
    assert_eq!(visitor.beam_count, 3, "Beam lattice beams counted");
}

/// Standard mesh with no extensions produces correct counts — backward compatibility.
#[test]
fn test_backward_compatibility() {
    let resources = r#"
        <object id="6" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </mesh>
        </object>
    "#;
    let build = r#"<item objectid="6" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // Existing behavior unchanged
    assert_eq!(visitor.vertex_count, 3);
    assert_eq!(visitor.triangle_count, 1);
    // No extension callbacks fired
    assert_eq!(visitor.beam_count, 0);
    assert_eq!(visitor.disp_vertex_count, 0);
    assert_eq!(visitor.disp_triangle_count, 0);
    assert_eq!(visitor.disp_normal_count, 0);
    let ext_events: Vec<&String> = visitor
        .events
        .iter()
        .filter(|e| e.contains("beam_lattice") || e.contains("displacement"))
        .collect();
    assert!(
        ext_events.is_empty(),
        "No extension events expected: {:?}",
        ext_events
    );
}

/// Two objects — one regular mesh, one displacement mesh — both processed correctly.
#[test]
fn test_multiple_objects_mixed() {
    let resources = r#"
        <object id="7" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </mesh>
        </object>
        <object id="8" type="model">
            <displacementmesh>
                <vertices>
                    <vertex x="0" y="0" z="5" />
                    <vertex x="1" y="0" z="5" />
                    <vertex x="0" y="1" z="5" />
                    <vertex x="1" y="1" z="5" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                    <triangle v1="1" v2="3" v3="2" />
                </triangles>
            </displacementmesh>
        </object>
    "#;
    let build = r#"<item objectid="7" /><item objectid="8" />"#;
    let xml = make_model_xml(resources, build);

    let mut visitor = TestVisitor::new();
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    // Object 7 — regular mesh
    assert_eq!(visitor.vertex_count, 3, "Object 7 mesh vertices");
    assert_eq!(visitor.triangle_count, 1, "Object 7 mesh triangles");

    // Object 8 — displacement mesh
    assert_eq!(
        visitor.disp_vertex_count, 4,
        "Object 8 displacement vertices"
    );
    assert_eq!(
        visitor.disp_triangle_count, 2,
        "Object 8 displacement triangles"
    );

    // start_displacement_mesh fired with id 8
    assert!(
        visitor
            .events
            .contains(&"start_displacement_mesh:8".to_string()),
        "Expected start_displacement_mesh:8 in events: {:?}",
        visitor.events
    );
}

/// Beam attributes are parsed correctly (r1, r2, default radius, cap mode).
#[test]
fn test_beam_attribute_parsing() {
    let resources = r#"
        <object id="9" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
                <beamlattice radius="0.5" minlength="0.001">
                    <beams>
                        <beam v1="0" v2="1" r1="0.3" r2="0.4" cap="butt" />
                        <beam v1="1" v2="2" r1="0.6" />
                        <beam v1="0" v2="2" />
                    </beams>
                </beamlattice>
            </mesh>
        </object>
    "#;
    let build = r#"<item objectid="9" />"#;
    let xml = make_model_xml(resources, build);

    // Use a visitor that captures beam data
    struct BeamCapture {
        beams: Vec<Beam>,
    }
    impl ModelVisitor for BeamCapture {
        fn on_beam(&mut self, beam: &Beam) -> lib3mf_core::error::Result<()> {
            self.beams.push(beam.clone());
            Ok(())
        }
    }

    let mut visitor = BeamCapture { beams: Vec::new() };
    parse_model_streaming(Cursor::new(xml), &mut visitor).expect("parse failed");

    assert_eq!(visitor.beams.len(), 3);

    // First beam: explicit r1, r2, cap=butt
    assert!((visitor.beams[0].r1 - 0.3).abs() < 1e-5, "beam[0].r1");
    assert!((visitor.beams[0].r2 - 0.4).abs() < 1e-5, "beam[0].r2");
    assert_eq!(visitor.beams[0].cap_mode, lib3mf_core::model::CapMode::Butt);

    // Second beam: explicit r1, r2 defaults to r1
    assert!((visitor.beams[1].r1 - 0.6).abs() < 1e-5, "beam[1].r1");
    assert!(
        (visitor.beams[1].r2 - 0.6).abs() < 1e-5,
        "beam[1].r2 should default to r1"
    );
    assert_eq!(
        visitor.beams[1].cap_mode,
        lib3mf_core::model::CapMode::Sphere
    );

    // Third beam: both r1 and r2 default to lattice radius (0.5)
    assert!(
        (visitor.beams[2].r1 - 0.5).abs() < 1e-5,
        "beam[2].r1 should default to lattice radius"
    );
    assert!(
        (visitor.beams[2].r2 - 0.5).abs() < 1e-5,
        "beam[2].r2 should default to r1"
    );
}
