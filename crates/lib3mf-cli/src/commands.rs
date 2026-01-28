use clap::ValueEnum;
use lib3mf_core::archive::{find_model_path, opc, ArchiveReader, ZipArchiver};
use lib3mf_core::parser::parse_model;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, ValueEnum, Debug, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Tree,
}

pub fn stats(path: PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let mut archiver = open_archive(&path)?;
    let model_path = find_model_path(&mut archiver).map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;
    let stats = model
        .compute_stats(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to compute stats: {}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        _ => {
            println!("Stats for {:?}", path);
            println!("Unit: {:?}", stats.unit);
            println!("Generator: {:?}", stats.generator.unwrap_or_default());
            println!("Geometry:");
            println!("  Objects: {}", stats.geometry.object_count);
            println!("  Instances: {}", stats.geometry.instance_count);
            println!("  Vertices: {}", stats.geometry.vertex_count);
            println!("  Triangles: {}", stats.geometry.triangle_count);
            
            println!("Materials:");
            println!("  Base Groups: {}", stats.materials.base_materials_count);
            println!("  Color Groups: {}", stats.materials.color_groups_count);
            
            if !stats.vendor.plates.is_empty() {
                println!("Vendor Data (Bambu):");
                println!("  Plates: {}", stats.vendor.plates.len());
                for plate in stats.vendor.plates {
                     println!("    - ID {}: {}", plate.id, plate.name.unwrap_or_default());
                }
            }
        }
    }
    Ok(())
}

pub fn list(path: PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let mut archiver = open_archive(&path)?;
    let entries = archiver.list_entries().map_err(|e| anyhow::anyhow!("Failed to list entries: {}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Tree => {
            print_tree(&entries);
        }
        OutputFormat::Text => {
            for entry in entries {
                println!("{}", entry);
            }
        }
    }
    Ok(())
}

pub fn rels(path: PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let mut archiver = open_archive(&path)?;
    
    // Read relationships
    let rels_data = archiver.read_entry("_rels/.rels").unwrap_or_default();
    let rels = if !rels_data.is_empty() {
        opc::parse_relationships(&rels_data).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // Read content types
    let types_data = archiver.read_entry("[Content_Types].xml").unwrap_or_default();
    let types = if !types_data.is_empty() {
        opc::parse_content_types(&types_data).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    match format {
        OutputFormat::Json => {
            #[derive(Serialize)]
            struct OpcData {
                relationships: Vec<lib3mf_core::archive::opc::Relationship>,
                content_types: Vec<lib3mf_core::archive::opc::ContentType>,
            }
            let data = OpcData { relationships: rels, content_types: types };
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        _ => {
            println!("Relationships:");
            for rel in rels {
                println!("  - ID: {}, Type: {}, Target: {}", rel.id, rel.rel_type, rel.target);
            }
            println!("\nContent Types:");
            for ct in types {
                 println!("  - {:?}", ct);
            }
        }
    }
    Ok(())
}

pub fn dump(path: PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let mut archiver = open_archive(&path)?;
    let model_path = find_model_path(&mut archiver).map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&model)?);
        }
        _ => {
            println!("{:#?}", model);
        }
    }
    Ok(())
}

pub fn extract(path: PathBuf, inner_path: String, output: Option<PathBuf>) -> anyhow::Result<()> {
    let mut archiver = open_archive(&path)?;
    let data = archiver.read_entry(&inner_path).map_err(|e| anyhow::anyhow!("Failed to read entry '{}': {}", inner_path, e))?;
    
    if let Some(out_path) = output {
        let mut f = File::create(&out_path).map_err(|e| anyhow::anyhow!("Failed to create output file {:?}: {}", out_path, e))?;
        f.write_all(&data)?;
        println!("Extracted '{}' to {:?}", inner_path, out_path);
    } else {
        std::io::stdout().write_all(&data)?;
    }
    Ok(())
}

pub fn copy(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    let mut archiver = open_archive(&input)?;
    let model_path = find_model_path(&mut archiver).map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    let file = File::create(&output).map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
    model.write(file).map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;
    
    println!("Copied {:?} to {:?}", input, output);
    Ok(())
}

fn open_archive(path: &PathBuf) -> anyhow::Result<ZipArchiver<File>> {
    let file = File::open(path).map_err(|e| anyhow::anyhow!("Failed to open file {:?}: {}", path, e))?;
    ZipArchiver::new(file).map_err(|e| anyhow::anyhow!("Failed to open zip archive: {}", e))
}

fn print_tree(paths: &[String]) {
    // Basic tree printer
    // Build a map of path components
    let mut tree: BTreeMap<String, node::Node> = BTreeMap::new();
    
    for path in paths {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current_level = &mut tree;
        
        for (i, part) in parts.iter().enumerate() {
            let is_file = i == parts.len() - 1;
            let node = current_level.entry(part.to_string()).or_insert_with(|| node::Node::new());
            current_level = &mut node.children;
        }
    }
    
    node::print_nodes(&tree, "");
}

mod node {
    use std::collections::BTreeMap;
    
    pub struct Node {
        pub children: BTreeMap<String, Node>,
    }
    
    impl Node {
        pub fn new() -> Self {
            Self { children: BTreeMap::new() }
        }
    }
    
    pub fn print_nodes(nodes: &BTreeMap<String, Node>, prefix: &str) {
        let count = nodes.len();
        for (i, (name, node)) in nodes.iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_last { "└── " } else { "├── " };
            println!("{}{}{}", prefix, connector, name);
             
            let child_prefix = if is_last { "    " } else { "│   " };
            let new_prefix = format!("{}{}", prefix, child_prefix);
            print_nodes(&node.children, &new_prefix);
        }
    }
}
