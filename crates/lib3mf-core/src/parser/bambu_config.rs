//! Bambu Studio config file parsers.
//!
//! Parses the vendor-specific configuration files embedded in Bambu Studio 3MF archives:
//! - `slice_info.config` (XML): Print time/weight estimates, filament usage, slicer warnings
//! - `model_settings.config` (XML): Per-object metadata, parts, plates, assembly transforms
//! - `project_settings.config` (JSON): Printer model, layer height, filament settings
//! - Per-profile configs (JSON): `filament_settings_N.config`, `machine_settings_N.config`, etc.
//!
//! All parsers handle missing or malformed input gracefully without returning errors,
//! following the principle that vendor data enrichment should not block model loading.

use crate::error::Result;
use crate::model::stats::{
    AssemblyItem, BambuMeshStat, BambuObjectMetadata, BambuPartMetadata, BambuProfileConfig,
    BambuProjectSettings, PartSubtype, PlateInfo, PlateModelInstance, SlicerWarning,
};
use crate::parser::xml_parser::{XmlParser, get_attribute};
use quick_xml::events::Event;
use serde_json::Value;
use std::io::Cursor;

// ── Slice info return type ────────────────────────────────────────────────────

/// Parsed data from `Metadata/slice_info.config`.
#[derive(Debug, Clone, Default)]
pub struct SliceInfoData {
    /// Slicer client type, e.g. "slicer"
    pub client_type: Option<String>,
    /// Slicer client version, e.g. "01.10.02.73"
    pub client_version: Option<String>,
    /// Per-plate slicing results
    pub plates: Vec<SlicePlateInfo>,
}

/// Slicing results for a single plate.
#[derive(Debug, Clone, Default)]
pub struct SlicePlateInfo {
    /// Plate index (1-based).
    pub id: u32,
    /// Estimated print time in seconds.
    pub prediction: Option<u32>, // seconds
    /// Estimated total filament weight in grams.
    pub weight: Option<f32>, // grams
    /// Per-filament usage statistics.
    pub filaments: Vec<SliceFilamentUsage>,
    /// Slicer warnings for this plate.
    pub warnings: Vec<SlicerWarning>,
    /// Objects included on this plate.
    pub objects: Vec<SliceObjectInfo>,
}

/// Per-filament usage data for a plate.
#[derive(Debug, Clone, Default)]
pub struct SliceFilamentUsage {
    /// Filament slot index.
    pub id: u32,
    /// AMS tray info index.
    pub tray_info_idx: Option<String>,
    /// Filament type string (e.g., `"PLA"`).
    pub type_: Option<String>,
    /// Display color in hex format.
    pub color: Option<String>,
    /// Filament used in meters.
    pub used_m: Option<f32>,
    /// Filament used in grams.
    pub used_g: Option<f32>,
}

/// Object participation record within a plate.
#[derive(Debug, Clone, Default)]
pub struct SliceObjectInfo {
    /// Object resource ID.
    pub id: u32,
    /// Object display name.
    pub name: Option<String>,
}

// ── Model settings return type ────────────────────────────────────────────────

/// Parsed data from `Metadata/model_settings.config`.
#[derive(Debug, Clone, Default)]
pub struct ModelSettingsData {
    /// Plate layout and gcode/thumbnail paths
    pub plates: Vec<PlateInfo>,
    /// Per-object metadata (name, extruder, parts, overrides)
    pub objects: Vec<BambuObjectMetadata>,
    /// Assembly transforms for build instances
    pub assembly: Vec<AssemblyItem>,
}

// ── parse_slice_info ──────────────────────────────────────────────────────────

/// Parse `Metadata/slice_info.config` (XML).
///
/// Returns [`SliceInfoData`] on success. Returns `Ok(Default::default())` on
/// empty or malformed content so callers can always proceed.
pub fn parse_slice_info(content: &[u8]) -> Result<SliceInfoData> {
    if content.is_empty() {
        return Ok(SliceInfoData::default());
    }

    let mut parser = XmlParser::new(Cursor::new(content));
    let mut data = SliceInfoData::default();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) => {
                match e.name().as_ref() {
                    b"header_item" => {
                        // Flat header items directly under <header>
                        let key = get_attribute(&e, b"key");
                        let value = get_attribute(&e, b"value");
                        if let (Some(k), Some(v)) = (key, value) {
                            match k.as_ref() {
                                "X-BBL-Client-Type" => {
                                    data.client_type = Some(v.into_owned());
                                }
                                "X-BBL-Client-Version" => {
                                    data.client_version = Some(v.into_owned());
                                }
                                _ => {}
                            }
                        }
                    }
                    b"plate" => {
                        // Parse <plate> element and its children
                        let plate = parse_slice_plate(&mut parser)?;
                        data.plates.push(plate);
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(data)
}

fn parse_slice_plate(parser: &mut XmlParser<Cursor<&[u8]>>) -> Result<SlicePlateInfo> {
    let mut plate = SlicePlateInfo::default();

    loop {
        match parser.read_next_event()? {
            Event::Empty(e) | Event::Start(e) => match e.name().as_ref() {
                b"metadata" => {
                    let key = get_attribute(&e, b"key");
                    let value = get_attribute(&e, b"value");
                    if let (Some(k), Some(v)) = (key, value) {
                        match k.as_ref() {
                            "index" => {
                                if let Ok(id) = v.parse::<u32>() {
                                    plate.id = id;
                                }
                            }
                            "prediction" => {
                                plate.prediction = v.parse::<u32>().ok();
                            }
                            "weight" => {
                                plate.weight = v.parse::<f32>().ok();
                            }
                            _ => {}
                        }
                    }
                }
                b"filament" => {
                    let id = get_attribute(&e, b"id")
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    let tray_info_idx = get_attribute(&e, b"tray_info_idx").map(|v| v.into_owned());
                    let type_ = get_attribute(&e, b"type").map(|v| v.into_owned());
                    let color = get_attribute(&e, b"color").map(|v| v.into_owned());
                    let used_m = get_attribute(&e, b"used_m").and_then(|v| v.parse::<f32>().ok());
                    let used_g = get_attribute(&e, b"used_g").and_then(|v| v.parse::<f32>().ok());
                    plate.filaments.push(SliceFilamentUsage {
                        id,
                        tray_info_idx,
                        type_,
                        color,
                        used_m,
                        used_g,
                    });
                }
                b"warning" => {
                    let msg = get_attribute(&e, b"msg")
                        .map(|v| v.into_owned())
                        .unwrap_or_default();
                    let level = get_attribute(&e, b"level").map(|v| v.into_owned());
                    let error_code = get_attribute(&e, b"error_code").map(|v| v.into_owned());
                    plate.warnings.push(SlicerWarning {
                        msg,
                        level,
                        error_code,
                    });
                }
                b"object" => {
                    let id = get_attribute(&e, b"identify_id")
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    let name = get_attribute(&e, b"name").map(|v| v.into_owned());
                    plate.objects.push(SliceObjectInfo { id, name });
                }
                _ => {}
            },
            Event::End(end) if end.name().as_ref() == b"plate" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(plate)
}

// ── parse_model_settings ──────────────────────────────────────────────────────

/// Parse `Metadata/model_settings.config` (XML).
///
/// Returns enriched [`ModelSettingsData`] containing plates, objects, and assembly.
/// Returns `Ok(Default::default())` on empty or malformed content.
pub fn parse_model_settings(content: &[u8]) -> Result<ModelSettingsData> {
    if content.is_empty() {
        return Ok(ModelSettingsData::default());
    }

    let mut parser = XmlParser::new(Cursor::new(content));
    let mut data = ModelSettingsData::default();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"object" => {
                    let id = get_attribute(&e, b"id")
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    let obj = parse_model_object(&mut parser, id)?;
                    data.objects.push(obj);
                }
                b"plate" => {
                    let plate = parse_model_plate(&mut parser)?;
                    data.plates.push(plate);
                }
                b"assemble" => {
                    let items = parse_assemble(&mut parser)?;
                    data.assembly.extend(items);
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(data)
}

fn parse_model_object(
    parser: &mut XmlParser<Cursor<&[u8]>>,
    id: u32,
) -> Result<BambuObjectMetadata> {
    let mut obj = BambuObjectMetadata {
        id,
        ..Default::default()
    };

    loop {
        match parser.read_next_event()? {
            Event::Empty(e) | Event::Start(e) => {
                match e.name().as_ref() {
                    b"metadata" => {
                        // Two forms:
                        // <metadata key="name" value="..." />  → keyed metadata
                        // <metadata face_count="225154"/>      → attribute-style metadata (no key attr)
                        let key = get_attribute(&e, b"key");
                        let value = get_attribute(&e, b"value");
                        if let Some(k) = key {
                            if let Some(v) = value {
                                match k.as_ref() {
                                    "name" => obj.name = Some(v.into_owned()),
                                    "extruder" => {
                                        obj.extruder = v.parse::<u32>().ok();
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            // attribute-style: <metadata face_count="N"/>
                            if let Some(fc) = get_attribute(&e, b"face_count") {
                                obj.face_count = fc.parse::<u64>().ok();
                            }
                        }
                    }
                    b"part" => {
                        let part_id = get_attribute(&e, b"id")
                            .and_then(|v| v.parse::<u32>().ok())
                            .unwrap_or(0);
                        let subtype = get_attribute(&e, b"subtype")
                            .map(|v| PartSubtype::parse(&v))
                            .unwrap_or_default();
                        // <part> has a full Start event (not Empty), parse its children
                        let part = parse_part(parser, part_id, subtype)?;
                        obj.parts.push(part);
                    }
                    _ => {}
                }
            }
            Event::End(end) if end.name().as_ref() == b"object" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(obj)
}

fn parse_part(
    parser: &mut XmlParser<Cursor<&[u8]>>,
    id: u32,
    subtype: PartSubtype,
) -> Result<BambuPartMetadata> {
    let mut part = BambuPartMetadata {
        id,
        subtype,
        ..Default::default()
    };

    // Known metadata keys that go into dedicated fields (not print_overrides)
    const KNOWN_KEYS: &[&str] = &[
        "name",
        "matrix",
        "source_object_id",
        "source_volume_id",
        "source_offset_x",
        "source_offset_y",
        "source_offset_z",
        "source_in_inches",
    ];

    loop {
        match parser.read_next_event()? {
            Event::Empty(e) | Event::Start(e) => match e.name().as_ref() {
                b"metadata" => {
                    let key = get_attribute(&e, b"key");
                    let value = get_attribute(&e, b"value");
                    if let (Some(k), Some(v)) = (key, value) {
                        let k_str = k.as_ref();
                        match k_str {
                            "name" => part.name = Some(v.into_owned()),
                            "matrix" => part.matrix = Some(v.into_owned()),
                            "source_volume_id" => {
                                let src = part.source.get_or_insert_with(Default::default);
                                src.volume_id = v.parse::<u32>().ok();
                            }
                            "source_offset_x" => {
                                let src = part.source.get_or_insert_with(Default::default);
                                src.offset_x = v.parse::<f64>().ok();
                            }
                            "source_offset_y" => {
                                let src = part.source.get_or_insert_with(Default::default);
                                src.offset_y = v.parse::<f64>().ok();
                            }
                            "source_offset_z" => {
                                let src = part.source.get_or_insert_with(Default::default);
                                src.offset_z = v.parse::<f64>().ok();
                            }
                            other => {
                                if !KNOWN_KEYS.contains(&other) {
                                    part.print_overrides
                                        .insert(other.to_string(), v.into_owned());
                                }
                            }
                        }
                    }
                }
                b"mesh_stat" => {
                    let edges_fixed =
                        get_attribute(&e, b"edges_fixed").and_then(|v| v.parse::<u32>().ok());
                    let degenerate_facets =
                        get_attribute(&e, b"degenerate_facets").and_then(|v| v.parse::<u32>().ok());
                    let facets_removed =
                        get_attribute(&e, b"facets_removed").and_then(|v| v.parse::<u32>().ok());
                    let facets_reversed =
                        get_attribute(&e, b"facets_reversed").and_then(|v| v.parse::<u32>().ok());
                    let backwards_edges =
                        get_attribute(&e, b"backwards_edges").and_then(|v| v.parse::<u32>().ok());
                    part.mesh_stat = Some(BambuMeshStat {
                        edges_fixed,
                        degenerate_facets,
                        facets_removed,
                        facets_reversed,
                        backwards_edges,
                    });
                }
                _ => {}
            },
            Event::End(end) if end.name().as_ref() == b"part" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(part)
}

fn parse_model_plate(parser: &mut XmlParser<Cursor<&[u8]>>) -> Result<PlateInfo> {
    let mut plate = PlateInfo::default();

    loop {
        match parser.read_next_event()? {
            Event::Empty(e) | Event::Start(e) => match e.name().as_ref() {
                b"metadata" => {
                    let key = get_attribute(&e, b"key");
                    let value = get_attribute(&e, b"value");
                    if let (Some(k), Some(v)) = (key, value) {
                        match k.as_ref() {
                            "plater_id" => {
                                if let Ok(id) = v.parse::<u32>() {
                                    plate.id = id;
                                }
                            }
                            "plater_name" => {
                                if !v.is_empty() {
                                    plate.name = Some(v.into_owned());
                                }
                            }
                            "locked" => {
                                plate.locked = v == "true";
                            }
                            "gcode_file" => {
                                plate.gcode_file = Some(v.into_owned());
                            }
                            "thumbnail_file" => {
                                plate.thumbnail_file = Some(v.into_owned());
                            }
                            _ => {}
                        }
                    }
                }
                b"model_instance" => {
                    let instance = parse_model_instance(parser)?;
                    plate.items.push(instance);
                }
                _ => {}
            },
            Event::End(end) if end.name().as_ref() == b"plate" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(plate)
}

fn parse_model_instance(parser: &mut XmlParser<Cursor<&[u8]>>) -> Result<PlateModelInstance> {
    let mut instance = PlateModelInstance::default();

    loop {
        match parser.read_next_event()? {
            Event::Empty(e) | Event::Start(e) => {
                if e.name().as_ref() == b"metadata" {
                    let key = get_attribute(&e, b"key");
                    let value = get_attribute(&e, b"value");
                    if let (Some(k), Some(v)) = (key, value) {
                        match k.as_ref() {
                            "object_id" => {
                                instance.object_id = v.parse::<u32>().unwrap_or(0);
                            }
                            "instance_id" => {
                                instance.instance_id = v.parse::<u32>().unwrap_or(0);
                            }
                            "identify_id" => {
                                instance.identify_id = v.parse::<u32>().ok();
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::End(end) if end.name().as_ref() == b"model_instance" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(instance)
}

fn parse_assemble(parser: &mut XmlParser<Cursor<&[u8]>>) -> Result<Vec<AssemblyItem>> {
    let mut items = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Empty(e) | Event::Start(e) => {
                if e.name().as_ref() == b"assemble_item" {
                    let object_id = get_attribute(&e, b"object_id")
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    let instance_count = get_attribute(&e, b"instance_count")
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(1);
                    let transform = get_attribute(&e, b"transform").map(|v| v.into_owned());
                    let offset = get_attribute(&e, b"offset").map(|v| v.into_owned());
                    items.push(AssemblyItem {
                        object_id,
                        instance_count,
                        transform,
                        offset,
                    });
                }
            }
            Event::End(end) if end.name().as_ref() == b"assemble" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(items)
}

// ── parse_project_settings ────────────────────────────────────────────────────

/// Keys containing G-code blobs to skip (avoid large allocations in extras).
const GCODE_BLOB_KEYS: &[&str] = &[
    "change_filament_gcode",
    "machine_end_gcode",
    "machine_start_gcode",
    "time_lapse_gcode",
    "before_layer_change_gcode",
    "layer_change_gcode",
    "printer_start_gcode",
    "printer_end_gcode",
    "toolchange_gcode",
    "gcode_end",
    "gcode_start",
];

/// Parse `Metadata/project_settings.config` (JSON).
///
/// Extracts typed fields for the most important settings. All other fields go
/// into `extras`, except G-code blob keys which are silently skipped.
///
/// Returns `Ok(Default::default())` on empty or invalid JSON.
pub fn parse_project_settings(content: &[u8]) -> Result<BambuProjectSettings> {
    if content.is_empty() {
        return Ok(BambuProjectSettings::default());
    }

    let Ok(text) = std::str::from_utf8(content) else {
        return Ok(BambuProjectSettings::default());
    };

    let Ok(json): std::result::Result<serde_json::Map<String, Value>, _> =
        serde_json::from_str(text)
    else {
        return Ok(BambuProjectSettings::default());
    };

    let mut settings = BambuProjectSettings::default();

    for (key, value) in &json {
        // Skip G-code blobs
        if GCODE_BLOB_KEYS.contains(&key.as_str()) {
            continue;
        }

        match key.as_str() {
            "printer_model" => {
                settings.printer_model = value.as_str().map(|s| s.to_string());
            }
            "inherits" => {
                settings.printer_inherits = value.as_str().map(|s| s.to_string());
            }
            "bed_type" | "curr_bed_type" => {
                // bed_type may not exist; prefer curr_bed_type if present
                if settings.bed_type.is_none() {
                    settings.bed_type = json_to_string_opt(value);
                }
            }
            "layer_height" => {
                settings.layer_height = json_to_f32(value);
            }
            "first_layer_height" => {
                settings.first_layer_height = json_to_f32(value);
            }
            "filament_type" => {
                settings.filament_type = json_to_string_vec(value);
            }
            "filament_colour" => {
                settings.filament_colour = json_to_string_vec(value);
            }
            "nozzle_diameter" => {
                settings.nozzle_diameter = json_to_f32_vec(value);
            }
            "print_sequence" => {
                settings.print_sequence = value.as_str().map(|s| s.to_string());
            }
            "wall_loops" => {
                settings.wall_loops = json_to_u32(value);
            }
            "sparse_infill_density" => {
                settings.infill_density = value.as_str().map(|s| s.to_string());
            }
            "support_type" => {
                settings.support_type = value.as_str().map(|s| s.to_string());
            }
            _ => {
                // Everything else → extras
                settings.extras.insert(key.clone(), value.clone());
            }
        }
    }

    // curr_bed_type overrides bed_type if both present
    if let Some(curr) = json.get("curr_bed_type").and_then(|v| v.as_str()) {
        settings.bed_type = Some(curr.to_string());
    }

    Ok(settings)
}

// ── parse_profile_config ──────────────────────────────────────────────────────

/// Parse a per-profile config JSON file (`filament_settings_N.config`, etc.).
///
/// Extracts `inherits` and `name` fields; everything else (minus gcode blobs)
/// goes into `extras`.
///
/// - `config_type`: "filament", "machine", or "process"
/// - `index`: the N in the filename
pub fn parse_profile_config(
    content: &[u8],
    config_type: &str,
    index: u32,
) -> Result<BambuProfileConfig> {
    if content.is_empty() {
        return Ok(BambuProfileConfig {
            config_type: config_type.to_string(),
            index,
            ..Default::default()
        });
    }

    let Ok(text) = std::str::from_utf8(content) else {
        return Ok(BambuProfileConfig {
            config_type: config_type.to_string(),
            index,
            ..Default::default()
        });
    };

    let Ok(json): std::result::Result<serde_json::Map<String, Value>, _> =
        serde_json::from_str(text)
    else {
        return Ok(BambuProfileConfig {
            config_type: config_type.to_string(),
            index,
            ..Default::default()
        });
    };

    let mut profile = BambuProfileConfig {
        config_type: config_type.to_string(),
        index,
        ..Default::default()
    };

    for (key, value) in &json {
        if GCODE_BLOB_KEYS.contains(&key.as_str()) {
            continue;
        }
        match key.as_str() {
            "inherits" => {
                profile.inherits = value.as_str().map(|s| s.to_string());
            }
            "name" => {
                profile.name = value.as_str().map(|s| s.to_string());
            }
            _ => {
                profile.extras.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(profile)
}

// ── JSON helpers ──────────────────────────────────────────────────────────────

fn json_to_string_opt(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Array(arr) => arr.first().and_then(|x| x.as_str()).map(|s| s.to_string()),
        _ => None,
    }
}

fn json_to_f32(v: &Value) -> Option<f32> {
    match v {
        Value::Number(n) => n.as_f64().map(|x| x as f32),
        Value::String(s) => s.trim().parse::<f32>().ok(),
        _ => None,
    }
}

fn json_to_u32(v: &Value) -> Option<u32> {
    match v {
        Value::Number(n) => n.as_u64().map(|x| x as u32),
        Value::String(s) => s.trim().parse::<u32>().ok(),
        _ => None,
    }
}

fn json_to_string_vec(v: &Value) -> Vec<String> {
    match v {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect(),
        Value::String(s) => vec![s.clone()],
        _ => vec![],
    }
}

fn json_to_f32_vec(v: &Value) -> Vec<f32> {
    match v {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|x| match x {
                Value::Number(n) => n.as_f64().map(|f| f as f32),
                Value::String(s) => s.trim().parse::<f32>().ok(),
                _ => None,
            })
            .collect(),
        Value::Number(n) => n.as_f64().map(|f| vec![f as f32]).unwrap_or_default(),
        Value::String(s) => s.trim().parse::<f32>().map(|f| vec![f]).unwrap_or_default(),
        _ => vec![],
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_slice_info ──

    #[test]
    fn test_parse_slice_info_empty() {
        let result = parse_slice_info(b"").unwrap();
        assert!(result.client_type.is_none());
        assert!(result.plates.is_empty());
    }

    #[test]
    fn test_parse_slice_info_invalid_xml() {
        // Should not panic - just returns an error or partial result
        let _ = parse_slice_info(b"not xml <<< garbage >>>");
    }

    #[test]
    fn test_parse_slice_info_header_only() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<config>
  <header>
    <header_item key="X-BBL-Client-Type" value="slicer"/>
    <header_item key="X-BBL-Client-Version" value="02.02.01.60"/>
  </header>
</config>"#;
        let result = parse_slice_info(xml).unwrap();
        assert_eq!(result.client_type.as_deref(), Some("slicer"));
        assert_eq!(result.client_version.as_deref(), Some("02.02.01.60"));
        assert!(result.plates.is_empty(), "SimplePyramid has no plates");
    }

    #[test]
    fn test_parse_slice_info_with_plate() {
        let xml = br##"<?xml version="1.0" encoding="UTF-8"?>
<config>
  <header>
    <header_item key="X-BBL-Client-Type" value="slicer"/>
    <header_item key="X-BBL-Client-Version" value="01.10.02.73"/>
  </header>
  <plate>
    <metadata key="index" value="1"/>
    <metadata key="prediction" value="1895"/>
    <metadata key="weight" value="11.57"/>
    <filament id="1" tray_info_idx="GFA00" type="PLA" color="#FFFFFF" used_m="3.82" used_g="11.57" />
    <warning msg="bed_temp_too_high" level="1" error_code="1000C001" />
    <object identify_id="145" name="3DBenchy.stl" skipped="false" />
  </plate>
</config>"##;
        let result = parse_slice_info(xml).unwrap();
        assert_eq!(result.client_type.as_deref(), Some("slicer"));
        assert_eq!(result.plates.len(), 1);

        let plate = &result.plates[0];
        assert_eq!(plate.id, 1);
        assert_eq!(plate.prediction, Some(1895));
        assert!((plate.weight.unwrap() - 11.57_f32).abs() < 0.01);

        assert_eq!(plate.filaments.len(), 1);
        assert_eq!(plate.filaments[0].id, 1);
        assert_eq!(plate.filaments[0].tray_info_idx.as_deref(), Some("GFA00"));
        assert_eq!(plate.filaments[0].type_.as_deref(), Some("PLA"));
        assert!((plate.filaments[0].used_g.unwrap() - 11.57_f32).abs() < 0.01);

        assert_eq!(plate.warnings.len(), 1);
        assert_eq!(plate.warnings[0].msg, "bed_temp_too_high");

        assert_eq!(plate.objects.len(), 1);
        assert_eq!(plate.objects[0].id, 145);
        assert_eq!(plate.objects[0].name.as_deref(), Some("3DBenchy.stl"));
    }

    // ── parse_model_settings ──

    #[test]
    fn test_parse_model_settings_empty() {
        let result = parse_model_settings(b"").unwrap();
        assert!(result.plates.is_empty());
        assert!(result.objects.is_empty());
        assert!(result.assembly.is_empty());
    }

    #[test]
    fn test_parse_model_settings_invalid_xml() {
        let _ = parse_model_settings(b"<bad xml");
    }

    #[test]
    fn test_parse_model_settings_with_object() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<config>
  <object id="8">
    <metadata key="name" value="3DBenchy.stl"/>
    <metadata key="extruder" value="1"/>
    <metadata face_count="225154"/>
    <part id="1" subtype="normal_part">
      <metadata key="name" value="3DBenchy.stl"/>
      <metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>
      <metadata key="source_volume_id" value="0"/>
      <metadata key="inner_wall_speed" value="50"/>
      <mesh_stat face_count="225154" edges_fixed="0" degenerate_facets="0" facets_removed="0" facets_reversed="0" backwards_edges="0"/>
    </part>
  </object>
  <plate>
    <metadata key="plater_id" value="1"/>
    <metadata key="plater_name" value=""/>
    <metadata key="locked" value="false"/>
    <metadata key="gcode_file" value="Metadata/plate_1.gcode"/>
    <metadata key="thumbnail_file" value="Metadata/plate_1.png"/>
    <model_instance>
      <metadata key="object_id" value="8"/>
      <metadata key="instance_id" value="0"/>
      <metadata key="identify_id" value="145"/>
    </model_instance>
  </plate>
  <assemble>
    <assemble_item object_id="8" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0" />
  </assemble>
</config>"#;
        let result = parse_model_settings(xml).unwrap();

        // Objects
        assert_eq!(result.objects.len(), 1);
        let obj = &result.objects[0];
        assert_eq!(obj.id, 8);
        assert_eq!(obj.name.as_deref(), Some("3DBenchy.stl"));
        assert_eq!(obj.extruder, Some(1));
        assert_eq!(obj.face_count, Some(225154));
        assert_eq!(obj.parts.len(), 1);

        let part = &obj.parts[0];
        assert_eq!(part.id, 1);
        assert_eq!(part.subtype, PartSubtype::NormalPart);
        assert_eq!(part.name.as_deref(), Some("3DBenchy.stl"));
        assert!(part.matrix.is_some());
        assert!(part.source.is_some());
        assert!(part.mesh_stat.is_some());
        // inner_wall_speed goes to print_overrides
        assert!(part.print_overrides.contains_key("inner_wall_speed"));

        // Plates
        assert_eq!(result.plates.len(), 1);
        let plate = &result.plates[0];
        assert_eq!(plate.id, 1);
        assert!(!plate.locked);
        assert_eq!(plate.gcode_file.as_deref(), Some("Metadata/plate_1.gcode"));
        assert_eq!(
            plate.thumbnail_file.as_deref(),
            Some("Metadata/plate_1.png")
        );
        assert_eq!(plate.items.len(), 1);
        assert_eq!(plate.items[0].object_id, 8);
        assert_eq!(plate.items[0].identify_id, Some(145));

        // Assembly
        assert_eq!(result.assembly.len(), 1);
        assert_eq!(result.assembly[0].object_id, 8);
        assert!(result.assembly[0].transform.is_some());
    }

    // ── parse_project_settings ──

    #[test]
    fn test_parse_project_settings_empty() {
        let result = parse_project_settings(b"").unwrap();
        assert!(result.printer_model.is_none());
        assert!(result.filament_type.is_empty());
    }

    #[test]
    fn test_parse_project_settings_invalid_json() {
        let result = parse_project_settings(b"not json {{{").unwrap();
        assert!(result.printer_model.is_none());
    }

    #[test]
    fn test_parse_project_settings_empty_object() {
        let result = parse_project_settings(b"{}").unwrap();
        assert!(result.printer_model.is_none());
        assert!(result.extras.is_empty());
    }

    #[test]
    fn test_parse_project_settings_basic() {
        let json = serde_json::json!({
            "printer_model": "Bambu Lab A1",
            "curr_bed_type": "Textured PEI Plate",
            "layer_height": 0.2,
            "filament_type": ["PLA"],
            "filament_colour": ["#FFFFFF"],
            "nozzle_diameter": ["0.4"],
            "wall_loops": "3",
            "sparse_infill_density": "15%",
            "support_type": "normal(auto)",
            "change_filament_gcode": "this is a big gcode blob that should be skipped",
            "some_extra_key": "some_value"
        })
        .to_string();
        let result = parse_project_settings(json.as_bytes()).unwrap();

        assert_eq!(result.printer_model.as_deref(), Some("Bambu Lab A1"));
        assert_eq!(result.bed_type.as_deref(), Some("Textured PEI Plate"));
        assert!((result.layer_height.unwrap() - 0.2_f32).abs() < 0.001);
        assert_eq!(result.filament_type, vec!["PLA"]);
        assert_eq!(result.filament_colour, vec!["#FFFFFF"]);
        assert_eq!(result.nozzle_diameter, vec![0.4_f32]);
        assert_eq!(result.wall_loops, Some(3));
        assert_eq!(result.infill_density.as_deref(), Some("15%"));
        assert_eq!(result.support_type.as_deref(), Some("normal(auto)"));
        // G-code key is skipped
        assert!(!result.extras.contains_key("change_filament_gcode"));
        // Unknown keys go to extras
        assert!(result.extras.contains_key("some_extra_key"));
    }

    // ── parse_profile_config ──

    #[test]
    fn test_parse_profile_config_empty() {
        let result = parse_profile_config(b"", "filament", 1).unwrap();
        assert_eq!(result.config_type, "filament");
        assert_eq!(result.index, 1);
        assert!(result.inherits.is_none());
        assert!(result.name.is_none());
    }

    #[test]
    fn test_parse_profile_config_invalid_json() {
        let result = parse_profile_config(b"not json", "machine", 2).unwrap();
        assert_eq!(result.config_type, "machine");
        assert_eq!(result.index, 2);
    }

    #[test]
    fn test_parse_profile_config_basic() {
        let json = serde_json::json!({
            "inherits": "Bambu PLA Basic @BBL P1P",
            "name": "Bambu PLA Basic @BBL P1P(project.3mf)",
            "filament_type": ["PLA"],
            "change_filament_gcode": "gcode blob to skip"
        })
        .to_string();
        let result = parse_profile_config(json.as_bytes(), "filament", 1).unwrap();

        assert_eq!(result.config_type, "filament");
        assert_eq!(result.index, 1);
        assert_eq!(result.inherits.as_deref(), Some("Bambu PLA Basic @BBL P1P"));
        assert_eq!(
            result.name.as_deref(),
            Some("Bambu PLA Basic @BBL P1P(project.3mf)")
        );
        // G-code blob skipped
        assert!(!result.extras.contains_key("change_filament_gcode"));
        // Other keys in extras
        assert!(result.extras.contains_key("filament_type"));
    }
}
