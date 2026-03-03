use crate::model::Unit;
use crate::utils::hardware::HardwareCapabilities;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive statistics and metadata for a 3MF model.
///
/// Aggregates various statistics about the model's geometry, materials,
/// production metadata, and vendor-specific information. Used by the CLI
/// `stats` command and for model analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
    /// Unit of measurement for the model
    pub unit: Unit,
    /// Software that generated the model (from metadata)
    pub generator: Option<String>,
    /// Custom metadata key-value pairs from the model
    pub metadata: HashMap<String, String>,
    /// Geometric statistics (vertices, triangles, volume, etc.)
    pub geometry: GeometryStats,
    /// Material and property statistics
    pub materials: MaterialsStats,
    /// Production extension metadata statistics
    pub production: ProductionStats,
    /// Displacement extension statistics
    pub displacement: DisplacementStats,
    /// Vendor-specific data (e.g., Bambu Studio project info)
    pub vendor: VendorData,
    /// System hardware capabilities info
    pub system_info: HardwareCapabilities,
    /// Thumbnail statistics
    pub thumbnails: ThumbnailStats,
}

/// Statistics about thumbnails in a 3MF package.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThumbnailStats {
    /// Whether a package-level thumbnail is present.
    pub package_thumbnail_present: bool,
    /// Number of object-level thumbnails in the package.
    pub object_thumbnail_count: usize,
}

/// Statistics about material and property resources in a 3MF model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialsStats {
    /// Number of base material groups.
    pub base_materials_count: usize,
    /// Number of color groups.
    pub color_groups_count: usize,
    /// Number of 2D texture coordinate groups.
    pub texture_2d_groups_count: usize,
    /// Number of composite materials groups.
    pub composite_materials_count: usize,
    /// Number of multi-properties groups.
    pub multi_properties_count: usize,
}

/// Geometric statistics for the model.
///
/// Aggregates counts and measurements of the model's geometry including
/// vertices, triangles, bounding box, surface area, and volume.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeometryStats {
    /// Total number of object resources
    pub object_count: usize,
    /// Number of build items (instances to print)
    pub instance_count: usize,
    /// Total number of triangles across all meshes
    pub triangle_count: u64,
    /// Total number of vertices across all meshes
    pub vertex_count: u64,
    /// Axis-aligned bounding box of the entire model
    pub bounding_box: Option<BoundingBox>,
    /// Total surface area in square model units
    pub surface_area: f64,
    /// Total volume in cubic model units
    pub volume: f64,
    /// Whether all meshes are manifold (watertight)
    pub is_manifold: bool,
    /// Count of objects by type (e.g., {"model": 5, "support": 2})
    #[serde(default)]
    pub type_counts: HashMap<String, usize>,
}

/// An axis-aligned bounding box in 3D space.
///
/// Represents the smallest box (aligned with coordinate axes) that
/// contains all geometry. Useful for understanding model size and
/// for spatial queries.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum corner coordinates [x, y, z]
    pub min: [f32; 3],
    /// Maximum corner coordinates [x, y, z]
    pub max: [f32; 3],
}

impl BoundingBox {
    /// Transforms the bounding box by the given 4x4 matrix, returning a new axis-aligned bounding box.
    pub fn transform(&self, matrix: glam::Mat4) -> Self {
        let corners = [
            glam::Vec3::new(self.min[0], self.min[1], self.min[2]),
            glam::Vec3::new(self.min[0], self.min[1], self.max[2]),
            glam::Vec3::new(self.min[0], self.max[1], self.min[2]),
            glam::Vec3::new(self.min[0], self.max[1], self.max[2]),
            glam::Vec3::new(self.max[0], self.min[1], self.min[2]),
            glam::Vec3::new(self.max[0], self.min[1], self.max[2]),
            glam::Vec3::new(self.max[0], self.max[1], self.min[2]),
            glam::Vec3::new(self.max[0], self.max[1], self.max[2]),
        ];

        let mut transformed_min = glam::Vec3::splat(f32::INFINITY);
        let mut transformed_max = glam::Vec3::splat(f32::NEG_INFINITY);

        for corner in corners {
            let transformed = matrix.transform_point3(corner);
            transformed_min = transformed_min.min(transformed);
            transformed_max = transformed_max.max(transformed);
        }

        Self {
            min: [transformed_min.x, transformed_min.y, transformed_min.z],
            max: [transformed_max.x, transformed_max.y, transformed_max.z],
        }
    }
}

/// Statistics from the Production Extension (UUIDs).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductionStats {
    /// Number of objects and build items that have UUIDs assigned.
    pub uuid_count: usize,
}

/// Statistics from the Displacement Extension.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplacementStats {
    /// Number of displacement meshes in the model.
    pub mesh_count: usize,
    /// Number of displacement texture resources.
    pub texture_count: usize,
    /// Total number of normal vectors across all displacement meshes.
    pub normal_count: u64,
    /// Total number of gradient vectors across all displacement meshes.
    pub gradient_count: u64,
    /// Total number of displaced triangles (triangles with displacement coordinate indices).
    pub displaced_triangle_count: u64,
    /// Total number of triangles in all displacement meshes.
    pub total_triangle_count: u64,
}

/// Vendor-specific data extracted from Bambu Studio 3MF files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorData {
    /// Printer model name from Bambu Studio metadata.
    pub printer_model: Option<String>,
    /// List of filament configurations used in the print.
    pub filaments: Vec<FilamentInfo>,
    /// List of plate/build plate configurations.
    pub plates: Vec<PlateInfo>,
    /// Estimated print time from the slicer.
    pub print_time_estimate: Option<String>,
    /// Version string of the slicer that generated the file.
    pub slicer_version: Option<String>,
    /// Nozzle diameter in millimeters.
    pub nozzle_diameter: Option<f32>,
    /// Warnings generated by the slicer during slicing.
    pub slicer_warnings: Vec<SlicerWarning>,
    /// Per-object metadata from Bambu Studio project files.
    pub object_metadata: Vec<BambuObjectMetadata>,
    /// Global project settings from Bambu Studio.
    pub project_settings: Option<BambuProjectSettings>,
    /// Slicer profile configurations embedded in the file.
    pub profile_configs: Vec<BambuProfileConfig>,
    /// Assembly information from Bambu Studio files.
    pub assembly_info: Vec<AssemblyItem>,
    /// Path to Bambu cover thumbnail (from OPC relationship), e.g., "Metadata/plate_1.png"
    pub bambu_cover_thumbnail: Option<String>,
    /// Path to Bambu embedded gcode (from OPC relationship), e.g., "Metadata/plate_1.gcode"
    pub bambu_gcode: Option<String>,
}

/// Information about a single filament used in a Bambu Studio print.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilamentInfo {
    /// Filament slot index.
    pub id: u32,
    /// Tray info index from the AMS system.
    pub tray_info_idx: Option<String>,
    /// Filament type string (e.g., `"PLA"`, `"PETG"`).
    pub type_: String,
    /// Display color in hex format.
    pub color: Option<String>,
    /// Estimated filament used in meters.
    pub used_m: Option<f32>,
    /// Estimated filament used in grams.
    pub used_g: Option<f32>,
}

/// Information about a single build plate in a Bambu Studio project.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlateInfo {
    /// Plate index (1-based).
    pub id: u32,
    /// Optional display name for this plate.
    pub name: Option<String>,
    /// Whether this plate is locked in the slicer.
    pub locked: bool,
    /// Path to the pre-sliced G-code file for this plate.
    pub gcode_file: Option<String>,
    /// Path to the thumbnail image for this plate.
    pub thumbnail_file: Option<String>,
    /// List of object instances on this plate.
    pub items: Vec<PlateModelInstance>,
}

/// An instance of an object on a Bambu Studio build plate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlateModelInstance {
    /// Object resource ID.
    pub object_id: u32,
    /// Instance index for multi-instance objects.
    pub instance_id: u32,
    /// Optional identify ID from the Bambu project metadata.
    pub identify_id: Option<u32>,
}

/// A warning message generated by the slicer during the slicing process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlicerWarning {
    /// Warning message text.
    pub msg: String,
    /// Warning severity level string.
    pub level: Option<String>,
    /// Machine-readable error code.
    pub error_code: Option<String>,
}

/// Per-object metadata from a Bambu Studio project file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BambuObjectMetadata {
    /// Object resource ID.
    pub id: u32,
    /// Object display name.
    pub name: Option<String>,
    /// Extruder index assigned to this object.
    pub extruder: Option<u32>,
    /// Number of triangular faces in this object.
    pub face_count: Option<u64>,
    /// Sub-part metadata for multi-part objects.
    pub parts: Vec<BambuPartMetadata>,
}

/// Metadata for a single part within a Bambu Studio object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BambuPartMetadata {
    /// Part index.
    pub id: u32,
    /// Part subtype (normal, modifier, support blocker/enforcer, etc.).
    pub subtype: PartSubtype,
    /// Part display name.
    pub name: Option<String>,
    /// 3x4 transform matrix string.
    pub matrix: Option<String>,
    /// Source volume information for the part.
    pub source: Option<BambuPartSource>,
    /// Mesh repair statistics.
    pub mesh_stat: Option<BambuMeshStat>,
    /// Per-part print setting overrides.
    pub print_overrides: HashMap<String, String>,
}

/// Classification of a Bambu Studio object part.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum PartSubtype {
    /// A normal printable part (default).
    #[default]
    NormalPart,
    /// A modifier volume that changes settings in a region.
    ModifierPart,
    /// A support blocker volume.
    SupportBlocker,
    /// A support enforcer volume.
    SupportEnforcer,
    /// An unrecognized subtype string.
    Other(String),
}

impl PartSubtype {
    /// Parse a Bambu part subtype string into a `PartSubtype` variant.
    pub fn parse(s: &str) -> Self {
        match s {
            "normal_part" => Self::NormalPart,
            "modifier_part" => Self::ModifierPart,
            "support_blocker" => Self::SupportBlocker,
            "support_enforcer" => Self::SupportEnforcer,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Source volume information for a Bambu Studio part.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BambuPartSource {
    /// Source volume ID (used to identify the originating volume).
    pub volume_id: Option<u32>,
    /// X offset of the source volume.
    pub offset_x: Option<f64>,
    /// Y offset of the source volume.
    pub offset_y: Option<f64>,
    /// Z offset of the source volume.
    pub offset_z: Option<f64>,
}

/// Mesh repair statistics from Bambu Studio's automatic repair.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BambuMeshStat {
    /// Number of edges fixed during repair.
    pub edges_fixed: Option<u32>,
    /// Number of degenerate faces removed.
    pub degenerate_facets: Option<u32>,
    /// Number of faces removed during repair.
    pub facets_removed: Option<u32>,
    /// Number of faces whose winding was reversed.
    pub facets_reversed: Option<u32>,
    /// Number of backwards edges corrected.
    pub backwards_edges: Option<u32>,
}

/// Global project settings from a Bambu Studio project file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BambuProjectSettings {
    /// Printer model name.
    pub printer_model: Option<String>,
    /// Name of the printer profile this inherits from.
    pub printer_inherits: Option<String>,
    /// Bed/plate type (e.g., `"Engineering Plate"`).
    pub bed_type: Option<String>,
    /// Layer height in millimeters.
    pub layer_height: Option<f32>,
    /// First layer height in millimeters.
    pub first_layer_height: Option<f32>,
    /// Filament type strings per extruder.
    pub filament_type: Vec<String>,
    /// Filament display colors per extruder.
    pub filament_colour: Vec<String>,
    /// Nozzle diameters per extruder.
    pub nozzle_diameter: Vec<f32>,
    /// Print sequence (e.g., `"by_layer"` or `"by_object"`).
    pub print_sequence: Option<String>,
    /// Number of perimeter walls.
    pub wall_loops: Option<u32>,
    /// Infill density percentage string.
    pub infill_density: Option<String>,
    /// Support generation type.
    pub support_type: Option<String>,
    /// Additional key-value settings not covered by named fields.
    pub extras: HashMap<String, serde_json::Value>,
}

/// A slicer profile configuration embedded in a Bambu Studio project file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BambuProfileConfig {
    /// Profile category: `"filament"`, `"machine"`, or `"process"`.
    pub config_type: String, // "filament", "machine", "process"
    /// Index suffix (the N in `filament_settings_N.config`).
    pub index: u32, // the N in filament_settings_N.config
    /// Name of the profile this inherits from.
    pub inherits: Option<String>,
    /// Display name of this profile.
    pub name: Option<String>,
    /// Additional settings not covered by named fields.
    pub extras: HashMap<String, serde_json::Value>,
}

/// An assembly item from a Bambu Studio project file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssemblyItem {
    /// Object resource ID.
    pub object_id: u32,
    /// Number of instances of this object in the assembly.
    pub instance_count: u32,
    /// Transform matrix string.
    pub transform: Option<String>,
    /// Offset string.
    pub offset: Option<String>,
}
