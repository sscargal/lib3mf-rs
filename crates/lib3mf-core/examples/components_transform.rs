use glam::{Mat4, Vec3};
use lib3mf_core::model::{
    BuildItem, Component, Components, Geometry, Mesh, Model, Object, ResourceId, Unit,
};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    println!("Creating model with components and transforms...");

    let mut model = Model {
        unit: Unit::Millimeter,
        ..Default::default()
    };

    // 1. Create a Pyramid Mesh
    let mut mesh = Mesh::new();
    let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v1 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(10.0, 10.0, 0.0);
    let v3 = mesh.add_vertex(0.0, 10.0, 0.0);
    let v_top = mesh.add_vertex(5.0, 5.0, 15.0);

    // Base
    mesh.add_triangle(v0, v2, v1);
    mesh.add_triangle(v0, v3, v2);
    // Sides
    mesh.add_triangle(v0, v1, v_top);
    mesh.add_triangle(v1, v2, v_top);
    mesh.add_triangle(v2, v3, v_top);
    mesh.add_triangle(v3, v0, v_top);

    let mesh_id = ResourceId(1);
    let mesh_obj = Object {
        id: mesh_id,
        name: Some("Pyramid Mesh".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
        geometry: Geometry::Mesh(mesh),
    };
    model.resources.add_object(mesh_obj)?;

    // 2. Create Components Object (Assembly)
    // We will place two pyramids.
    let comp1 = Component {
        object_id: mesh_id,
        path: None,
        uuid: None,
        transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    };

    let comp2 = Component {
        object_id: mesh_id,
        path: None,
        uuid: None,
        transform: Mat4::from_translation(Vec3::new(20.0, 0.0, 0.0))
            * Mat4::from_rotation_z(45.0_f32.to_radians()),
    };

    let assembly_id = ResourceId(2);
    let components = Components {
        components: vec![comp1, comp2],
    };

    let assembly_obj = Object {
        id: assembly_id,
        name: Some("Pyramid Assembly".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
        geometry: Geometry::Components(components),
    };
    model.resources.add_object(assembly_obj)?;

    // 3. Add Assembly to Build
    let item = BuildItem {
        object_id: assembly_id,
        transform: Mat4::IDENTITY,
        part_number: None,
        uuid: None,
        path: None,
    };
    model.build.items.push(item);

    // 4. Write
    let file = File::create("components.3mf")?;
    model.write(file)?;

    println!("Written to components.3mf");
    Ok(())
}
