use lib3mf_core::model::{
    Beam, BeamLattice, BuildItem, CapMode, ClippingMode, Geometry, Mesh, Model, Object, ResourceId,
    Unit,
};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    println!("Creating Beam Lattice model...");

    let mut model = Model {
        unit: Unit::Millimeter,
        ..Default::default()
    };

    // 1. Create Mesh with Vertices but NO Triangles
    let mut mesh = Mesh::new();

    // Define nodes of the lattice
    let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v1 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(0.0, 10.0, 0.0);
    let v3 = mesh.add_vertex(0.0, 0.0, 10.0);
    let v_center = mesh.add_vertex(3.3, 3.3, 3.3);

    // 2. Define Beams
    // Connect center to all corners
    let beams = vec![
        Beam {
            v1: v0,
            v2: v_center,
            r1: 1.0,
            r2: 0.5,
            cap_mode: CapMode::Sphere,
            ..Default::default()
        },
        Beam {
            v1,
            v2: v_center,
            r1: 1.0,
            r2: 0.5,
            cap_mode: CapMode::Sphere,
            ..Default::default()
        },
        Beam {
            v1: v2,
            v2: v_center,
            r1: 1.0,
            r2: 0.5,
            cap_mode: CapMode::Sphere,
            ..Default::default()
        },
        Beam {
            v1: v3,
            v2: v_center,
            r1: 1.0,
            r2: 0.5,
            cap_mode: CapMode::Sphere,
            ..Default::default()
        },
    ];

    let beam_lattice = BeamLattice {
        min_length: 0.1,
        precision: 0.0,
        clipping_mode: ClippingMode::None,
        beams,
        beam_sets: Vec::new(),
    };

    mesh.beam_lattice = Some(beam_lattice);

    // 3. Object
    let object_id = ResourceId(10);
    let object = Object {
        id: object_id,
        name: Some("Lattice Structure".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        thumbnail: None,
        pindex: None,
        geometry: Geometry::Mesh(mesh),
    };
    model.resources.add_object(object)?;

    // 4. Build
    let item = BuildItem {
        object_id,
        transform: glam::Mat4::IDENTITY,
        part_number: None,
        uuid: None,
        path: None,
    };
    model.build.items.push(item);

    // 5. Write
    let file = File::create("lattice.3mf")?;
    model.write(file)?;

    println!("Written to lattice.3mf");

    Ok(())
}
