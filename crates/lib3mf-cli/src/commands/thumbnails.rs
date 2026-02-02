use anyhow::Result;
use lib3mf_core::archive::ArchiveReader; // Trait must be in scope
use lib3mf_core::model::ResourceId;
use std::fs::{self, File};
use std::io::Write; // Removed Read
use std::path::PathBuf;

pub fn run(
    file: PathBuf,
    list: bool,
    extract: Option<PathBuf>,
    inject: Option<PathBuf>,
    oid: Option<u32>,
) -> Result<()> {
    if list {
        run_list(&file)?;
        return Ok(());
    }

    if let Some(dir) = extract {
        run_extract(&file, dir)?;
        return Ok(());
    }

    if let Some(img_path) = inject {
        run_inject(&file, img_path, oid)?;
        return Ok(());
    }

    // Default or help usage if no flags?
    println!("Please specify --list, --extract <DIR>, or --inject <IMG>.");
    Ok(())
}

fn run_list(file: &PathBuf) -> Result<()> {
    let mut archiver = crate::commands::open_archive(file)?;
    let model_path = lib3mf_core::archive::find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = lib3mf_core::parser::parse_model(std::io::Cursor::new(model_data))?;

    println!("Thumbnail Status for: {:?}", file);

    // Package Thumbnail check
    let pkg_thumb = archiver.entry_exists("Metadata/thumbnail.png")
        || archiver.entry_exists("/Metadata/thumbnail.png");
    println!(
        "Package Thumbnail: {}",
        if pkg_thumb { "Yes" } else { "No" }
    );

    // Parse Model Relationships to resolve thumbnail IDs to paths
    let model_rels_path = {
        let path = std::path::Path::new(&model_path);
        if let Some(parent) = path.parent() {
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            parent
                .join("_rels")
                .join(format!("{}.rels", fname))
                .to_string_lossy()
                .replace("\\", "/")
        } else {
            format!("_rels/{}.rels", model_path)
        }
    };

    let model_rels_data = archiver.read_entry(&model_rels_path).unwrap_or_default();
    let model_rels = if !model_rels_data.is_empty() {
        lib3mf_core::archive::opc::parse_relationships(&model_rels_data).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Build Map: Relationship ID -> Target Path
    let mut rel_map = std::collections::HashMap::new();
    for rel in model_rels {
        rel_map.insert(rel.id, rel.target);
    }

    // Objects
    if model.resources.iter_objects().count() > 0 {
        println!("\nObjects:");
        for obj in model.resources.iter_objects() {
            let thumb_display = if let Some(thumb_ref) = &obj.thumbnail {
                // Resolve relationship ID to actual path
                rel_map
                    .get(thumb_ref)
                    .map(|s| s.as_str())
                    .unwrap_or(thumb_ref) // Fallback to raw value if not found
            } else {
                "None"
            };
            let name = obj.name.as_deref().unwrap_or("-");
            println!(
                "  ID: {:<4} | Name: {:<20} | Thumbnail: {}",
                obj.id.0, name, thumb_display
            );
        }
    } else {
        println!("\nNo objects found.");
    }
    Ok(())
}

// ... run_extract is fine (I just fixed imports for it) ... I should preserve it.
// But replace_file_content replaces a block.
// I need to skip run_extract replacement if I can target correctly.
// I will target only run_list and run_inject.
// BUT `run_extract` is BETWEEN them in the file Step 133?
// No, Step 133 order: run, run_list, run_extract, run_inject.
// So I should replace run_list separate from run_inject?
// Or replace the whole file content after `run`?
// I'll replace `run_list` first.

// Wait, I can't overwrite `run_extract` because I made changes to it in Step 188.
// I should inspect the file again to be safe on line numbers?
// Step 239 was minimal diff.
// I'll view the file to get current state.
// Better safe than sorry.

fn run_inject(file: &PathBuf, img_path: PathBuf, oid: Option<u32>) -> Result<()> {
    // We need to preserve existing attachments!
    let mut archiver = crate::commands::open_archive(file)?;
    let model_path = lib3mf_core::archive::find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let mut model = lib3mf_core::parser::parse_model(std::io::Cursor::new(model_data))?;

    // Load ALL existing files as attachments (excluding system files)
    // Also load .rels files to preserve multi-part relationships
    let all_files = archiver.list_entries()?;
    for entry_path in all_files {
        // Skip files that PackageWriter regenerates
        if entry_path == model_path
            || entry_path == "_rels/.rels"
            || entry_path == "[Content_Types].xml"
        {
            continue;
        }

        // Load .rels files separately to preserve relationships
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

    println!("Injecting {:?} into {:?}", img_path, file);

    let img_data = fs::read(&img_path)?;

    if let Some(id) = oid {
        // Object Injection
        let rid = ResourceId(id);

        let mut found = false;
        for obj in model.resources.iter_objects_mut() {
            if obj.id == rid {
                // Set path
                let path = format!("3D/Textures/thumb_{}.png", id);
                obj.thumbnail = Some(path.clone());

                // Add attachment
                model.attachments.insert(path, img_data.clone());
                println!("Updated Object {} thumbnail.", id);
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!("Object ID {} not found.", id);
        }
    } else {
        // Package Injection
        let path = "Metadata/thumbnail.png".to_string();
        model.attachments.insert(path, img_data);
        println!("Updated Package Thumbnail.");
    }

    // Write back
    let f = File::create(file)?;
    model
        .write(f)
        .map_err(|e| anyhow::anyhow!("Failed to write 3MF: {}", e))?;

    println!("Done.");
    Ok(())
}

fn run_extract(file: &PathBuf, dir: PathBuf) -> Result<()> {
    // We need the archiver to read relationships
    let mut archiver = crate::commands::open_archive(file)?;

    // Parse Model (to get objects)
    // Note: open_archive returns ZipArchiver. We need to find model path.
    let model_path_str = lib3mf_core::archive::find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path_str)?;
    let model = lib3mf_core::parser::parse_model(std::io::Cursor::new(model_data))?;

    // Load Attachments (manually, since parse_model doesn't use archiver automatically to populate attachments?
    // Wait, parse_model ONLY parses XML. It doesn't load attachments.
    // The previously used `open_model` helper did `ZipArchiver::new` but returned `Model`.
    // Wait, `open_model` in `commands.rs` (Step 128) lines 38-81:
    // It returns `ModelSource`.
    // `ModelSource::Archive` holds `ZipArchiver` and `Model`.
    // But `parse_model` returns `Model`.
    // The `Model` returned by `parse_model` has EMPTY attachments!
    // Attachments are loaded by `Model::load_attachments`?
    // `lib3mf-core` allows lazy loading or expected the caller to fill `attachments`?
    // Let's check `open_model` implementation again.
    // Line 54: `let model = parse_model(...)`.
    // It DOES NOT load attachments!
    // So `model.attachments` is empty in `thumbnails.rs` when using `open_model`!
    // This is a bug in my `thumbnails.rs` implementation (and potentially `stats` if it relies on attachments).
    // `stats` relies on `model.compute_stats` which takes `archiver`.
    // `lib3mf-core`'s `compute_stats` doesn't load attachments into the model struct, but accesses archiver.
    // But `stats.rs` (my update) checks `self.attachments`.
    // THIS MEANS `stats` (CLI) will report "No Package Thumbnail" because `model.attachments` is empty.

    // I need to fix `thumbnails.rs` to load attachments or access them via archiver.
    // And `stats_impl.rs` check `self.attachments` is WRONG if they aren't loaded.
    // `stats_impl.rs` should check `archiver` for the file existence!

    // Correction for `stats_impl.rs`:
    // It has `archiver` available in `compute_stats`.
    // `let pkg_thumb = archiver.entry_exists("Metadata/thumbnail.png") || archiver.entry_exists("/Metadata/thumbnail.png");`

    // Correction for `thumbnails.rs`:
    // I need to use `archiver` to read files.

    fs::create_dir_all(&dir)?;
    println!("Extracting thumbnails to {:?}...", dir);

    // 1. Package Thumbnail
    // Check various common paths or check relationships?
    // Ideally check _rels/.rels to find the target of the thumbnail relationship.
    // Parsing _rels/.rels
    let global_rels_data = archiver.read_entry("_rels/.rels").unwrap_or_default();
    let global_rels = if !global_rels_data.is_empty() {
        lib3mf_core::archive::opc::parse_relationships(&global_rels_data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut pkg_thumb_path = None;
    for rel in global_rels {
        if rel.rel_type.ends_with("metadata/thumbnail") {
            pkg_thumb_path = Some(rel.target);
            break;
        }
    }
    // Fallback
    if pkg_thumb_path.is_none() && archiver.entry_exists("Metadata/thumbnail.png") {
        pkg_thumb_path = Some("Metadata/thumbnail.png".to_string());
    }

    if let Some(path) = pkg_thumb_path {
        if let Ok(data) = archiver.read_entry(&path) {
            let out = dir.join("package_thumbnail.png");
            let mut f = File::create(&out)?;
            f.write_all(&data)?;
            println!("  Extracted Package Thumbnail: {:?}", out);
        }
    }

    // 2. Object Thumbnails
    // Parse Model Relationships
    // Path is e.g. "3D/_rels/3dmodel.model.rels" (if main model is "3D/3dmodel.model")
    // We need to construct the rels path from `model_path_str`.
    // e.g. "3D/3dmodel.model" -> "3D/_rels/3dmodel.model.rels"
    let model_rels_path = {
        let path = std::path::Path::new(&model_path_str);
        if let Some(parent) = path.parent() {
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            parent
                .join("_rels")
                .join(format!("{}.rels", fname))
                .to_string_lossy()
                .replace("\\", "/")
        } else {
            format!("_rels/{}.rels", model_path_str) // Unlikely for root file but possible
        }
    };

    let model_rels_data = archiver.read_entry(&model_rels_path).unwrap_or_default();
    let model_rels = if !model_rels_data.is_empty() {
        lib3mf_core::archive::opc::parse_relationships(&model_rels_data).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Build Map ID -> Target
    let mut rel_map = std::collections::HashMap::new();
    for rel in model_rels {
        rel_map.insert(rel.id, rel.target);
    }

    for obj in model.resources.iter_objects() {
        if let Some(thumb_ref) = &obj.thumbnail {
            // Resolve ref
            let target = rel_map.get(thumb_ref).cloned().or_else(|| {
                // Maybe it IS a path (legacy or incorrectly written)?
                Some(thumb_ref.clone())
            });

            if let Some(path) = target {
                // Read from archiver
                let lookup_path = path.trim_start_matches('/');
                if let Ok(bytes) = archiver.read_entry(lookup_path) {
                    let fname = format!("obj_{}_thumbnail.png", obj.id.0);
                    let out = dir.join(fname);
                    let mut f = File::create(&out)?;
                    f.write_all(&bytes)?;
                    println!("  Extracted Object {} Thumbnail: {:?}", obj.id.0, out);
                } else {
                    println!(
                        "  Warning: Object {} thumbnail target '{}' not found in archive.",
                        obj.id.0, lookup_path
                    );
                }
            }
        }
    }

    Ok(())
}
