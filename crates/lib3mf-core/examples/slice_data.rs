use lib3mf_core::model::{
    BuildItem, Geometry, Model, Object, Polygon, ResourceId, Segment, Slice, SliceStack, Unit,
    Vertex2D,
};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    println!("Creating model with slice data...");

    let mut model = Model {
        unit: Unit::Millimeter,
        ..Default::default()
    };

    // 1. Create a Slice Stack
    // A slice stack defines geometry as a series of 2D layers.
    let mut slice_stack = SliceStack {
        id: ResourceId(5),
        z_bottom: 0.0,
        slices: Vec::new(),
        refs: Vec::new(),
    };

    // Layer 1: A square at Z=1.0
    let mut slice1 = Slice {
        z_top: 1.0,
        vertices: vec![
            Vertex2D { x: 0.0, y: 0.0 },
            Vertex2D { x: 10.0, y: 0.0 },
            Vertex2D { x: 10.0, y: 10.0 },
            Vertex2D { x: 0.0, y: 10.0 },
        ],
        polygons: Vec::new(),
    };

    // Define the square polygon
    let poly1 = Polygon {
        start_segment: 0,
        segments: vec![
            Segment {
                v2: 1,
                ..Default::default()
            },
            Segment {
                v2: 2,
                ..Default::default()
            },
            Segment {
                v2: 3,
                ..Default::default()
            },
            Segment {
                v2: 0,
                ..Default::default()
            },
        ],
    };
    slice1.polygons.push(poly1);
    slice_stack.slices.push(slice1);

    // Layer 2: A slightly smaller square at Z=2.0
    let mut slice2 = Slice {
        z_top: 2.0,
        vertices: vec![
            Vertex2D { x: 1.0, y: 1.0 },
            Vertex2D { x: 9.0, y: 1.0 },
            Vertex2D { x: 9.0, y: 9.0 },
            Vertex2D { x: 1.0, y: 9.0 },
        ],
        polygons: Vec::new(),
    };
    let poly2 = Polygon {
        start_segment: 0,
        segments: vec![
            Segment {
                v2: 1,
                ..Default::default()
            },
            Segment {
                v2: 2,
                ..Default::default()
            },
            Segment {
                v2: 3,
                ..Default::default()
            },
            Segment {
                v2: 0,
                ..Default::default()
            },
        ],
    };
    slice2.polygons.push(poly2);
    slice_stack.slices.push(slice2);

    // 2. Add Slice Stack to Resources
    // Note: ResourceCollection usually has a method like add_slice_stack.
    model.resources.add_slice_stack(slice_stack)?;

    // 3. Create Object referencing the Slice Stack
    let object_id = ResourceId(10);
    let object = Object {
        id: object_id,
        name: Some("Sliced Geometry".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
        geometry: Geometry::SliceStack(ResourceId(5)),
    };
    model.resources.add_object(object)?;

    // 4. Add to Build
    let item = BuildItem {
        object_id,
        transform: glam::Mat4::IDENTITY,
        part_number: None,
        uuid: None,
        path: None,
    };
    model.build.items.push(item);

    // 5. Write to file
    let file = File::create("slices.3mf")?;
    model.write(file)?;

    println!("Written to slices.3mf");

    Ok(())
}
