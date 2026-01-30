use lib3mf_core::error::Result;
use lib3mf_core::model::{
    BaseMaterial, BaseMaterialsGroup, BlendMethod, Color, Composite, CompositeMaterials, Model,
    Multi, MultiProperties, ResourceId, Texture2DCoord, Texture2DGroup, Unit,
};

fn main() -> Result<()> {
    // This example demonstrates how to create and access advanced material properties
    // like Texture2DGroup, CompositeMaterials, and MultiProperties.

    println!("Advanced Materials Example");
    println!("==========================");

    let mut model = Model {
        unit: Unit::Millimeter,
        ..Default::default()
    };

    // 1. Setup Base Materials
    // Advanced materials often reference base materials.
    let base_group = BaseMaterialsGroup {
        id: ResourceId(1),
        materials: vec![
            BaseMaterial {
                name: "Red Plastic".to_string(),
                display_color: Color::new(255, 0, 0, 255),
            },
            BaseMaterial {
                name: "Blue Plastic".to_string(),
                display_color: Color::new(0, 0, 255, 255),
            },
        ],
    };
    model.resources.add_base_materials(base_group)?;

    // 2. Texture 2D Groups
    // Defines a set of UV coordinates for a texture.
    let tex_group = Texture2DGroup {
        id: ResourceId(2),
        texture_id: ResourceId(100), // Referenced texture resource ID
        coords: vec![
            Texture2DCoord { u: 0.0, v: 0.0 },
            Texture2DCoord { u: 1.0, v: 0.0 },
            Texture2DCoord { u: 1.0, v: 1.0 },
            Texture2DCoord { u: 0.0, v: 1.0 },
        ],
    };
    model.resources.add_texture_2d_group(tex_group)?;
    println!("Added Texture2DGroup (ID: 2) with 4 coordinates");

    // 3. Composite Materials
    // Mix multiple base materials in specific ratios.
    let comp_group = CompositeMaterials {
        id: ResourceId(3),
        base_material_id: ResourceId(1), // Base group to mix from
        indices: vec![0, 1],             // Red and Blue indices
        composites: vec![
            Composite {
                values: vec![0.5, 0.5],
            }, // 50/50 mix
            Composite {
                values: vec![0.2, 0.8],
            }, // 20/80 mix
        ],
    };
    model.resources.add_composite_materials(comp_group)?;
    println!("Added CompositeMaterials (ID: 3) with 2 mixing ratios");

    // 4. Multi Properties
    // Combine properties from different groups (e.g., Mix Color + Texture).
    let multi_prop = MultiProperties {
        id: ResourceId(4),
        pids: vec![ResourceId(1), ResourceId(2)], // Reference Base Materials and Texture Group
        blend_methods: vec![BlendMethod::Mix, BlendMethod::Multiply],
        multis: vec![
            Multi {
                pindices: vec![0, 0],
            }, // Red + first UV
            Multi {
                pindices: vec![1, 2],
            }, // Blue + third UV
        ],
    };
    model.resources.add_multi_properties(multi_prop)?;
    println!("Added MultiProperties (ID: 4) combining Base Materials and Textures");

    // Accessing resources back
    let resources = &model.resources;
    println!("\nSummary of model resources:");
    println!(
        "  Base Material Groups: {}",
        resources.base_material_groups_count()
    );
    println!(
        "  Texture 2D Groups:    {}",
        resources.texture_2d_groups_count()
    );
    println!(
        "  Composite Materials:  {}",
        resources.composite_materials_count()
    );
    println!(
        "  Multi Properties:     {}",
        resources.multi_properties_count()
    );

    if let Some(mg) = resources.get_multi_properties(ResourceId(4)) {
        println!("\nInspecting MultiProperties ID 4:");
        for (i, m) in mg.multis.iter().enumerate() {
            println!(
                "  Property {}: references pids {:?} with indices {:?}",
                i, mg.pids, m.pindices
            );
        }
    }

    Ok(())
}
