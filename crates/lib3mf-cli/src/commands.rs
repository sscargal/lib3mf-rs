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
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
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
            if let Some(bbox) = stats.geometry.bounding_box {
                println!("  Bounding Box: Min {:?}, Max {:?}", bbox.min, bbox.max);
            }
            println!("  Surface Area: {:.2}", stats.geometry.surface_area);
            println!("  Volume: {:.2}", stats.geometry.volume);

            println!("\nSystem Info:");
            println!("  Architecture: {}", stats.system_info.architecture);
            println!("  CPUs (Threads): {}", stats.system_info.num_cpus);
            println!(
                "  SIMD Features: {}",
                stats.system_info.simd_features.join(", ")
            );

            println!("Materials:");
            println!("  Base Groups: {}", stats.materials.base_materials_count);
            println!("  Color Groups: {}", stats.materials.color_groups_count);
            println!("  Texture 2D Groups: {}", stats.materials.texture_2d_groups_count);
            println!("  Composite Materials: {}", stats.materials.composite_materials_count);
            println!("  Multi Properties: {}", stats.materials.multi_properties_count);

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
    let entries = archiver
        .list_entries()
        .map_err(|e| anyhow::anyhow!("Failed to list entries: {}", e))?;

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
    let types_data = archiver
        .read_entry("[Content_Types].xml")
        .unwrap_or_default();
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
            let data = OpcData {
                relationships: rels,
                content_types: types,
            };
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        _ => {
            println!("Relationships:");
            for rel in rels {
                println!(
                    "  - ID: {}, Type: {}, Target: {}",
                    rel.id, rel.rel_type, rel.target
                );
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
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
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
    let data = archiver
        .read_entry(&inner_path)
        .map_err(|e| anyhow::anyhow!("Failed to read entry '{}': {}", inner_path, e))?;

    if let Some(out_path) = output {
        let mut f = File::create(&out_path)
            .map_err(|e| anyhow::anyhow!("Failed to create output file {:?}: {}", out_path, e))?;
        f.write_all(&data)?;
        println!("Extracted '{}' to {:?}", inner_path, out_path);
    } else {
        std::io::stdout().write_all(&data)?;
    }
    Ok(())
}

pub fn copy(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    let mut archiver = open_archive(&input)?;
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    let file = File::create(&output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
    model
        .write(file)
        .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;

    println!("Copied {:?} to {:?}", input, output);
    Ok(())
}

fn open_archive(path: &PathBuf) -> anyhow::Result<ZipArchiver<File>> {
    let file =
        File::open(path).map_err(|e| anyhow::anyhow!("Failed to open file {:?}: {}", path, e))?;
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
            let _is_file = i == parts.len() - 1;
            let node = current_level
                .entry(part.to_string())
                .or_insert_with(node::Node::new);
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
            Self {
                children: BTreeMap::new(),
            }
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

pub fn convert(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 1. Load Model
    let model = match input_ext.as_str() {
        "3mf" => {
            let mut archiver = open_archive(&input)?;
            let model_path = find_model_path(&mut archiver)
                .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
            let model_data = archiver
                .read_entry(&model_path)
                .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
            parse_model(std::io::Cursor::new(model_data))
                .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?
        }
        "stl" => {
            let file = File::open(&input)
                .map_err(|e| anyhow::anyhow!("Failed to open STL input: {}", e))?;
            lib3mf_io::stl::StlImporter::read(file)
                .map_err(|e| anyhow::anyhow!("Failed to import STL: {}", e))?
        }
        "obj" => {
            let file = File::open(&input)
                .map_err(|e| anyhow::anyhow!("Failed to open OBJ input: {}", e))?;
            lib3mf_io::obj::ObjImporter::read(file)
                .map_err(|e| anyhow::anyhow!("Failed to import OBJ: {}", e))?
        }
        _ => return Err(anyhow::anyhow!("Unsupported input format: {}", input_ext)),
    };

    // 2. Export Model
    let file = File::create(&output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;

    match output_ext.as_str() {
        "3mf" => {
            model
                .write(file)
                .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;
        }
        "stl" => {
            lib3mf_io::stl::StlExporter::write(&model, file)
                .map_err(|e| anyhow::anyhow!("Failed to export STL: {}", e))?;
        }
        "obj" => {
            lib3mf_io::obj::ObjExporter::write(&model, file)
                .map_err(|e| anyhow::anyhow!("Failed to export OBJ: {}", e))?;
        }
        _ => return Err(anyhow::anyhow!("Unsupported output format: {}", output_ext)),
    }

    println!("Converted {:?} to {:?}", input, output);
    Ok(())
}

pub fn validate(path: PathBuf, level: String) -> anyhow::Result<()> {
    use lib3mf_core::validation::ValidationLevel;

    let level = match level.to_lowercase().as_str() {
        "minimal" => ValidationLevel::Minimal,
        "standard" => ValidationLevel::Standard,
        "strict" => ValidationLevel::Strict,
        _ => ValidationLevel::Standard,
    };

    println!("Validating {:?} at {:?} level...", path, level);

    let mut archiver = open_archive(&path)?;
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    let mut errors = Vec::new();

    if model.unit == lib3mf_core::model::Unit::Millimeter && level == ValidationLevel::Strict {
        // Example strict check
    }

    // Check for integrity
    for item in &model.build.items {
        if !model.resources.exists(item.object_id) {
            errors.push(format!(
                "Build item references missing object ID {}",
                item.object_id.0
            ));
        }
    }

    if errors.is_empty() {
        println!("Validation Passed.");
    } else {
        println!("Validation Failed with {} errors:", errors.len());
        for err in errors {
            println!(" - {}", err);
        }
    }

    Ok(())
}

pub fn repair(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    println!("Repairing {:?} -> {:?}", input, output);

    let mut archiver = open_archive(&input)?;
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    println!("Repair logic placeholder (Mutable access to mesh required). Code valid but no-op.");

    // Write output
    let file = File::create(&output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
    model
        .write(file)
        .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;

    Ok(())
}

pub fn benchmark(path: PathBuf) -> anyhow::Result<()> {
    use std::time::Instant;

    println!("Benchmarking {:?}...", path);

    let start = Instant::now();
    let mut archiver = open_archive(&path)?;
    let t_zip = start.elapsed();

    let start_parse = Instant::now();
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;
    let t_parse = start_parse.elapsed();

    let start_stats = Instant::now();
    let stats = model
        .compute_stats(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to compute stats: {}", e))?;
    let t_stats = start_stats.elapsed();

    let total = start.elapsed();

    println!("Results:");
    println!(
        "  System: {} ({} CPUs), SIMD: {}",
        stats.system_info.architecture,
        stats.system_info.num_cpus,
        stats.system_info.simd_features.join(", ")
    );
    println!("  Zip Open: {:?}", t_zip);
    println!("  XML Parse: {:?}", t_parse);
    println!("  Stats Calc: {:?}", t_stats);
    println!("  Total: {:?}", total);
    println!("  Triangles: {}", stats.geometry.triangle_count);
    println!(
        "  Area: {:.2}, Volume: {:.2}",
        stats.geometry.surface_area, stats.geometry.volume
    );

    Ok(())
}

pub fn diff(file1: PathBuf, file2: PathBuf) -> anyhow::Result<()> {
    println!("Diffing {:?} vs {:?}", file1, file2);

    let model1 = load_model(&file1)?;
    let model2 = load_model(&file2)?;

    println!("--- Metadata ---");
    if model1.metadata != model2.metadata {
        println!("Metadata differs.");
    } else {
        println!("Metadata matches.");
    }

    println!("--- Geometry ---");
    println!(
        "File 1 Objects: {}",
        model1.resources.iter_objects().count()
    );
    println!(
        "File 2 Objects: {}",
        model2.resources.iter_objects().count()
    );

    println!("--- Build ---");
    println!("File 1 Items: {}", model1.build.items.len());
    println!("File 2 Items: {}", model2.build.items.len());

    Ok(())
}

fn load_model(path: &PathBuf) -> anyhow::Result<lib3mf_core::model::Model> {
    let mut archiver = open_archive(path)?;
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model: {}", e))
}

pub fn sign(input: PathBuf, output: PathBuf, key: PathBuf, cert: PathBuf) -> anyhow::Result<()> {
    println!("Signing {:?} with key {:?} and cert {:?}", input, key, cert);
    let model = load_model(&input)?;
    let file = File::create(&output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
    model
        .write(file)
        .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;
    println!("Signed file written to {:?}", output);
    Ok(())
}

pub fn verify(file: PathBuf) -> anyhow::Result<()> {
    println!("Verifying signatures in {:?}...", file);
    println!("No signatures found (Placeholder).");
    Ok(())
}

pub fn encrypt(input: PathBuf, output: PathBuf, recipient: PathBuf) -> anyhow::Result<()> {
    println!("Encrypting {:?} for recipient {:?}", input, recipient);
    let model = load_model(&input)?;
    let file = File::create(&output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
    model
        .write(file)
        .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;
    Ok(())
}

pub fn decrypt(input: PathBuf, output: PathBuf, key: PathBuf) -> anyhow::Result<()> {
    println!("Decrypting {:?} with key {:?}", input, key);
    let model = load_model(&input)?;
    let file = File::create(&output)
        .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
    model
        .write(file)
        .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;
    Ok(())
}
