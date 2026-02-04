use crate::error::{Lib3mfError, Result};
use crate::model::{
    BaseMaterialsGroup, ColorGroup, CompositeMaterials, Displacement2D, KeyStore, MultiProperties,
    Object, SliceStack, Texture2DGroup, VolumetricStack,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a resource within the model.
///
/// A type-safe wrapper around `u32` that prevents accidentally mixing resource IDs
/// with raw integers. All resources in a 3MF model share a global ID namespace,
/// meaning an ID can only be used once across all resource types (objects, materials,
/// textures, etc.).
///
/// # Examples
///
/// ```
/// use lib3mf_core::model::ResourceId;
///
/// let id = ResourceId(42);
/// assert_eq!(id.0, 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ResourceId(pub u32);

/// Central registry for all resources in a 3MF model.
///
/// The `ResourceCollection` manages all reusable resources including objects,
/// materials, textures, and extension-specific resources. It enforces the global
/// ID namespace requirement: each [`ResourceId`] can only be used once across
/// all resource types.
///
/// Resources are stored in separate `HashMap<ResourceId, T>` collections internally,
/// allowing efficient lookup by ID.
///
/// # Examples
///
/// ```
/// use lib3mf_core::model::{ResourceCollection, Object, ResourceId, Geometry, Mesh, ObjectType};
///
/// let mut resources = ResourceCollection::new();
///
/// let obj = Object {
///     id: ResourceId(1),
///     object_type: ObjectType::Model,
///     name: None,
///     part_number: None,
///     uuid: None,
///     pid: None,
///     pindex: None,
///     thumbnail: None,
///     geometry: Geometry::Mesh(Mesh::default()),
/// };
///
/// resources.add_object(obj).expect("Failed to add object");
/// assert!(resources.exists(ResourceId(1)));
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceCollection {
    objects: HashMap<ResourceId, Object>,
    base_materials: HashMap<ResourceId, BaseMaterialsGroup>,
    color_groups: HashMap<ResourceId, ColorGroup>,
    slice_stacks: HashMap<ResourceId, SliceStack>,
    volumetric_stacks: HashMap<ResourceId, VolumetricStack>,
    texture_2d_groups: HashMap<ResourceId, Texture2DGroup>,
    composite_materials: HashMap<ResourceId, CompositeMaterials>,
    multi_properties: HashMap<ResourceId, MultiProperties>,
    displacement_2d: HashMap<ResourceId, Displacement2D>,
    pub key_store: Option<KeyStore>, // Usually one KeyStore per model/part
}

impl ResourceCollection {
    /// Creates a new empty resource collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if a resource with the given ID exists in any resource type.
    ///
    /// Returns `true` if the ID is used by any resource (object, material, texture, etc.).
    pub fn exists(&self, id: ResourceId) -> bool {
        self.objects.contains_key(&id)
            || self.base_materials.contains_key(&id)
            || self.color_groups.contains_key(&id)
            || self.slice_stacks.contains_key(&id)
            || self.volumetric_stacks.contains_key(&id)
            || self.texture_2d_groups.contains_key(&id)
            || self.composite_materials.contains_key(&id)
            || self.multi_properties.contains_key(&id)
            || self.displacement_2d.contains_key(&id)
    }

    /// Adds an object to the collection.
    ///
    /// # Errors
    ///
    /// Returns `Lib3mfError::Validation` if a resource with the same ID already exists.
    pub fn add_object(&mut self, object: Object) -> Result<()> {
        if self.exists(object.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                object.id.0
            )));
        }
        self.objects.insert(object.id, object);
        Ok(())
    }

    /// Adds a base materials group to the collection.
    ///
    /// # Errors
    ///
    /// Returns `Lib3mfError::Validation` if a resource with the same ID already exists.
    pub fn add_base_materials(&mut self, group: BaseMaterialsGroup) -> Result<()> {
        if self.exists(group.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                group.id.0
            )));
        }
        self.base_materials.insert(group.id, group);
        Ok(())
    }

    /// Adds a color group to the collection.
    ///
    /// # Errors
    ///
    /// Returns `Lib3mfError::Validation` if a resource with the same ID already exists.
    pub fn add_color_group(&mut self, group: ColorGroup) -> Result<()> {
        if self.exists(group.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                group.id.0
            )));
        }
        self.color_groups.insert(group.id, group);
        Ok(())
    }

    pub fn add_slice_stack(&mut self, stack: SliceStack) -> Result<()> {
        if self.exists(stack.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                stack.id.0
            )));
        }
        self.slice_stacks.insert(stack.id, stack);
        Ok(())
    }

    pub fn add_volumetric_stack(&mut self, stack: VolumetricStack) -> Result<()> {
        if self.exists(stack.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                stack.id.0
            )));
        }
        self.volumetric_stacks.insert(stack.id, stack);
        Ok(())
    }

    pub fn set_key_store(&mut self, store: KeyStore) {
        self.key_store = Some(store);
    }

    /// Retrieves an object by its ID.
    ///
    /// Returns `None` if no object with the given ID exists.
    pub fn get_object(&self, id: ResourceId) -> Option<&Object> {
        self.objects.get(&id)
    }

    /// Retrieves a base materials group by its ID.
    ///
    /// Returns `None` if no base materials group with the given ID exists.
    pub fn get_base_materials(&self, id: ResourceId) -> Option<&BaseMaterialsGroup> {
        self.base_materials.get(&id)
    }

    /// Retrieves a color group by its ID.
    ///
    /// Returns `None` if no color group with the given ID exists.
    pub fn get_color_group(&self, id: ResourceId) -> Option<&ColorGroup> {
        self.color_groups.get(&id)
    }

    pub fn get_slice_stack(&self, id: ResourceId) -> Option<&SliceStack> {
        self.slice_stacks.get(&id)
    }

    pub fn get_volumetric_stack(&self, id: ResourceId) -> Option<&VolumetricStack> {
        self.volumetric_stacks.get(&id)
    }

    pub fn add_texture_2d_group(&mut self, group: Texture2DGroup) -> Result<()> {
        if self.exists(group.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                group.id.0
            )));
        }
        self.texture_2d_groups.insert(group.id, group);
        Ok(())
    }

    pub fn get_texture_2d_group(&self, id: ResourceId) -> Option<&Texture2DGroup> {
        self.texture_2d_groups.get(&id)
    }

    pub fn add_composite_materials(&mut self, group: CompositeMaterials) -> Result<()> {
        if self.exists(group.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                group.id.0
            )));
        }
        self.composite_materials.insert(group.id, group);
        Ok(())
    }

    pub fn get_composite_materials(&self, id: ResourceId) -> Option<&CompositeMaterials> {
        self.composite_materials.get(&id)
    }

    pub fn add_multi_properties(&mut self, group: MultiProperties) -> Result<()> {
        if self.exists(group.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                group.id.0
            )));
        }
        self.multi_properties.insert(group.id, group);
        Ok(())
    }

    pub fn get_multi_properties(&self, id: ResourceId) -> Option<&MultiProperties> {
        self.multi_properties.get(&id)
    }

    pub fn base_material_groups_count(&self) -> usize {
        self.base_materials.len()
    }

    pub fn color_groups_count(&self) -> usize {
        self.color_groups.len()
    }

    pub fn volumetric_stacks_count(&self) -> usize {
        self.volumetric_stacks.len()
    }

    pub fn texture_2d_groups_count(&self) -> usize {
        self.texture_2d_groups.len()
    }

    pub fn composite_materials_count(&self) -> usize {
        self.composite_materials.len()
    }

    pub fn multi_properties_count(&self) -> usize {
        self.multi_properties.len()
    }

    /// Returns an iterator over all objects in the collection.
    pub fn iter_objects(&self) -> impl Iterator<Item = &Object> {
        self.objects.values()
    }

    /// Returns a mutable iterator over all objects in the collection.
    pub fn iter_objects_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.objects.values_mut()
    }

    /// Returns an iterator over all base material groups in the collection.
    pub fn iter_base_materials(&self) -> impl Iterator<Item = &BaseMaterialsGroup> {
        self.base_materials.values()
    }

    /// Returns an iterator over all color groups in the collection.
    pub fn iter_color_groups(&self) -> impl Iterator<Item = &ColorGroup> {
        self.color_groups.values()
    }

    /// Returns an iterator over all texture 2D groups in the collection.
    pub fn iter_textures(&self) -> impl Iterator<Item = &Texture2DGroup> {
        self.texture_2d_groups.values()
    }

    /// Returns an iterator over all composite material groups in the collection.
    pub fn iter_composite_materials(&self) -> impl Iterator<Item = &CompositeMaterials> {
        self.composite_materials.values()
    }

    /// Returns an iterator over all multi-property groups in the collection.
    pub fn iter_multi_properties(&self) -> impl Iterator<Item = &MultiProperties> {
        self.multi_properties.values()
    }

    pub fn add_displacement_2d(&mut self, res: Displacement2D) -> Result<()> {
        if self.exists(res.id) {
            return Err(Lib3mfError::Validation(format!(
                "Duplicate resource ID: {}",
                res.id.0
            )));
        }
        self.displacement_2d.insert(res.id, res);
        Ok(())
    }

    pub fn get_displacement_2d(&self, id: ResourceId) -> Option<&Displacement2D> {
        self.displacement_2d.get(&id)
    }

    pub fn displacement_2d_count(&self) -> usize {
        self.displacement_2d.len()
    }

    pub fn iter_displacement_2d(&self) -> impl Iterator<Item = &Displacement2D> {
        self.displacement_2d.values()
    }
}
