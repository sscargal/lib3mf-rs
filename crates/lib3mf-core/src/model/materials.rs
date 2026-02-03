use crate::model::ResourceId;
use serde::{Deserialize, Serialize};

/// Represents a color in sRGB space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMaterial {
    pub name: String,
    pub display_color: Color,
}

/// A resource group containing multiple base materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMaterialsGroup {
    pub id: ResourceId,
    pub materials: Vec<BaseMaterial>,
}

/// A resource group containing multiple colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGroup {
    pub id: ResourceId,
    pub colors: Vec<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture2DGroup {
    pub id: ResourceId,
    pub texture_id: ResourceId,
    pub coords: Vec<Texture2DCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture2DCoord {
    pub u: f32,
    pub v: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeMaterials {
    pub id: ResourceId,
    pub base_material_id: ResourceId,
    pub indices: Vec<u32>,
    pub composites: Vec<Composite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiProperties {
    pub id: ResourceId,
    pub pids: Vec<ResourceId>,
    pub blend_methods: Vec<BlendMethod>,
    pub multis: Vec<Multi>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multi {
    pub pindices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMethod {
    NoBlend,
    Mix,
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

/// A 2D displacement texture resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Displacement2D {
    pub id: ResourceId,
    pub path: String,
    #[serde(default)]
    pub channel: Channel,
    #[serde(default)]
    pub tile_style: TileStyle,
    #[serde(default)]
    pub filter: FilterMode,
    pub height: f32,
    #[serde(default)]
    pub offset: f32,
}
