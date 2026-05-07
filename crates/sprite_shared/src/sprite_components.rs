use ::tilemap_shared::directions::*;
use bevy::ecs::entity::MapEntities;
use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use ::sprite_animation_shared::*;
use crate::sprite_scale_offset::Offset2D;

pub use tilemap_shared::AcZ;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct MovementBased;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct GroundingBased;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct AddUpAnimAndScAcZ;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
#[relationship(relationship_target = HeldSprites)]
#[require(Prefix::trunc("Sprite"))]
pub struct BaseHolderRef {
    #[relationship]
    #[entities]
    pub base: Entity,
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = BaseHolderRef)]
pub struct HeldSprites(Vec<Entity>);

impl std::ops::Deref for HeldSprites {
    type Target = [Entity];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IntoIterator for HeldSprites {
    type Item = Entity;
    type IntoIter = std::vec::IntoIter<Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a> IntoIterator for &'a HeldSprites {
    type Item = &'a Entity;
    type IntoIter = std::slice::Iter<'a, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Component, Debug, Clone)]
pub struct SampleSpritesFromStrIds(Vec<StrId>);
impl SampleSpritesFromStrIds {
    pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Self {
        Self(
            ids.into_iter()
                .filter_map(|s| {
                    let trimmed = s.as_ref().trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(StrId::trunc(trimmed))
                    }
                })
                .collect(),
        )
    }
    pub fn ids(&self) -> &Vec<StrId> {
        &self.0
    }
}

#[derive(Component, std::fmt::Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, MapEntities)]
pub struct ScRef(#[entities] pub Entity);
impl ScRef {
    pub fn is_placeholder(&self) -> bool {
        self.0 == Entity::PLACEHOLDER
    }
}

#[derive(Component, Debug, Default, Serialize, Deserialize, Clone)]
#[require(SelectedForHotReload, AssetScoped, Prefix::trunc("SpriteCfg"))]
pub struct SpriteConfig;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct BaseMovementSpeed(pub f32);

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct SpriteAnimSfx {
    pub sound_paths: Vec<String>,
    pub every_n_frame_changes: f32,
}
impl Default for SpriteAnimSfx {
    fn default() -> Self {
        Self {
            sound_paths: Vec::new(),
            every_n_frame_changes: 1.0,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct SpriteLoopSfx {
    pub sound_paths: Vec<String>,
    pub condition: SfxPlayCondition,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SfxPlayCondition {
    #[default]
    WhileAnimationPlaying,
    WhileMoveActive,
}
impl From<&str> for SfxPlayCondition {
    fn from(value: &str) -> Self {
        match value.trim() {
            "WhileMoveActive" | "while_move_active" | "move_active" => Self::WhileMoveActive,
            _ => Self::WhileAnimationPlaying,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct SpriteTimedSfx {
    pub sound_paths: Vec<String>,
    pub condition: SfxPlayCondition,
    pub time_interval_secs: f32,
    pub scale_interval_with_animation_speed: bool,
}

#[derive(Component, Default, Deserialize, Serialize, Debug, Clone)]
pub struct MappedAnimations(pub HashMap<AnimType, HashId>);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct AnimType {
    pub direction: CardinalDirection,
    pub moving: MoveAnimActive,
    //sprinting:bool? o hacer moveanimactive que tenga un sprinting state?
    pub grounding: Grounding,
    pub state_id: Option<AnimExtraState>,
}
impl AnimType {
    pub fn from_tuple(tuple: (String, String, String, String)) -> Self {
        let (direction, moving, grounding, state_id) = tuple;
        Self {
            direction: CardinalDirection::from(direction),
            moving: MoveAnimActive::from(moving.as_str()),
            grounding: Grounding::from(grounding),
            state_id: if !state_id.is_empty() {
                Some(AnimExtraState::new(state_id))
            } else {
                None
            },
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct ExcludedFromBaseAnimPickingSystem;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub enum FlipHorizIfDir {Left, Right, Any,}

#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct ColorHolder(pub Color);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct UseFallbackSprite;

#[derive(Component, Debug, Default, Clone)]
pub struct Exclusive;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BecomeChildOfSpriteWithTag(pub Tag);

#[derive(Component, Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScsToBuild(pub HashSet<HashId>);
impl ScsToBuild {
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashSet::with_capacity(capacity))
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetForChildren(pub HashMap<Tag, (Offset2D, AppliesOnSpriteDirection)>);

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub enum AppliesOnSpriteDirection {
    None,
    Up,
    Down,
    UpDown,
    Left,
    Right,
    Sideways,
    Any,
}
impl From<&str> for AppliesOnSpriteDirection {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct YSortOrigin(pub f32);
impl YSortOrigin {
    pub const Y_SORT_DIV: f32 = 1e-6;
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ExcludedFromNormalSizeModifier;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct AutoCorrectOffsetBasedOnParentSizeResults;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct FlippedTransform {
    pub x: bool,
    pub y: bool,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct DontNegativizeOffset;

impl From<String> for AppliesOnSpriteDirection {
    fn from(s: String) -> Self {
        Self::from_str(&s)
    }
}
impl AppliesOnSpriteDirection {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "up" | "north" => Self::Up,
            "down" | "south" => Self::Down,
            "left" | "west" => Self::Left,
            "right" | "east" => Self::Right,
            "up_down" | "updown" | "north_south" | "northsouth" => Self::UpDown,
            "sideways" | "west_east" | "westeast" | "east_west" | "eastwest" | "sideway" => Self::Sideways,
            "no" | "none" | "false" => Self::None,
            _ => Self::Any,
        }
    }
    pub fn applies_on_direction(&self, direction: CardinalDirection) -> bool {
        match self {
            Self::None => false,
            Self::Up => direction == CardinalDirection::North,
            Self::Down => direction == CardinalDirection::South,
            Self::Left => direction == CardinalDirection::West,
            Self::Right => direction == CardinalDirection::East,
            Self::UpDown => direction == CardinalDirection::North || direction == CardinalDirection::South,
            Self::Sideways => direction == CardinalDirection::West || direction == CardinalDirection::East,
            Self::Any => true,
        }
    }
}
