use lib3mf_core::model::{Mesh, MeshRepair, RepairOptions};

fn main() {
    println!("--- Geometry Repair Example ---");

    // 1. Create a broken mesh
    let mut mesh = Mesh::new();

    // Triangle 1
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_triangle(0, 1, 2);

    // Triangle 2 (Disconnected from Tri 1, vertices duplicated slightly offset)
    // Offset by 0.00001 (10 microns), default repair epsilon is 0.0001 (100 microns)
    mesh.add_vertex(1.00001, 0.0, 0.0); // Should be 1
    mesh.add_vertex(0.0, 1.00001, 0.0); // Should be 2
    mesh.add_vertex(1.0, 1.0, 0.0);
    mesh.add_triangle(3, 5, 4);

    // Degenerate Triangle (Zero Area)
    mesh.add_vertex(2.0, 0.0, 0.0);
    mesh.add_triangle(0, 1, 6); // (0,0,0)-(1,0,0)-(2,0,0) -> Line

    println!("Original Stats:");
    println!("  Vertices: {}", mesh.vertices.len());
    println!("  Triangles: {}", mesh.triangles.len());

    // 2. Repair
    println!("\nRepairing...");
    let options = RepairOptions {
        stitch_epsilon: 0.0001, // 0.1mm
        remove_degenerate: true,
        remove_duplicate_faces: true,
        harmonize_orientations: true,
        remove_islands: false,
        fill_holes: false,
    };

    let stats = mesh.repair(options);

    println!("Repair Report:");
    println!("  Vertices Removed:  {}", stats.vertices_removed);
    println!("  Triangles Removed: {}", stats.triangles_removed);
    println!("  Triangles Flipped: {}", stats.triangles_flipped);
    println!("  Triangles Added:   {}", stats.triangles_added);

    println!("\nFinal Stats:");
    println!("  Vertices: {}", mesh.vertices.len());
    println!("  Triangles: {}", mesh.triangles.len());

    // Check results
    if mesh.vertices.len() == 4 && mesh.triangles.len() == 2 {
        println!("\nSUCCESS: Mesh repaired correctly.");
    } else {
        println!("\nFAILURE: Unexpected repair results.");
    }
}
