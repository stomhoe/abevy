use bevy::math::IVec2;
use bevy::prelude::*;
use common::{common_components::Tag, common_tag_components::TagSet};
use serde::{Deserialize, Serialize};

use crate::AcZ;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DeleteOtherTilesSeri {
    #[serde(default)]
    pub spared_z: Vec<f32>,
    #[serde(default)]
    pub targeted_z: Vec<f32>,
    #[serde(default)]
    pub spared_tags: Vec<String>,
    #[serde(default)]
    pub targeted_tags: Vec<String>,
    #[serde(default)]
    pub extra_radius: u32,
    #[serde(default)]
    pub displacement: (i32, i32),
    #[serde(default)]
    pub priority: f32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct DeleteOtherTilesInSamePos {
    pub spared_z: bevy::platform::collections::HashSet<AcZ>,
    pub targeted_z: bevy::platform::collections::HashSet<AcZ>,
    pub spared_tags: TagSet,
    pub targeted_tags: TagSet,
    pub extra_radius: u32,
    pub displacement: IVec2,
    /// use this only if both delete each other and they don't spare each other. the one with higher priority doesn't get deleted
    pub priority: f32,
}

impl DeleteOtherTilesInSamePos {
    pub fn is_empty(&self) -> bool {
        self.spared_z.is_empty()
            && self.targeted_z.is_empty()
            && self.spared_tags.is_empty()
            && self.targeted_tags.is_empty()
    }
    pub fn apply_delete_other_tiles_field(&mut self, field: &str, values: &[String]) -> bool {
        match field {
            "spared_tags" => {
                let mut applied = false;
                for value in values {
                    if value.trim().is_empty() {
                        continue;
                    }
                    self.spared_tags.insert(Tag::trunc(value));
                    applied = true;
                }
                applied
            }
            "targeted_tags" => {
                let mut applied = false;
                for value in values {
                    if value.trim().is_empty() {
                        continue;
                    }
                    self.targeted_tags.insert(Tag::trunc(value));
                    applied = true;
                }
                applied
            }
            "spared_z" => {
                let mut applied = false;
                for value in values {
                    let Ok(value) = value.parse::<f32>() else {
                        continue;
                    };
                    self.spared_z.insert(AcZ(value));
                    applied = true;
                }
                applied
            }
            "targeted_z" => {
                let mut applied = false;
                for value in values {
                    let Ok(value) = value.parse::<f32>() else {
                        continue;
                    };
                    self.targeted_z.insert(AcZ(value));
                    applied = true;
                }
                applied
            }
            "extra_radius" => {
                let Some(value) = values.first() else {
                    return false;
                };
                let Ok(value) = value.parse::<u32>() else {
                    return false;
                };
                self.extra_radius = value;
                true
            }
            "priority" => {
                let Some(value) = values.first() else {
                    return false;
                };
                let Ok(value) = value.parse::<f32>() else {
                    return false;
                };
                self.priority = value;
                true
            }
            "displacement" => {
                let Some(x) = values.first().and_then(|value| value.parse::<i32>().ok()) else {
                    return false;
                };
                let Some(y) = values.get(1).and_then(|value| value.parse::<i32>().ok()) else {
                    return false;
                };
                self.displacement = IVec2::new(x, y);
                true
            }
            _ => false,
        }
    }
    pub fn merge_from(&mut self, other: &Self) {
        self.spared_z.extend(other.spared_z.iter().copied());
        self.targeted_z.extend(other.targeted_z.iter().copied());
        for tag in other.spared_tags.iter() {
            self.spared_tags.insert(tag.clone());
        }
        for tag in other.targeted_tags.iter() {
            self.targeted_tags.insert(tag.clone());
        }
        self.extra_radius = other.extra_radius;
        self.priority = other.priority;
        self.displacement = other.displacement;
    }

    pub fn should_delete_tile_based_on_tag_sets(
        &self,
        target_z: &AcZ,
        target_tags: Option<&TagSet>,
    ) -> bool {
        if !self.targeted_z.is_empty() {
            if !self.targeted_z.contains(target_z) {
                return false;
            }
            if let Some(tags) = target_tags {
                if self.spared_tags.intersects(tags) {
                    return false;
                }
            }
            return true;
        }
        if !self.targeted_tags.is_empty() {
            let Some(tags) = target_tags else {
                return false;
            };
            if !self.targeted_tags.intersects(tags) {
                return false;
            }
            if self.spared_tags.intersects(tags) {
                return false;
            }
            if self.spared_z.contains(target_z) {
                return false;
            }
            return true;
        }
        if self.spared_z.contains(target_z) {
            return false;
        }
        if let Some(tags) = target_tags {
            if self.spared_tags.intersects(tags) {
                return false;
            }
        }
        true
    }
}

impl DeleteOtherTilesSeri {
    pub fn delete_other_tiles_from_seri(&self) -> DeleteOtherTilesInSamePos {
        let mut spared_z = bevy::platform::collections::HashSet::default();
        for &z in &self.spared_z {
            spared_z.insert(AcZ(z));
        }
        let mut targeted_z = bevy::platform::collections::HashSet::default();
        for &z in &self.targeted_z {
            targeted_z.insert(AcZ(z));
        }
        let mut spared_tags = TagSet::default();
        for tag in &self.spared_tags {
            spared_tags.insert(Tag::trunc(tag));
        }
        let mut targeted_tags = TagSet::default();
        for tag in &self.targeted_tags {
            targeted_tags.insert(Tag::trunc(tag));
        }
        DeleteOtherTilesInSamePos {
            spared_z,
            targeted_z,
            spared_tags,
            targeted_tags,
            extra_radius: self.extra_radius,
            displacement: IVec2::new(self.displacement.0, self.displacement.1),
            priority: self.priority,
        }
    }
}
