# Design: Model Statistics Reporting

## 1. Overview
The goal is to provide comprehensive information about a 3MF file to both human users (via CLI) and applications (via API). This includes standard 3D geometry statistics, 3MF-specific metadata, and vendor-specific telemetry (e.g., Bambu Studio, PrusaSlicer).

## 2. Information Sources

### 2.1 Standard 3MF Metadata (Core Spec)
Properties available in the root `.model` file:
- **Unit**: Millimeter, Micron, etc.
- **Language**: `xml:lang`
- **Application**: Generator name/version (e.g., "BambuStudio-01.10.02.73").
- **Title**: Model title.
- **Designer**: Author/Creator.
- **Description**: Textual description.
- **Copyright**: Licensing info.
- **License**: Creative Commons, etc.
- **ModificationDate**: ISO 8601 date.

### 2.2 Geometry Statistics (Derived)
Requires parsing the mesh data (Phase 3):
- **Object Count**: Total number of unique mesh objects.
- **Instance Count**: Total number of items in the `<build>` section.
- **Triangle Count**: Total triangles (poly count).
- **Vertex Count**: Total vertices.
- **Manifold Status**: Is the mesh watertight? (Requires validation logic).
- **Bounding Box**: Min/Max X, Y, Z dimensions.
- **Dimensions**: Width, Depth, Height.

### 2.3 Production Extension (Standard)
If `xmlns:p` is present:
- **UUIDs**: Unique IDs for build items and objects.
- **Path**: Source path of the component if split across files.

### 2.4 Vendor Extensions (Telemetry)
Vendors like Bambu Lab and Prusa add rich metadata:

#### Bambu Studio / OrcaSlicer
- **Namespace**: `http://schemas.bambulab.com/package/2021`
- **Metadata**:
    - `BambuStudio:3mfVersion`
    - `ProjectSettings`: Links to `Metadata/project_settings.config`.
    - `SlicingInfo`: Links to `Metadata/slice_info.config`.
- **Config Files**:
    - `Metadata/model_settings.config` (XML): Defines **Plates** and object assignments.
        - Structure: `<plate>` elements containing `<metadata key="plater_name">` and `<model_instance>` with `object_id`.
    - `Metadata/process_settings_*.config`: Slicer settings (layer height, infill).
    - `Metadata/filament_settings_*.config`: Material type (PLA, PETG), color, temp.
    - `Metadata/machine_settings_*.config`: Printer model (A1, X1C).
- **Auxiliaries**:
    - Thumbnails: `.thumbnails/*.png`.
    - Plate Images: `Metadata/plate_*.png`.

## 3. Reporting Design

### 3.1 API Design

```rust
pub struct ModelStats {
    pub unit: Unit,
    pub generator: Option<String>,
    pub metadata: HashMap<String, String>,
    pub geometry: GeometryStats,
    pub production: ProductionStats,
    pub vendor: VendorData,
}

pub struct GeometryStats {
    pub object_count: usize,
    pub triangle_count: u64,
    pub vertex_count: u64,
    pub bounds: BoundingBox,
    pub is_manifold: bool,
}

pub struct PlateInfo {
    pub id: u32,
    pub name: Option<String>,
    pub items: Vec<ResourceId>, // Items on this plate
}

pub struct VendorData {
    pub printer_model: Option<String>,
    pub filaments: Vec<FilamentInfo>,
    pub plates: Vec<PlateInfo>, // Multi-plate support
    pub print_time_estimate: Option<String>,
}
```

### 3.2 CLI Output Mockup

```text
$ lib3mf inspect Benchy.3mf

[General]
Create Date: 2025-03-03
Generator:   BambuStudio-01.10.02.73
Unit:        Millimeter
License:     BY-ND

[Geometry]
Objects:     1
Instances:   1
Triangles:   224,192
Vertices:    112,098
Bounds:      60.0 x 31.0 x 48.0 mm
Manifold:    Yes

[Production]
UUID:        2c7c17d8-22b5-4d84-8835-1976022ea369

[Vendor: Bambu Lab]
Printer:     Bambu Lab A1
Filament:    Bambu PLA Basic (White)
Profile:     14min44s, SpeedBoatRace
Plates:      1 (Plate 1)
Plate Image: Metadata/plate_1.png
```

## 4. Implementation Plan (Integrated)

1.  **Phase 3 (XML Parsing)**: Extract standard metadata and simple geometry counts during parse.
2.  **Phase 5 (Validation)**: Compute manifold status.
3.  **Phase 16 (CLI)**: Implement the `inspect` command to aggregate and format this data.
4.  **New Middleware**: `lib3mf-inspector` (optional) or part of `lib3mf-core` to parse vendor config files if requested.
