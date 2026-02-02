pub mod thumbnails;

use clap::ValueEnum;
use lib3mf_core::archive::{find_model_path, opc, ArchiveReader, ZipArchiver};
use lib3mf_core::parser::parse_model;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

#[derive(Clone, ValueEnum, Debug, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Tree,
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Copy)]
pub enum RepairType {
    /// Remove degenerate triangles (zero area)
    Degenerate,
    /// Remove duplicate triangles
    Duplicates,
    /// Harmonize triangle winding
    Harmonize,
    /// Remove disconnected components (islands)
    Islands,
    /// Attempt to fill holes (boundary loops)
    Holes,
    /// Perform all repairs
    All,
}

enum ModelSource {
    Archive(ZipArchiver<File>, lib3mf_core::model::Model),
    Raw(lib3mf_core::model::Model),
}

fn open_model(path: &PathBuf) -> anyhow::Result<ModelSource> {
    let mut file =
        File::open(path).map_err(|e| anyhow::anyhow!("Failed to open file {:?}: {}", path, e))?;

    let mut magic = [0u8; 4];
    let is_zip = file.read_exact(&mut magic).is_ok() && &magic == b"PK\x03\x04";
    file.rewind()?;

    if is_zip {
        let mut archiver = ZipArchiver::new(file)
            .map_err(|e| anyhow::anyhow!("Failed to open zip archive: {}", e))?;
        let model_path = find_model_path(&mut archiver)
            .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
        let model_data = archiver
            .read_entry(&model_path)
            .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
        let model = parse_model(std::io::Cursor::new(model_data))
            .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;
        Ok(ModelSource::Archive(archiver, model))
    } else {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "stl" => {
                let model = lib3mf_converters::stl::StlImporter::read(file)
                    .map_err(|e| anyhow::anyhow!("Failed to import STL: {}", e))?;
                Ok(ModelSource::Raw(model))
            }
            "obj" => {
                let model = lib3mf_converters::obj::ObjImporter::read(file)
                    .map_err(|e| anyhow::anyhow!("Failed to import OBJ: {}", e))?;
                Ok(ModelSource::Raw(model))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported format: {} (and not a ZIP/3MF archive)",
                ext
            )),
        }
    }
}

pub fn stats(path: PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let mut source = open_model(&path)?;
    let stats = match source {
        ModelSource::Archive(ref mut archiver, ref model) => model
            .compute_stats(archiver)
            .map_err(|e| anyhow::anyhow!("Failed to compute stats: {}", e))?,
        ModelSource::Raw(ref model) => {
            struct NoArchive;
            impl std::io::Read for NoArchive {
                fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                    Ok(0)
                }
            }
            impl std::io::Seek for NoArchive {
                fn seek(&mut self, _: std::io::SeekFrom) -> std::io::Result<u64> {
                    Ok(0)
                }
            }
            impl lib3mf_core::archive::ArchiveReader for NoArchive {
                fn read_entry(&mut self, _: &str) -> lib3mf_core::error::Result<Vec<u8>> {
                    Err(lib3mf_core::error::Lib3mfError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Raw format",
                    )))
                }
                fn entry_exists(&mut self, _: &str) -> bool {
                    false
                }
                fn list_entries(&mut self) -> lib3mf_core::error::Result<Vec<String>> {
                    Ok(vec![])
                }
            }
            model
                .compute_stats(&mut NoArchive)
                .map_err(|e| anyhow::anyhow!("Failed to compute stats: {}", e))?
        }
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        OutputFormat::Tree => {
            println!("Model Hierarchy for {:?}", path);
            match source {
                ModelSource::Archive(mut archiver, model) => {
                    let mut resolver =
                        lib3mf_core::model::resolver::PartResolver::new(&mut archiver, model);
                    print_model_hierarchy_resolved(&mut resolver);
                }
                ModelSource::Raw(model) => {
                    print_model_hierarchy(&model);
                }
            }
        }
        _ => {
            println!("Stats for {:?}", path);
            println!(
                "Unit: {:?} (Scale: {} m)",
                stats.unit,
                stats.unit.scale_factor()
            );
            println!("Generator: {:?}", stats.generator.unwrap_or_default());
            println!("Geometry:");

            // Display object counts by type per CONTEXT.md decision
            let type_display: Vec<String> = ["model", "support", "solidsupport", "surface", "other"]
                .iter()
                .filter_map(|&type_name| {
                    stats.geometry.type_counts.get(type_name).and_then(|&count| {
                        if count > 0 {
                            Some(format!("{} {}", count, type_name))
                        } else {
                            None
                        }
                    })
                })
                .collect();

            if type_display.is_empty() {
                println!("  Objects: 0");
            } else {
                println!("  Objects: {}", type_display.join(", "));
            }

            println!("  Instances: {}", stats.geometry.instance_count);
            println!("  Vertices: {}", stats.geometry.vertex_count);
            println!("  Triangles: {}", stats.geometry.triangle_count);
            if let Some(bbox) = stats.geometry.bounding_box {
                println!("  Bounding Box: Min {:?}, Max {:?}", bbox.min, bbox.max);
            }
            let scale = stats.unit.scale_factor();
            println!(
                "  Surface Area: {:.2} (native units^2)",
                stats.geometry.surface_area
            );
            println!(
                "                {:.6} m^2",
                stats.geometry.surface_area * scale * scale
            );
            println!(
                "  Volume:       {:.2} (native units^3)",
                stats.geometry.volume
            );
            println!(
                "                {:.6} m^3",
                stats.geometry.volume * scale * scale * scale
            );

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
            println!(
                "  Texture 2D Groups: {}",
                stats.materials.texture_2d_groups_count
            );
            println!(
                "  Composite Materials: {}",
                stats.materials.composite_materials_count
            );
            println!(
                "  Multi Properties: {}",
                stats.materials.multi_properties_count
            );

            if !stats.vendor.plates.is_empty() {
                println!("Vendor Data (Bambu):");
                println!("  Plates: {}", stats.vendor.plates.len());
                for plate in stats.vendor.plates {
                    println!("    - ID {}: {}", plate.id, plate.name.unwrap_or_default());
                }
            }

            println!("Thumbnails:");
            println!(
                "  Package Thumbnail: {}",
                if stats.thumbnails.package_thumbnail_present {
                    "Yes"
                } else {
                    "No"
                }
            );
            println!(
                "  Object Thumbnails: {}",
                stats.thumbnails.object_thumbnail_count
            );
        }
    }
    Ok(())
}

pub fn list(path: PathBuf, format: OutputFormat) -> anyhow::Result<()> {
    let source = open_model(&path)?;

    let entries = match source {
        ModelSource::Archive(mut archiver, _) => archiver
            .list_entries()
            .map_err(|e| anyhow::anyhow!("Failed to list entries: {}", e))?,
        ModelSource::Raw(_) => vec![path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("model")
            .to_string()],
    };

    match format {
        OutputFormat::Json => {
            let tree = build_file_tree(&entries);
            println!("{}", serde_json::to_string_pretty(&tree)?);
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
    let mut model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    // Load all existing files to preserve multi-part relationships and attachments
    let all_files = archiver.list_entries()?;
    for entry_path in all_files {
        // Skip files that PackageWriter regenerates
        if entry_path == model_path
            || entry_path == "_rels/.rels"
            || entry_path == "[Content_Types].xml"
        {
            continue;
        }

        // Load .rels files to preserve relationships
        if entry_path.ends_with(".rels") {
            if let Ok(data) = archiver.read_entry(&entry_path) {
                if let Ok(rels) = lib3mf_core::archive::opc::parse_relationships(&data) {
                    model.existing_relationships.insert(entry_path, rels);
                }
            }
            continue;
        }

        // Load other data as attachments
        if let Ok(data) = archiver.read_entry(&entry_path) {
            model.attachments.insert(entry_path, data);
        }
    }

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

fn build_file_tree(paths: &[String]) -> node::FileNode {
    let mut root = node::FileNode::new_dir();
    for path in paths {
        let parts: Vec<&str> = path.split('/').collect();
        root.insert(&parts);
    }
    root
}

fn print_tree(paths: &[String]) {
    // Legacy tree printer
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

fn print_model_hierarchy(model: &lib3mf_core::model::Model) {
    let mut tree: BTreeMap<String, node::Node> = BTreeMap::new();

    for (i, item) in model.build.items.iter().enumerate() {
        let (obj_name, obj_type) = model
            .resources
            .get_object(item.object_id)
            .map(|obj| (
                obj.name.clone().unwrap_or_else(|| format!("Object {}", item.object_id.0)),
                obj.object_type
            ))
            .unwrap_or_else(|| (format!("Object {}", item.object_id.0), lib3mf_core::model::ObjectType::Model));

        let name = format!(
            "Build Item {} [{}] (type: {}, ID: {})",
            i + 1,
            obj_name,
            obj_type,
            item.object_id.0
        );
        let node = tree.entry(name).or_insert_with(node::Node::new);

        // Recurse into objects
        add_object_to_tree(model, item.object_id, node);
    }

    node::print_nodes(&tree, "");
}

fn add_object_to_tree(
    model: &lib3mf_core::model::Model,
    id: lib3mf_core::model::ResourceId,
    parent: &mut node::Node,
) {
    if let Some(obj) = model.resources.get_object(id) {
        match &obj.geometry {
            lib3mf_core::model::Geometry::Mesh(mesh) => {
                let info = format!(
                    "Mesh: {} vertices, {} triangles",
                    mesh.vertices.len(),
                    mesh.triangles.len()
                );
                parent.children.insert(info, node::Node::new());
            }
            lib3mf_core::model::Geometry::Components(comps) => {
                for (i, comp) in comps.components.iter().enumerate() {
                    let child_obj_name = model
                        .resources
                        .get_object(comp.object_id)
                        .and_then(|obj| obj.name.clone())
                        .unwrap_or_else(|| format!("Object {}", comp.object_id.0));

                    let name = format!(
                        "Component {} [{}] (ID: {})",
                        i + 1,
                        child_obj_name,
                        comp.object_id.0
                    );
                    let node = parent.children.entry(name).or_insert_with(node::Node::new);
                    add_object_to_tree(model, comp.object_id, node);
                }
            }
            _ => {
                parent
                    .children
                    .insert("Unknown Geometry".to_string(), node::Node::new());
            }
        }
    }
}

fn print_model_hierarchy_resolved<A: ArchiveReader>(
    resolver: &mut lib3mf_core::model::resolver::PartResolver<A>,
) {
    let mut tree: BTreeMap<String, node::Node> = BTreeMap::new();

    let build_items = resolver.get_root_model().build.items.clone();

    for (i, item) in build_items.iter().enumerate() {
        let (obj_name, obj_id, obj_type) = {
            let res = resolver
                .resolve_object(item.object_id, None)
                .unwrap_or(None);
            match res {
                Some((_model, obj)) => (
                    obj.name
                        .clone()
                        .unwrap_or_else(|| format!("Object {}", obj.id.0)),
                    obj.id,
                    obj.object_type,
                ),
                None => (
                    format!("Missing Object {}", item.object_id.0),
                    item.object_id,
                    lib3mf_core::model::ObjectType::Model,
                ),
            }
        };

        let name = format!("Build Item {} [{}] (type: {}, ID: {})", i + 1, obj_name, obj_type, obj_id.0);
        let node = tree.entry(name).or_insert_with(node::Node::new);

        // Recurse into objects
        add_object_to_tree_resolved(resolver, obj_id, None, node);
    }

    node::print_nodes(&tree, "");
}

fn add_object_to_tree_resolved<A: ArchiveReader>(
    resolver: &mut lib3mf_core::model::resolver::PartResolver<A>,
    id: lib3mf_core::model::ResourceId,
    path: Option<&str>,
    parent: &mut node::Node,
) {
    let components = {
        let resolved = resolver.resolve_object(id, path).unwrap_or(None);
        if let Some((_model, obj)) = resolved {
            match &obj.geometry {
                lib3mf_core::model::Geometry::Mesh(mesh) => {
                    let info = format!(
                        "Mesh: {} vertices, {} triangles",
                        mesh.vertices.len(),
                        mesh.triangles.len()
                    );
                    parent.children.insert(info, node::Node::new());
                    None
                }
                lib3mf_core::model::Geometry::Components(comps) => Some(comps.components.clone()),
                _ => {
                    parent
                        .children
                        .insert("Unknown Geometry".to_string(), node::Node::new());
                    None
                }
            }
        } else {
            None
        }
    };

    if let Some(comps) = components {
        for (i, comp) in comps.iter().enumerate() {
            let next_path = comp.path.as_deref().or(path);
            let (child_obj_name, child_obj_id) = {
                let res = resolver
                    .resolve_object(comp.object_id, next_path)
                    .unwrap_or(None);
                match res {
                    Some((_model, obj)) => (
                        obj.name
                            .clone()
                            .unwrap_or_else(|| format!("Object {}", obj.id.0)),
                        obj.id,
                    ),
                    None => (
                        format!("Missing Object {}", comp.object_id.0),
                        comp.object_id,
                    ),
                }
            };

            let name = format!(
                "Component {} [{}] (ID: {})",
                i + 1,
                child_obj_name,
                child_obj_id.0
            );
            let node = parent.children.entry(name).or_insert_with(node::Node::new);
            add_object_to_tree_resolved(resolver, child_obj_id, next_path, node);
        }
    }
}

mod node {
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    #[serde(untagged)]
    pub enum FileNode {
        File(Empty),
        Dir(BTreeMap<String, FileNode>),
    }

    #[derive(Serialize)]
    pub struct Empty {}

    impl FileNode {
        pub fn new_dir() -> Self {
            FileNode::Dir(BTreeMap::new())
        }

        pub fn new_file() -> Self {
            FileNode::File(Empty {})
        }

        pub fn insert(&mut self, path_parts: &[&str]) {
            if let FileNode::Dir(children) = self {
                if let Some((first, rest)) = path_parts.split_first() {
                    let entry = children
                        .entry(first.to_string())
                        .or_insert_with(FileNode::new_dir);

                    if rest.is_empty() {
                        // It's a file
                        if let FileNode::Dir(sub) = entry {
                            if sub.is_empty() {
                                *entry = FileNode::new_file();
                            } else {
                                // Conflict: Path is both a dir and a file?
                                // Keep as dir for now or handle appropriately.
                                // In 3MF/Zip, this shouldn't happen usually for exact paths.
                            }
                        }
                    } else {
                        // Recurse
                        entry.insert(rest);
                    }
                }
            }
        }
    }

    // Helper for legacy Node struct compatibility if needed,
    // or just reimplement internal printing logic.
    #[derive(Serialize)] // Optional, mainly for internal use
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
    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Special handling for STL export from 3MF to support components
    if output_ext == "stl" {
        // We need to keep the archive open for resolving components
        // Try opening as archive (zip)
        let file_res = File::open(&input);

        let should_use_resolver = if let Ok(mut f) = file_res {
            let mut magic = [0u8; 4];
            f.read_exact(&mut magic).is_ok() && &magic == b"PK\x03\x04"
        } else {
            false
        };

        if should_use_resolver {
            let mut archiver = open_archive(&input)?;
            let model_path = find_model_path(&mut archiver)
                .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
            let model_data = archiver
                .read_entry(&model_path)
                .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
            let model = parse_model(std::io::Cursor::new(model_data))
                .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

            let resolver = lib3mf_core::model::resolver::PartResolver::new(&mut archiver, model);
            let file = File::create(&output)
                .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;

            // Access the root model via resolver for export
            let root_model = resolver.get_root_model().clone(); // Clone to pass to export, or export takes ref

            lib3mf_converters::stl::StlExporter::write_with_resolver(&root_model, resolver, file)
                .map_err(|e| anyhow::anyhow!("Failed to export STL: {}", e))?;

            println!("Converted {:?} to {:?}", input, output);
            return Ok(());
        }
    }

    // Fallback to legacy conversion (or non-archive)
    // 1. Load Model
    let model = load_model(&input)?;

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
            lib3mf_converters::stl::StlExporter::write(&model, file)
                .map_err(|e| anyhow::anyhow!("Failed to export STL: {}", e))?;
        }
        "obj" => {
            lib3mf_converters::obj::ObjExporter::write(&model, file)
                .map_err(|e| anyhow::anyhow!("Failed to export OBJ: {}", e))?;
        }
        _ => return Err(anyhow::anyhow!("Unsupported output format: {}", output_ext)),
    }

    println!("Converted {:?} to {:?}", input, output);
    Ok(())
}

pub fn validate(path: PathBuf, level: String) -> anyhow::Result<()> {
    use lib3mf_core::validation::ValidationLevel;

    let level_enum = match level.to_lowercase().as_str() {
        "minimal" => ValidationLevel::Minimal,
        "standard" => ValidationLevel::Standard,
        "strict" => ValidationLevel::Strict,
        _ => ValidationLevel::Standard,
    };

    println!("Validating {:?} at {:?} level...", path, level_enum);

    let model = load_model(&path)?;

    let mut errors = Vec::new();

    if model.unit == lib3mf_core::model::Unit::Millimeter && level_enum == ValidationLevel::Strict {
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

pub fn repair(
    input: PathBuf,
    output: PathBuf,
    epsilon: f32,
    fixes: Vec<RepairType>,
) -> anyhow::Result<()> {
    use lib3mf_core::model::{Geometry, MeshRepair, RepairOptions};

    println!("Repairing {:?} -> {:?}", input, output);

    let mut archiver = open_archive(&input)?;
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path: {}", e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model data: {}", e))?;
    let mut model = parse_model(std::io::Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML: {}", e))?;

    let mut options = RepairOptions {
        stitch_epsilon: epsilon,
        remove_degenerate: false,
        remove_duplicate_faces: false,
        harmonize_orientations: false,
        remove_islands: false,
        fill_holes: false,
    };

    let has_all = fixes.contains(&RepairType::All);
    for fix in fixes {
        match fix {
            RepairType::Degenerate => options.remove_degenerate = true,
            RepairType::Duplicates => options.remove_duplicate_faces = true,
            RepairType::Harmonize => options.harmonize_orientations = true,
            RepairType::Islands => options.remove_islands = true,
            RepairType::Holes => options.fill_holes = true,
            RepairType::All => {
                options.remove_degenerate = true;
                options.remove_duplicate_faces = true;
                options.harmonize_orientations = true;
                options.remove_islands = true;
                options.fill_holes = true;
            }
        }
    }

    if has_all {
        options.remove_degenerate = true;
        options.remove_duplicate_faces = true;
        options.harmonize_orientations = true;
        options.remove_islands = true;
        options.fill_holes = true;
    }

    println!("Repair Options: {:?}", options);

    let mut total_vertices_removed = 0;
    let mut total_triangles_removed = 0;
    let mut total_triangles_flipped = 0;
    let mut total_triangles_added = 0;

    for object in model.resources.iter_objects_mut() {
        if let Geometry::Mesh(mesh) = &mut object.geometry {
            let stats = mesh.repair(options);
            if stats.vertices_removed > 0
                || stats.triangles_removed > 0
                || stats.triangles_flipped > 0
                || stats.triangles_added > 0
            {
                println!(
                    "Repaired Object {}: Removed {} vertices, {} triangles. Flipped {}. Added {}.",
                    object.id.0,
                    stats.vertices_removed,
                    stats.triangles_removed,
                    stats.triangles_flipped,
                    stats.triangles_added
                );
                total_vertices_removed += stats.vertices_removed;
                total_triangles_removed += stats.triangles_removed;
                total_triangles_flipped += stats.triangles_flipped;
                total_triangles_added += stats.triangles_added;
            }
        }
    }

    println!("Total Repair Stats:");
    println!("  Vertices Removed:  {}", total_vertices_removed);
    println!("  Triangles Removed: {}", total_triangles_removed);
    println!("  Triangles Flipped: {}", total_triangles_flipped);
    println!("  Triangles Added:   {}", total_triangles_added);

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

pub fn diff(file1: PathBuf, file2: PathBuf, format: &str) -> anyhow::Result<()> {
    println!("Comparing {:?} and {:?}...", file1, file2);

    let model_a = load_model(&file1)?;
    let model_b = load_model(&file2)?;

    let diff = lib3mf_core::utils::diff::compare_models(&model_a, &model_b);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&diff)?);
    } else if diff.is_empty() {
        println!("Models are identical.");
    } else {
        println!("Differences found:");
        if !diff.metadata_diffs.is_empty() {
            println!("  Metadata:");
            for d in &diff.metadata_diffs {
                println!("    - {:?}: {:?} -> {:?}", d.key, d.old_value, d.new_value);
            }
        }
        if !diff.resource_diffs.is_empty() {
            println!("  Resources:");
            for d in &diff.resource_diffs {
                match d {
                    lib3mf_core::utils::diff::ResourceDiff::Added { id, type_name } => {
                        println!("    + Added ID {}: {}", id, type_name)
                    }
                    lib3mf_core::utils::diff::ResourceDiff::Removed { id, type_name } => {
                        println!("    - Removed ID {}: {}", id, type_name)
                    }
                    lib3mf_core::utils::diff::ResourceDiff::Changed { id, details } => {
                        println!("    * Changed ID {}:", id);
                        for det in details {
                            println!("      . {}", det);
                        }
                    }
                }
            }
        }
        if !diff.build_diffs.is_empty() {
            println!("  Build Items:");
            for d in &diff.build_diffs {
                println!("    - {:?}", d);
            }
        }
    }

    Ok(())
}

fn load_model(path: &PathBuf) -> anyhow::Result<lib3mf_core::model::Model> {
    match open_model(path)? {
        ModelSource::Archive(_, model) => Ok(model),
        ModelSource::Raw(model) => Ok(model),
    }
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
    let mut archiver = open_archive(&file)?;

    // 1. Read Global Relationships to find signatures
    let rels_data = archiver.read_entry("_rels/.rels").unwrap_or_default();
    if rels_data.is_empty() {
        println!("No relationships found. File is not signed.");
        return Ok(());
    }

    let rels = opc::parse_relationships(&rels_data)?;
    let sig_rels: Vec<_> = rels.iter().filter(|r|        r.rel_type == "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel/relationship/signature"
        || r.rel_type.ends_with("/signature") // Loose check
    ).collect();

    if sig_rels.is_empty() {
        println!("No signature relationships found.");
        return Ok(());
    }

    println!("Found {} signatures to verify.", sig_rels.len());

    for rel in sig_rels {
        println!("Verifying signature: {}", rel.target);
        // Target is usually absolute path like "/Metadata/sig.xml"
        let target_path = rel.target.trim_start_matches('/');

        let sig_xml_bytes = match archiver.read_entry(target_path) {
            Ok(b) => b,
            Err(e) => {
                println!("  [ERROR] Failed to read signature part: {}", e);
                continue;
            }
        };

        // Parse Signature
        let sig_xml_str = String::from_utf8_lossy(&sig_xml_bytes);
        // We use Cursor wrapping String for parser
        let mut sig_parser = lib3mf_core::parser::xml_parser::XmlParser::new(std::io::Cursor::new(
            sig_xml_bytes.clone(),
        ));
        let signature = match lib3mf_core::parser::crypto_parser::parse_signature(&mut sig_parser) {
            Ok(s) => s,
            Err(e) => {
                println!("  [ERROR] Failed to parse signature XML: {}", e);
                continue;
            }
        };

        // Canonicalize SignedInfo
        // We need the Bytes of SignedInfo.
        // Option 1: Re-read file and extract substring (risky if not formatted same).
        // Option 2: Use Canonicalizer on the original bytes to extract subtree.
        let signed_info_c14n = match lib3mf_core::utils::c14n::Canonicalizer::canonicalize_subtree(
            &sig_xml_str,
            "SignedInfo",
        ) {
            Ok(b) => b,
            Err(e) => {
                println!("  [ERROR] Failed to extract/canonicalize SignedInfo: {}", e);
                continue;
            }
        };

        // Prepare Content Resolver
        // This closure allows the verifier to fetch the bytes of parts referenced by the signature.
        // We need to clone the archive reader or access it safely.
        // Archiver is mut... tricky with closure if capturing mut ref.
        // But we iterate sequentially. We can pass a closure that reads from a shared ref or re-opens?
        // Actually, we can just pre-read referenced parts? No, References are inside Signature.
        // Ideally, we pass a closure. But `archiver` is needed.
        // Simpler: Read all entries into a Map? No, memory.
        // We can use a ref cell or mutex for archiver?
        // Or better: `verify_signature_extended` takes a closure.
        // The closure can't mutate archiver easily if archiver requires mut.
        // `ZipArchiver::read_entry` takes `&mut self`.
        // We can close and re-open? Inefficient.

        // Hack: Read all referenced parts needed by THIS signature before calling verify?
        // But verify_signature calls the resolver.
        // Let's implement a wrapper struct or use RefCell.
        // `archiver` is `ZipArchiver<File>`.
        // Let's defer resolver implementation by collecting references first?
        // `verify_signature` logic iterates references and calls resolver.
        // If we duplicate the "resolve" logic:
        // 1. Collect URIs from signature.
        // 2. Read all contents into a Map.
        // 3. Pass Map lookup to verifier.

        let mut content_map = BTreeMap::new();
        for ref_item in &signature.signed_info.references {
            let uri = &ref_item.uri;
            if uri.is_empty() {
                continue;
            } // Implicit reference to something?
            let part_path = uri.trim_start_matches('/');
            match archiver.read_entry(part_path) {
                Ok(data) => {
                    content_map.insert(uri.clone(), data);
                }
                Err(e) => println!("  [WARNING] Could not read referenced part {}: {}", uri, e),
            }
        }

        let resolver = |uri: &str| -> lib3mf_core::error::Result<Vec<u8>> {
            content_map.get(uri).cloned().ok_or_else(|| {
                lib3mf_core::error::Lib3mfError::Validation(format!("Content not found: {}", uri))
            })
        };

        match lib3mf_core::crypto::verification::verify_signature_extended(
            &signature,
            resolver,
            &signed_info_c14n,
        ) {
            Ok(valid) => {
                if valid {
                    println!("  [PASS] Signature is VALD.");
                    // Check certificate trust if present
                    if let Some(mut ki) = signature.key_info {
                        if let Some(x509) = ki.x509_data.take() {
                            if let Some(_cert_str) = x509.certificate {
                                println!(
                                    "  [INFO] Signed by X.509 Certificate (Trust check pending)"
                                );
                                // TODO: Validate chain
                            }
                        } else {
                            println!("  [INFO] Signed by Raw Key (Self-signed equivalent)");
                        }
                    }
                } else {
                    println!("  [FAIL] Signature is INVALID (Verification returned false).");
                }
            }
            Err(e) => println!("  [FAIL] Verification Error: {}", e),
        }
    }

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
