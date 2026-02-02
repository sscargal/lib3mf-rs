use crate::error::{Lib3mfError, Result};
use crate::model::{
    BaseMaterialsGroup, ColorGroup, CompositeMaterials, KeyStore, MultiProperties, Object,
    SliceStack, Texture2DGroup, VolumetricStack,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a resource within the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ResourceId(pub u32);

/// Collection of all resources in the model (Objects, Materials, etc.).
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
    pub key_store: Option<KeyStore>, // Usually one KeyStore per model/part
}

impl ResourceCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exists(&self, id: ResourceId) -> bool {
        self.objects.contains_key(&id)
            || self.base_materials.contains_key(&id)
            || self.color_groups.contains_key(&id)
            || self.slice_stacks.contains_key(&id)
            || self.volumetric_stacks.contains_key(&id)
            || self.texture_2d_groups.contains_key(&id)
            || self.composite_materials.contains_key(&id)
            || self.multi_properties.contains_key(&id)
    }

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

    pub fn get_object(&self, id: ResourceId) -> Option<&Object> {
        self.objects.get(&id)
    }

    pub fn get_base_materials(&self, id: ResourceId) -> Option<&BaseMaterialsGroup> {
        self.base_materials.get(&id)
    }

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

    pub fn iter_objects(&self) -> impl Iterator<Item = &Object> {
        self.objects.values()
    }

    pub fn iter_objects_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.objects.values_mut()
    }

    pub fn iter_base_materials(&self) -> impl Iterator<Item = &BaseMaterialsGroup> {
        self.base_materials.values()
    }

    pub fn iter_color_groups(&self) -> impl Iterator<Item = &ColorGroup> {
        self.color_groups.values()
    }

    pub fn iter_textures(&self) -> impl Iterator<Item = &Texture2DGroup> {
        self.texture_2d_groups.values()
    }

    pub fn iter_composite_materials(&self) -> impl Iterator<Item = &CompositeMaterials> {
        self.composite_materials.values()
    }

    pub fn iter_multi_properties(&self) -> impl Iterator<Item = &MultiProperties> {
        self.multi_properties.values()
    }
}
