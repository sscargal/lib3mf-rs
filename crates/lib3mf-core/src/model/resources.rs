use crate::error::{Lib3mfError, Result};
use crate::model::{BaseMaterialsGroup, ColorGroup, Object, SliceStack};
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
}

impl ResourceCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exists(&self, id: ResourceId) -> bool {
        self.objects.contains_key(&id) || 
        self.base_materials.contains_key(&id) ||
        self.color_groups.contains_key(&id) ||
        self.slice_stacks.contains_key(&id)
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

    pub fn base_material_groups_count(&self) -> usize {
        self.base_materials.len()
    }

    pub fn color_groups_count(&self) -> usize {
        self.color_groups.len()
    }

    pub fn iter_objects(&self) -> impl Iterator<Item = &Object> {
        self.objects.values()
    }
}
