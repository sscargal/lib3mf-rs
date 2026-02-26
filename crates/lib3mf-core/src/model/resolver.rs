use crate::archive::ArchiveReader;
use crate::error::{Lib3mfError, Result};
use crate::model::{Geometry, Mesh, Model, Object, ObjectType, ResourceId, Unit};
use crate::parser::model_parser::parse_model;
use std::collections::HashMap;
use std::io::Cursor;

const ROOT_PATH: &str = "ROOT";
const MAIN_MODEL_PART: &str = "3D/3dmodel.model";

/// Resolves resources across multiple model parts in a 3MF package.
pub struct PartResolver<'a, A: ArchiveReader> {
    archive: &'a mut A,
    models: HashMap<String, Model>,
}

impl<'a, A: ArchiveReader> PartResolver<'a, A> {
    pub fn new(archive: &'a mut A, root_model: Model) -> Self {
        let mut models = HashMap::new();
        models.insert(ROOT_PATH.to_string(), root_model);
        Self { archive, models }
    }

    pub fn resolve_object(
        &mut self,
        id: ResourceId,
        path: Option<&str>,
    ) -> Result<Option<(&Model, &Object)>> {
        let model = self.get_or_load_model(path)?;
        Ok(model.resources.get_object(id).map(|obj| (model, obj)))
    }

    pub fn resolve_base_materials(
        &mut self,
        id: ResourceId,
        path: Option<&str>,
    ) -> Result<Option<&crate::model::BaseMaterialsGroup>> {
        let model = self.get_or_load_model(path)?;
        Ok(model.resources.get_base_materials(id))
    }

    pub fn resolve_color_group(
        &mut self,
        id: ResourceId,
        path: Option<&str>,
    ) -> Result<Option<&crate::model::ColorGroup>> {
        let model = self.get_or_load_model(path)?;
        Ok(model.resources.get_color_group(id))
    }

    fn get_or_load_model(&mut self, path: Option<&str>) -> Result<&Model> {
        let part_path = match path {
            Some(p) => {
                let p = p.trim_start_matches('/');
                if p.is_empty() || p.eq_ignore_ascii_case(MAIN_MODEL_PART) {
                    ROOT_PATH
                } else {
                    p
                }
            }
            None => ROOT_PATH,
        };

        if !self.models.contains_key(part_path) {
            let data = self.archive.read_entry(part_path).or_else(|_| {
                let alt = format!("/{}", part_path);
                self.archive.read_entry(&alt)
            })?;

            let model = parse_model(Cursor::new(data))?;
            self.models.insert(part_path.to_string(), model);
        }

        Ok(self.models.get(part_path).unwrap())
    }

    pub fn get_root_model(&self) -> &Model {
        self.models.get("ROOT").unwrap()
    }

    pub fn archive_mut(&mut self) -> &mut A {
        self.archive
    }

    /// Resolves all printable meshes from the build, flattening component hierarchies.
    ///
    /// Walks build items → component trees → sub-model files (via Production Extension
    /// `p:path` references), accumulates transforms, and returns a flat `Vec<ResolvedMesh>`.
    ///
    /// # Filtering
    ///
    /// - `options.filter_non_printable` (default `true`): skip `BuildItem.printable == Some(false)`
    /// - `options.filter_other_objects` (default `true`): skip leaf objects with `ObjectType::Other`
    ///
    /// # Transform accumulation
    ///
    /// Each `ResolvedMesh.transform` is the accumulated product of transforms along the
    /// build item → component chain: `build_item.transform * comp1.transform * comp2.transform ...`
    /// Transforms are NOT pre-applied to vertex positions; the consumer applies them in their
    /// own coordinate space and precision.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A component references an object that does not exist in the target model
    /// - A missing sub-model file is referenced
    /// - A component cycle is detected (same `(object_id, path)` in the current ancestry)
    /// - The component nesting depth exceeds `options.max_depth`
    pub fn resolve_meshes(&mut self, options: &ResolveOptions) -> Result<Vec<ResolvedMesh>> {
        // Clone build items to release the immutable borrow on self before calling
        // methods that require &mut self (same pattern as stats_impl.rs:15-23).
        let build_items = self.get_root_model().build.items.clone();

        let mut out = Vec::new();
        let mut ancestry: Vec<(u32, String)> = Vec::new();

        for item in &build_items {
            if options.filter_non_printable && item.printable == Some(false) {
                continue;
            }

            resolve_recursive(
                item.object_id,
                item.path.as_deref(),
                item.transform,
                0,
                &mut ancestry,
                options,
                self,
                &mut out,
            )?;
        }

        Ok(out)
    }
}

/// A single resolved mesh instance with its accumulated world transform.
///
/// Produced by [`PartResolver::resolve_meshes`]. Each entry corresponds to one leaf
/// mesh object in the component hierarchy.
///
/// # Mesh ownership
///
/// The mesh is owned (not a reference). The borrow checker prevents returning `&'a Mesh`
/// because `get_or_load_model()` requires `&mut self` on `PartResolver`, which conflicts
/// with holding shared references into previously loaded model data. Cloning is the
/// standard pattern used throughout this codebase (see `stats_impl.rs` lines 237-260).
/// A future refactor to `Rc<Model>` inside `PartResolver` would enable zero-copy returns.
#[derive(Debug, Clone)]
pub struct ResolvedMesh {
    /// The actual mesh geometry (owned clone — see struct-level doc for rationale).
    pub mesh: Mesh,
    /// Accumulated transform from the build item and component chain.
    /// This is the product of all transforms from root to this leaf:
    /// `build_item.transform * comp1.transform * ... * compN.transform`.
    /// Transforms are NOT pre-applied to vertex positions.
    pub transform: glam::Mat4,
    /// Object type from the source object (Model, Support, SolidSupport, Surface, Other).
    pub object_type: ObjectType,
    /// Human-readable name of the source object, if set.
    pub name: Option<String>,
    /// Unit of measurement from the source model.
    /// Not converted — the consumer uses [`Unit::convert`] to reach their target unit.
    pub unit: Unit,
}

/// Options controlling the behavior of [`PartResolver::resolve_meshes`].
#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// Skip build items where `printable == Some(false)`. Default: `true`.
    ///
    /// When `true`, only items that are either unspecified or explicitly printable are included.
    pub filter_non_printable: bool,
    /// Skip leaf objects whose `object_type` is [`ObjectType::Other`]. Default: `true`.
    ///
    /// BambuStudio/OrcaSlicer 3MF files include modifier volumes as `type="other"` objects.
    /// Enabling this filter (the default) omits modifier volumes from results.
    pub filter_other_objects: bool,
    /// Maximum component nesting depth before returning an error. Default: `16`.
    ///
    /// Protects against malformed files with excessively deep or infinite component trees.
    pub max_depth: u32,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            filter_non_printable: true,
            filter_other_objects: true,
            max_depth: 16,
        }
    }
}

/// Normalizes a component path to a canonical string for cycle detection and path inheritance.
///
/// Mirrors the normalization logic in `PartResolver::get_or_load_model()` (lines 52–62).
/// - `None`, `"ROOT"`, `"3D/3dmodel.model"`, and `"/3D/3dmodel.model"` all map to `"ROOT"`.
/// - All other paths have leading `/` stripped.
fn canonical_path(path: Option<&str>) -> String {
    match path {
        None | Some(ROOT_PATH) => ROOT_PATH.to_string(),
        Some(p) => {
            let p = p.trim_start_matches('/');
            if p.is_empty() || p.eq_ignore_ascii_case(MAIN_MODEL_PART) {
                ROOT_PATH.to_string()
            } else {
                p.to_string()
            }
        }
    }
}

/// Recursively walks the component tree and collects [`ResolvedMesh`] entries.
///
/// This is the internal workhorse for [`PartResolver::resolve_meshes`]. It mirrors
/// `Model::accumulate_object_stats()` from `stats_impl.rs` but collects resolved meshes
/// instead of statistics.
///
/// # Parameters
///
/// - `id`: The object ID to resolve in the model at `path`.
/// - `path`: The archive path of the model containing `id` (`None` means root model).
/// - `transform`: Accumulated parent transform to multiply with component transforms.
/// - `depth`: Current recursion depth (checked against `options.max_depth`).
/// - `ancestry`: DFS stack of `(object_id, canonical_path)` pairs in the current tree path.
///   Used for cycle detection (not a global visited set — instancing is legal).
/// - `options`: Filtering and safety options.
/// - `resolver`: The part resolver providing model access and sub-model loading.
/// - `out`: Output collection for resolved meshes.
#[allow(clippy::too_many_arguments)]
fn resolve_recursive(
    id: ResourceId,
    path: Option<&str>,
    transform: glam::Mat4,
    depth: u32,
    ancestry: &mut Vec<(u32, String)>,
    options: &ResolveOptions,
    resolver: &mut PartResolver<impl ArchiveReader>,
    out: &mut Vec<ResolvedMesh>,
) -> Result<()> {
    // Depth guard — protects against deeply nested or infinite component trees.
    if depth > options.max_depth {
        return Err(Lib3mfError::InvalidStructure(format!(
            "Component tree depth {} exceeds maximum of {}",
            depth, options.max_depth
        )));
    }

    // Cycle detection — uses DFS ancestry stack so instancing (same object in
    // different subtrees) is correctly allowed (per RESEARCH.md Pitfall 1).
    let canonical = canonical_path(path);
    let key = (id.0, canonical.clone());
    if ancestry.contains(&key) {
        return Err(Lib3mfError::InvalidStructure(format!(
            "Cycle detected: object {} in path {:?} appears in current ancestry",
            id.0, path
        )));
    }
    ancestry.push(key.clone());

    // Resolve the object and clone data to escape the borrow on `resolver`.
    // The borrow checker prevents holding `&Object` (immutable) from `resolve_object`
    // while also calling `get_or_load_model` (&mut self) for child components.
    // Cloning geometry/type/name/unit follows the same pattern as stats_impl.rs:237-260.
    let (geom, inherited_path, obj_type, obj_name, obj_unit) = {
        let resolved = resolver.resolve_object(id, path)?;
        match resolved {
            None => {
                return Err(Lib3mfError::InvalidStructure(format!(
                    "Object {} not found in path {:?}",
                    id.0, path
                )));
            }
            Some((model, object)) => {
                let geom = object.geometry.clone();
                let obj_type = object.object_type;
                let obj_name = object.name.clone();
                let obj_unit = model.unit;
                // Compute the inherited path for children.
                // Root-context objects (None, ROOT, or main model path) do NOT propagate
                // their path — children that need a sub-model path must specify it explicitly
                // via their own `component.path`. Sub-file objects DO propagate their path.
                // This mirrors stats_impl.rs:243-251.
                let inherited = if canonical == ROOT_PATH {
                    None
                } else {
                    Some(canonical.clone())
                };
                (geom, inherited, obj_type, obj_name, obj_unit)
            }
        }
    };

    match geom {
        Geometry::Mesh(mesh) => {
            // ObjectType filtering happens at the leaf (Mesh) level, not at the Component level.
            // A Components object may contain a mix of model and other sub-objects.
            if !options.filter_other_objects || obj_type != ObjectType::Other {
                out.push(ResolvedMesh {
                    mesh,
                    transform,
                    object_type: obj_type,
                    name: obj_name,
                    unit: obj_unit,
                });
            }
        }
        Geometry::Components(comps) => {
            for comp in comps.components {
                // Path priority: component's own path (1) > inherited from parent (2) > None (root).
                // This matches stats_impl.rs:299.
                let next_path = comp.path.as_deref().or(inherited_path.as_deref());

                // Transform accumulation: parent * child (parent applied first).
                // This matches stats_impl.rs:304.
                resolve_recursive(
                    comp.object_id,
                    next_path,
                    transform * comp.transform,
                    depth + 1,
                    ancestry,
                    options,
                    resolver,
                    out,
                )?;
            }
        }
        // SliceStack, VolumetricStack, BooleanShape, DisplacementMesh:
        // These are not triangle meshes — skip silently.
        _ => {}
    }

    // Backtrack: remove from ancestry so the same object can appear in other subtrees
    // (instancing). The ancestry stack represents only the CURRENT path from root.
    ancestry.pop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BuildItem, Component, Components, Geometry, Mesh, Model, Object, ObjectType, ResourceId,
        Unit,
    };
    use std::collections::HashMap;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    // ---------------------------------------------------------------------------
    // MockArchive: in-memory ArchiveReader for testing without real ZIP files
    // ---------------------------------------------------------------------------

    struct MockArchive {
        entries: HashMap<String, Vec<u8>>,
        cursor: Cursor<Vec<u8>>,
    }

    impl MockArchive {
        fn new() -> Self {
            Self {
                entries: HashMap::new(),
                cursor: Cursor::new(Vec::new()),
            }
        }

        fn add_entry(&mut self, path: &str, data: Vec<u8>) {
            self.entries.insert(path.to_string(), data);
        }
    }

    impl Read for MockArchive {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.cursor.read(buf)
        }
    }

    impl Seek for MockArchive {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.cursor.seek(pos)
        }
    }

    impl ArchiveReader for MockArchive {
        fn read_entry(&mut self, name: &str) -> Result<Vec<u8>> {
            self.entries.get(name).cloned().ok_or_else(|| {
                Lib3mfError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Entry not found: {}", name),
                ))
            })
        }

        fn entry_exists(&mut self, name: &str) -> bool {
            self.entries.contains_key(name)
        }

        fn list_entries(&mut self) -> Result<Vec<String>> {
            Ok(self.entries.keys().cloned().collect())
        }
    }

    // ---------------------------------------------------------------------------
    // Helpers: build simple models and objects for tests
    // ---------------------------------------------------------------------------

    /// Serialize a Model to XML bytes (for use as a sub-model entry in MockArchive).
    fn model_to_xml_bytes(model: &Model) -> Vec<u8> {
        let mut buf = Vec::new();
        model.write_xml(&mut buf, None).expect("write_xml failed");
        buf
    }

    /// Create a simple triangle mesh with 3 vertices and 1 triangle.
    fn simple_mesh() -> Mesh {
        let mut mesh = Mesh::new();
        mesh.add_vertex(0.0, 0.0, 0.0);
        mesh.add_vertex(1.0, 0.0, 0.0);
        mesh.add_vertex(0.0, 1.0, 0.0);
        mesh.add_triangle(0, 1, 2);
        mesh
    }

    /// Create an Object with a mesh geometry.
    fn mesh_object(id: u32, object_type: ObjectType, name: Option<&str>) -> Object {
        Object {
            id: ResourceId(id),
            object_type,
            name: name.map(|s| s.to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(simple_mesh()),
        }
    }

    /// Create an Object with a components geometry.
    fn components_object(id: u32, components: Vec<Component>) -> Object {
        Object {
            id: ResourceId(id),
            object_type: ObjectType::Model,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Components(Components { components }),
        }
    }

    /// Create a Component reference to an object (no path, identity transform).
    fn component(object_id: u32) -> Component {
        Component {
            object_id: ResourceId(object_id),
            path: None,
            uuid: None,
            transform: glam::Mat4::IDENTITY,
        }
    }

    /// Create a Component with a specific transform.
    fn component_with_transform(object_id: u32, transform: glam::Mat4) -> Component {
        Component {
            object_id: ResourceId(object_id),
            path: None,
            uuid: None,
            transform,
        }
    }

    /// Create a Component with an external path.
    fn component_with_path(object_id: u32, path: &str, transform: glam::Mat4) -> Component {
        Component {
            object_id: ResourceId(object_id),
            path: Some(path.to_string()),
            uuid: None,
            transform,
        }
    }

    /// Create a BuildItem referencing an object.
    fn build_item(object_id: u32) -> BuildItem {
        BuildItem {
            object_id: ResourceId(object_id),
            uuid: None,
            path: None,
            part_number: None,
            transform: glam::Mat4::IDENTITY,
            printable: None,
        }
    }

    /// Create a BuildItem with a specific transform.
    fn build_item_with_transform(object_id: u32, transform: glam::Mat4) -> BuildItem {
        BuildItem {
            object_id: ResourceId(object_id),
            uuid: None,
            path: None,
            part_number: None,
            transform,
            printable: None,
        }
    }

    /// Create a BuildItem with a printable flag.
    fn build_item_printable(object_id: u32, printable: Option<bool>) -> BuildItem {
        BuildItem {
            object_id: ResourceId(object_id),
            uuid: None,
            path: None,
            part_number: None,
            transform: glam::Mat4::IDENTITY,
            printable,
        }
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn test_resolve_same_file_components() {
        // Object id=1: Mesh with 3 vertices, 1 triangle
        // Object id=2: Components referencing id=1 at identity transform
        // Build item referencing id=2
        let mut model = Model::default();
        let obj1 = mesh_object(1, ObjectType::Model, None);
        let obj2 = components_object(2, vec![component(1)]);
        model.resources.add_object(obj1).unwrap();
        model.resources.add_object(obj2).unwrap();
        model.build.items.push(build_item(2));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].mesh.vertices.len(), 3);
        assert_eq!(meshes[0].mesh.triangles.len(), 1);
        assert_eq!(meshes[0].transform, glam::Mat4::IDENTITY);
    }

    #[test]
    fn test_resolve_transform_accumulation() {
        // Object id=1: Mesh
        // Object id=2: Components referencing id=1 with translation (0,0,10)
        // Build item referencing id=2 with translation (5,0,0)
        let comp_transform = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 10.0));
        let build_transform = glam::Mat4::from_translation(glam::Vec3::new(5.0, 0.0, 0.0));

        let mut model = Model::default();
        let obj1 = mesh_object(1, ObjectType::Model, None);
        let obj2 = components_object(2, vec![component_with_transform(1, comp_transform)]);
        model.resources.add_object(obj1).unwrap();
        model.resources.add_object(obj2).unwrap();
        model
            .build
            .items
            .push(build_item_with_transform(2, build_transform));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();

        assert_eq!(meshes.len(), 1);
        let expected_transform = build_transform * comp_transform;
        assert_eq!(meshes[0].transform, expected_transform);
    }

    #[test]
    fn test_resolve_filters_other_objects() {
        // Object id=1: Mesh, type=Model
        // Object id=2: Mesh, type=Other
        // Object id=3: Components referencing both id=1 and id=2
        // Build item referencing id=3
        let mut model = Model::default();
        let obj1 = mesh_object(1, ObjectType::Model, None);
        let obj2 = mesh_object(2, ObjectType::Other, None);
        let obj3 = components_object(3, vec![component(1), component(2)]);
        model.resources.add_object(obj1).unwrap();
        model.resources.add_object(obj2).unwrap();
        model.resources.add_object(obj3).unwrap();
        model.build.items.push(build_item(3));

        let mut archive = MockArchive::new();

        // Default options: filter_other_objects = true → 1 mesh
        let mut resolver = PartResolver::new(&mut archive, model.clone());
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();
        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].object_type, ObjectType::Model);

        // filter_other_objects = false → 2 meshes
        let mut resolver = PartResolver::new(&mut archive, model);
        let opts = ResolveOptions {
            filter_other_objects: false,
            ..Default::default()
        };
        let meshes = resolver.resolve_meshes(&opts).unwrap();
        assert_eq!(meshes.len(), 2);
    }

    #[test]
    fn test_resolve_filters_non_printable() {
        // Object id=1: Mesh
        // Build item A: printable=Some(true)
        // Build item B: printable=Some(false)
        let mut model = Model::default();
        let obj1 = mesh_object(1, ObjectType::Model, None);
        model.resources.add_object(obj1).unwrap();
        model.build.items.push(build_item_printable(1, Some(true)));
        model.build.items.push(build_item_printable(1, Some(false)));

        let mut archive = MockArchive::new();

        // Default options: filter_non_printable = true → 1 mesh
        let mut resolver = PartResolver::new(&mut archive, model.clone());
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();
        assert_eq!(meshes.len(), 1);

        // filter_non_printable = false → 2 meshes (instancing)
        let mut resolver = PartResolver::new(&mut archive, model);
        let opts = ResolveOptions {
            filter_non_printable: false,
            ..Default::default()
        };
        let meshes = resolver.resolve_meshes(&opts).unwrap();
        assert_eq!(meshes.len(), 2);
    }

    #[test]
    fn test_resolve_cycle_detection() {
        // Object id=1: Components referencing id=2
        // Object id=2: Components referencing id=1 (cycle!)
        // Build item referencing id=1
        let mut model = Model::default();
        let obj1 = components_object(1, vec![component(2)]);
        let obj2 = components_object(2, vec![component(1)]);
        model.resources.add_object(obj1).unwrap();
        model.resources.add_object(obj2).unwrap();
        model.build.items.push(build_item(1));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let result = resolver.resolve_meshes(&ResolveOptions::default());

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Cycle"), "Expected 'Cycle' in error: {}", msg);
    }

    #[test]
    fn test_resolve_depth_limit() {
        // Create a chain: id=1 → id=2 → ... → id=18 (mesh)
        // With max_depth=16 it should error (depth exceeds limit at id=17 or deeper).
        // With max_depth=20 it should succeed.
        let mut model = Model::default();

        // Build chain: object 1 references 2, 2 references 3, ... 17 references 18
        for i in 1u32..18 {
            let obj = components_object(i, vec![component(i + 1)]);
            model.resources.add_object(obj).unwrap();
        }
        // Object 18: leaf mesh
        let leaf = mesh_object(18, ObjectType::Model, None);
        model.resources.add_object(leaf).unwrap();
        model.build.items.push(build_item(1));

        let mut archive = MockArchive::new();

        // With default max_depth=16, depth of 17 components should error
        let mut resolver = PartResolver::new(&mut archive, model.clone());
        let result = resolver.resolve_meshes(&ResolveOptions::default());
        assert!(
            result.is_err(),
            "Expected depth limit error with 17-level chain and max_depth=16"
        );

        // With max_depth=20, the 17-level chain should succeed
        let mut resolver = PartResolver::new(&mut archive, model);
        let opts = ResolveOptions {
            max_depth: 20,
            ..Default::default()
        };
        let meshes = resolver.resolve_meshes(&opts).unwrap();
        assert_eq!(meshes.len(), 1);
    }

    #[test]
    fn test_resolve_dangling_reference() {
        // Object id=1: Components referencing id=999 (doesn't exist)
        // Build item referencing id=1
        let mut model = Model::default();
        let obj1 = components_object(1, vec![component(999)]);
        model.resources.add_object(obj1).unwrap();
        model.build.items.push(build_item(1));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let result = resolver.resolve_meshes(&ResolveOptions::default());

        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("not found"),
            "Expected 'not found' in error: {}",
            msg
        );
    }

    #[test]
    fn test_resolve_instancing_not_false_cycle() {
        // Object id=1: Mesh (used twice via instancing)
        // Object id=2: Components with TWO refs both pointing to id=1
        // Build item referencing id=2
        // Instancing is legal — both references should produce ResolvedMesh entries.
        let mut model = Model::default();
        let obj1 = mesh_object(1, ObjectType::Model, None);
        let obj2 = components_object(2, vec![component(1), component(1)]);
        model.resources.add_object(obj1).unwrap();
        model.resources.add_object(obj2).unwrap();
        model.build.items.push(build_item(2));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();

        assert_eq!(
            meshes.len(),
            2,
            "Instancing (same object referenced twice) should produce 2 ResolvedMesh entries"
        );
    }

    #[test]
    fn test_resolve_unit_carried() {
        // Model with unit=Inch containing a mesh.
        // Assert: ResolvedMesh.unit == Unit::Inch.
        let mut model = Model::default();
        model.unit = Unit::Inch;
        let obj1 = mesh_object(1, ObjectType::Model, None);
        model.resources.add_object(obj1).unwrap();
        model.build.items.push(build_item(1));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].unit, Unit::Inch);
    }

    #[test]
    fn test_resolve_empty_build() {
        // Model with no build items → should return empty Vec.
        let model = Model::default();
        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();
        assert!(meshes.is_empty());
    }

    #[test]
    fn test_resolve_object_name_carried() {
        // Create a named object: name = "MyObject".
        // Assert: ResolvedMesh.name == Some("MyObject").
        let mut model = Model::default();
        let obj1 = mesh_object(1, ObjectType::Model, Some("MyObject"));
        model.resources.add_object(obj1).unwrap();
        model.build.items.push(build_item(1));

        let mut archive = MockArchive::new();
        let mut resolver = PartResolver::new(&mut archive, model);
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].name, Some("MyObject".to_string()));
    }

    #[test]
    fn test_resolve_cross_file_components() {
        // Root model: object id=8, type=model, Components referencing id=1 in sub-model
        // Sub-model at "3D/Objects/object_1.model":
        //   - id=1: Mesh, type=model
        //   - id=2: Mesh, type=other
        // Build item referencing id=8

        // Build the sub-model
        let mut sub_model = Model::default();
        let sub_obj1 = mesh_object(1, ObjectType::Model, None);
        let sub_obj2 = mesh_object(2, ObjectType::Other, None);
        sub_model.resources.add_object(sub_obj1).unwrap();
        sub_model.resources.add_object(sub_obj2).unwrap();
        let sub_xml = model_to_xml_bytes(&sub_model);

        // Build the root model
        let sub_path = "3D/Objects/object_1.model";
        let mut root_model = Model::default();
        let comp1 = component_with_path(1, sub_path, glam::Mat4::IDENTITY);
        let comp2 = component_with_path(2, sub_path, glam::Mat4::IDENTITY);
        let root_obj = components_object(8, vec![comp1, comp2]);
        root_model.resources.add_object(root_obj).unwrap();
        root_model.build.items.push(build_item(8));

        // Set up MockArchive with the sub-model
        let mut archive = MockArchive::new();
        archive.add_entry(sub_path, sub_xml);

        let mut resolver = PartResolver::new(&mut archive, root_model);

        // Default options: filter_other_objects=true → only id=1 (type=model) returned
        let meshes = resolver.resolve_meshes(&ResolveOptions::default()).unwrap();
        assert_eq!(
            meshes.len(),
            1,
            "Expected 1 mesh (type=other filtered out), got {}",
            meshes.len()
        );

        // With filter_other_objects=false → both id=1 and id=2 returned
        let mut archive2 = MockArchive::new();
        let sub_xml2 = model_to_xml_bytes(&sub_model);
        archive2.add_entry(sub_path, sub_xml2);
        let mut resolver2 = PartResolver::new(&mut archive2, {
            let sub_path2 = "3D/Objects/object_1.model";
            let mut root_model2 = Model::default();
            let comp1b = component_with_path(1, sub_path2, glam::Mat4::IDENTITY);
            let comp2b = component_with_path(2, sub_path2, glam::Mat4::IDENTITY);
            let root_obj2 = components_object(8, vec![comp1b, comp2b]);
            root_model2.resources.add_object(root_obj2).unwrap();
            root_model2.build.items.push(build_item(8));
            root_model2
        });
        let opts = ResolveOptions {
            filter_other_objects: false,
            ..Default::default()
        };
        let meshes2 = resolver2.resolve_meshes(&opts).unwrap();
        assert_eq!(
            meshes2.len(),
            2,
            "Expected 2 meshes when filter_other_objects=false"
        );
    }
}
