use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::model::{Beam, DisplacementTriangle, ResourceId};
use lib3mf_core::parser::streaming::parse_model_streaming;
use lib3mf_core::parser::visitor::ModelVisitor;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

/// A simple visitor that counts vertices, triangles, beams, and displacement mesh elements
/// across all objects in a 3MF file.
struct StatsVisitor {
    vertex_count: usize,
    triangle_count: usize,
    beam_count: usize,
    disp_vertex_count: usize,
    disp_triangle_count: usize,
    disp_normal_count: usize,
}

impl ModelVisitor for StatsVisitor {
    fn on_start_mesh(&mut self, id: ResourceId) -> lib3mf_core::error::Result<()> {
        println!("Processing mesh ID: {}", id.0);
        Ok(())
    }

    fn on_vertex(&mut self, _x: f32, _y: f32, _z: f32) -> lib3mf_core::error::Result<()> {
        self.vertex_count += 1;
        Ok(())
    }

    fn on_triangle(&mut self, _v1: u32, _v2: u32, _v3: u32) -> lib3mf_core::error::Result<()> {
        self.triangle_count += 1;
        Ok(())
    }

    fn on_metadata(&mut self, name: &str, value: &str) -> lib3mf_core::error::Result<()> {
        println!("Metadata: {} = {}", name, value);
        Ok(())
    }

    fn on_start_beam_lattice(&mut self, id: ResourceId) -> lib3mf_core::error::Result<()> {
        println!("Processing beam lattice for object ID: {}", id.0);
        Ok(())
    }

    fn on_beam(&mut self, _beam: &Beam) -> lib3mf_core::error::Result<()> {
        self.beam_count += 1;
        Ok(())
    }

    fn on_end_beam_lattice(&mut self) -> lib3mf_core::error::Result<()> {
        println!("  Beams: {}", self.beam_count);
        Ok(())
    }

    fn on_start_displacement_mesh(&mut self, id: ResourceId) -> lib3mf_core::error::Result<()> {
        println!("Processing displacement mesh ID: {}", id.0);
        Ok(())
    }

    fn on_displacement_vertex(
        &mut self,
        _x: f32,
        _y: f32,
        _z: f32,
    ) -> lib3mf_core::error::Result<()> {
        self.disp_vertex_count += 1;
        Ok(())
    }

    fn on_displacement_triangle(
        &mut self,
        _triangle: &DisplacementTriangle,
    ) -> lib3mf_core::error::Result<()> {
        self.disp_triangle_count += 1;
        Ok(())
    }

    fn on_displacement_normal(
        &mut self,
        _nx: f32,
        _ny: f32,
        _nz: f32,
    ) -> lib3mf_core::error::Result<()> {
        self.disp_normal_count += 1;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: streaming_stats <file.3mf>");
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);
    let file = File::open(path)?;
    let mut archiver = ZipArchiver::new(file)?;

    // 1. Locate the 3D model part
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;

    // 2. Initialize the visitor
    let mut visitor = StatsVisitor {
        vertex_count: 0,
        triangle_count: 0,
        beam_count: 0,
        disp_vertex_count: 0,
        disp_triangle_count: 0,
        disp_normal_count: 0,
    };

    // 3. Parse in a streaming fashion
    // This doesn't build the full Model DOM in memory.
    println!("Parsing streaming model data...");
    parse_model_streaming(Cursor::new(model_data), &mut visitor)?;

    println!("--- Streaming Stats ---");
    println!("Total Vertices: {}", visitor.vertex_count);
    println!("Total Triangles: {}", visitor.triangle_count);
    println!("Total Beams: {}", visitor.beam_count);
    println!("Total Displacement Vertices: {}", visitor.disp_vertex_count);
    println!(
        "Total Displacement Triangles: {}",
        visitor.disp_triangle_count
    );
    println!("Total Displacement Normals: {}", visitor.disp_normal_count);

    Ok(())
}
