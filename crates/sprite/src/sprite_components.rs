use being_shared::Grounding;
use bevy::ecs::entity::MapEntities;
use bevy::math::{Vec2, UVec2};
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, Spritesheet};
use common::common_components::*;
use common::common_types::*;
use game_common::game_common_components::{FacingDirection};
use serde::{Deserialize, Serialize};
use sprite_animation_shared::{AnimationState, MoveAnimActive};

use crate::sprite_scale_offset_components::Offset2D;

#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Reflect)]
#[require(Replicated, AssetScoped, EntityPrefix::new_truncated("SpriteConfigs"), )]
pub struct SpriteConfigsHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(EntityPrefix::new_truncated("SpriteConfig"), AssetScoped, Replicated)]
pub struct SpriteConfig;

//todo agregarle path_to_sprite y hacer q si no hay un sprite component en la entity q lo instancie
//de esta forma no hay q pre-spawnear los spriteconfig en los clients y se pueden replicar normalmente


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct MovementBased;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct GroundingBased;


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
#[relationship(relationship_target = HeldSprites)]
#[require(EntityPrefix::new_truncated("Sprite"), Replicated,)]
pub struct BaseHolderRef {#[relationship]#[entities]pub base: Entity, }

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = BaseHolderRef)]
pub struct HeldSprites(Vec<Entity>);
impl HeldSprites {pub fn sprite_ents(&self) -> &Vec<Entity> { &self.0 }}

#[derive(Component, Default, Deserialize, Serialize, Debug, Reflect, MapEntities)]
pub struct SpriteCfgAnimationsMap (
    // anim_type - used anim
    //defaultear a usar FacingDirection::South si no es directionable
    //defaultear a usar Grounding::Grounded si no importa la altitud actual
    //Option<StrId> para guardar animaciones para estados específicos
    #[entities]pub HashMap<AnimType, Entity>
);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect, MapEntities)]
pub struct AnimType {
    pub direction: FacingDirection,
    pub moving: MoveAnimActive,
    pub grounding: Grounding,
    pub state_id: Option<AnimationState>,
}

impl AnimType {
    pub fn from_tuple(tuple: (String, String, String, String)) -> Self {
        let (direction, moving, grounding, state_id) = tuple;
        AnimType {
            direction: FacingDirection::from(direction),
            moving: MoveAnimActive::from(moving.as_str()),
            grounding: Grounding::from(grounding),
            state_id: if !state_id.is_empty() {
                Some(AnimationState::new(&state_id))
            } else {
                None
            },
        }
    }
    
}

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ExcludedFromBaseAnimPickingSystem;


#[derive(Component, Debug, Deserialize, Serialize,  Clone, Copy)]
pub enum FlipHorizIfDir{Left, Right, Any,}





#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect, MapEntities)]
#[require(Transform, Visibility)]
pub struct SpriteConfigRef(#[entities] pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct ColorHolder(pub Color);//NO HACER PARTE DE SpriteDataBundle

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct OffsetForChildren(pub HashMap<Category, (Offset2D, AppliesOnSpriteDirection)>);


#[derive(Component, Debug, Deserialize, Serialize,  Clone, Copy)]
pub enum AppliesOnSpriteDirection{None, Up, Down, UpDown, Left, Right, Sideways, Any,}
impl From<&str> for AppliesOnSpriteDirection {
    fn from(s: &str) -> Self {
        AppliesOnSpriteDirection::from_str(s)
    }
}

impl From<String> for AppliesOnSpriteDirection {
    fn from(s: String) -> Self {
        AppliesOnSpriteDirection::from_str(&s)
    }
}

impl AppliesOnSpriteDirection {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "up" | "north" => AppliesOnSpriteDirection::Up,
            "down" | "south" => AppliesOnSpriteDirection::Down,
            "left" | "west" => AppliesOnSpriteDirection::Left,
            "right" | "east" => AppliesOnSpriteDirection::Right,
            "up_down" | "updown" | "north_south" | "northsouth"  => AppliesOnSpriteDirection::UpDown,
            "sideways" | "west_east" | "westeast" | "east_west" | "eastwest" | "sideway" => AppliesOnSpriteDirection::Sideways,
            "no" | "none" | "false" => AppliesOnSpriteDirection::None,
            _ => AppliesOnSpriteDirection::Any,
        }
    }
    pub fn applies_on_direction(&self, direction: FacingDirection) -> bool {
        match self {
            AppliesOnSpriteDirection::None => false,
            AppliesOnSpriteDirection::Up => direction == FacingDirection::North,
            AppliesOnSpriteDirection::Down => direction == FacingDirection::South,
            AppliesOnSpriteDirection::Left => direction == FacingDirection::West,
            AppliesOnSpriteDirection::Right => direction == FacingDirection::East,
            AppliesOnSpriteDirection::UpDown => direction == FacingDirection::North || direction == FacingDirection::South,
            AppliesOnSpriteDirection::Sideways => direction == FacingDirection::West || direction == FacingDirection::East,
            AppliesOnSpriteDirection::Any => true,
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct Exclusive;

#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct BecomeChildOfSpriteWithCategory (pub Category);

// NO USAR ESTOS DOS PARA BEINGS
#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
/// DON'T REPLICATE
pub struct SpriteConfigStrIds(Vec<StrId>);
impl SpriteConfigStrIds {
    pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Self {
        Self(ids.into_iter().map(|s| StrId::new_truncated(s)).collect())
    }
    pub fn ids(&self) -> &Vec<StrId> { &self.0 }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone )]
pub struct SpriteCfgsToBuild(#[entities] pub HashSet<Entity>);


#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy )]
pub struct BecomeChildOf(#[entities] pub Entity);




