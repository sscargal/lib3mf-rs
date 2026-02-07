use crate::model::ResourceId;
use serde::{Deserialize, Serialize};

/// Represents a color in sRGB space with alpha channel.
///
/// Colors in 3MF use 8-bit RGBA format, with values from 0-255.
/// The alpha channel represents opacity (0 = fully transparent, 255 = fully opaque).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha/opacity component (0-255, where 255 is fully opaque)
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // Parse hex string #RRGGBB or #RRGGBBAA
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        let val = u32::from_str_radix(hex, 16).ok()?;

        match hex.len() {
            6 => Some(Self {
                r: ((val >> 16) & 0xFF) as u8,
                g: ((val >> 8) & 0xFF) as u8,
                b: (val & 0xFF) as u8,
                a: 255,
            }),
            8 => Some(Self {
                r: ((val >> 24) & 0xFF) as u8,
                g: ((val >> 16) & 0xFF) as u8,
                b: ((val >> 8) & 0xFF) as u8,
                a: (val & 0xFF) as u8,
            }),
            _ => None,
        }
    }

    /// Convert color to hex string #RRGGBBAA
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

/// A base material with a name and display color.
///
/// Base materials represent named material types (e.g., "PLA", "ABS", "Steel")
/// with an associated display color for visualization. The actual material
/// properties are typically handled by the printer/slicer software based
/// on the name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMaterial {
    /// Human-readable material name
    pub name: String,
    /// Display color for visualization
    pub display_color: Color,
}

/// A resource group containing multiple base materials.
///
/// Base materials groups are referenced by triangles via property IDs,
/// with the property index selecting which material from the group to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMaterialsGroup {
    /// Unique resource ID for this material group
    pub id: ResourceId,
    /// List of materials in this group
    pub materials: Vec<BaseMaterial>,
}

/// A resource group containing multiple colors for per-vertex/per-triangle coloring.
///
/// Color groups allow assigning different colors to different parts of a mesh.
/// Triangles reference the group via property ID and select specific colors
/// via property indices. Colors use RGBA format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGroup {
    /// Unique resource ID for this color group
    pub id: ResourceId,
    /// List of colors in this group
    pub colors: Vec<Color>,
}

/// A resource group defining texture coordinates for 2D texture mapping.
///
/// Texture groups map UV coordinates to vertices for applying texture images
/// to mesh surfaces. The texture image itself is stored as an attachment in
/// the 3MF package and referenced by `texture_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture2DGroup {
    /// Unique resource ID for this texture coordinate group
    pub id: ResourceId,
    /// Reference to the texture image resource (attachment path)
    pub texture_id: ResourceId,
    /// List of UV coordinates
    pub coords: Vec<Texture2DCoord>,
}

/// A 2D texture coordinate (UV mapping).
///
/// UV coordinates map vertices to positions in a texture image.
/// Typically, u and v range from 0.0 to 1.0, where (0,0) is one corner
/// of the texture and (1,1) is the opposite corner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture2DCoord {
    /// Horizontal texture coordinate (typically 0.0 to 1.0)
    pub u: f32,
    /// Vertical texture coordinate (typically 0.0 to 1.0)
    pub v: f32,
}

/// A 2D texture resource (image file reference).
///
/// Texture2D defines a reference to an image file within the 3MF package
/// that can be applied to mesh surfaces. The actual image data is stored
/// as an attachment and referenced by the path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture2D {
    /// Unique resource ID for this texture
    pub id: ResourceId,
    /// Path to the texture image within the 3MF package (e.g., "/3D/Textures/diffuse.png")
    pub path: String,
    /// MIME content type of the texture (e.g., "image/png", "image/jpeg")
    pub contenttype: String,
}

/// A resource group for composite/mixed materials.
///
/// Composite materials allow blending multiple materials together with
/// specified mixing ratios. This enables gradient materials, multi-material
/// prints, and material transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeMaterials {
    /// Unique resource ID for this composite materials group
    pub id: ResourceId,
    /// Reference to the base materials group to blend from
    pub base_material_id: ResourceId,
    /// Indices specifying which base materials are used in composites
    pub indices: Vec<u32>,
    /// List of composite material definitions
    pub composites: Vec<Composite>,
}

/// A single composite material definition specifying blend ratios.
///
/// The values specify mixing ratios for the materials referenced by
/// the parent `CompositeMaterials`' indices. Values typically sum to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    /// Mixing ratios for each material (typically summing to 1.0)
    pub values: Vec<f32>,
}

/// A resource group for combining multiple property types.
///
/// Multi-properties allow applying multiple different property types
/// (materials, colors, textures) to the same geometry, with specified
/// blending methods to combine them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProperties {
    /// Unique resource ID for this multi-properties group
    pub id: ResourceId,
    /// List of property resource IDs to combine
    pub pids: Vec<ResourceId>,
    /// Blending methods for combining each property
    pub blend_methods: Vec<BlendMethod>,
    /// List of multi-property index combinations
    pub multis: Vec<Multi>,
}

/// A single multi-property combination specifying indices into each property group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multi {
    /// Property indices for each property group (parallel to parent's pids)
    pub pindices: Vec<u32>,
}

/// Method for blending multiple properties together.
///
/// Determines how multiple properties (e.g., base color and texture) are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMethod {
    /// No blending - use first property only
    NoBlend,
    /// Linear interpolation/mixing
    Mix,
    /// Multiplicative blending
    Multiply,
}

/// Texture channel for displacement mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Channel {
    R,
    #[default]
    G,
    B,
    A,
}

/// Texture wrapping/tiling style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TileStyle {
    #[default]
    Wrap,
    Mirror,
    Clamp,
    None,
}

/// Texture filtering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FilterMode {
    #[default]
    Linear,
    Nearest,
}

/// A 2D displacement texture resource for surface detail.
///
/// Displacement textures modify surface geometry based on texture values,
/// allowing fine surface detail without requiring dense meshes. The texture
/// values are interpreted as height offsets along surface normals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Displacement2D {
    /// Unique resource ID for this displacement texture
    pub id: ResourceId,
    /// Path to the texture image in the 3MF package
    pub path: String,
    /// Which color channel to use for displacement values
    #[serde(default)]
    pub channel: Channel,
    /// How the texture wraps/tiles
    #[serde(default)]
    pub tile_style: TileStyle,
    /// Texture filtering mode
    #[serde(default)]
    pub filter: FilterMode,
    /// Maximum displacement height in model units
    pub height: f32,
    /// Base displacement offset in model units
    #[serde(default)]
    pub offset: f32,
}
