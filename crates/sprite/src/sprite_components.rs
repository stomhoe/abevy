use being_shared::Grounding;
use bevy::ecs::entity::{EntityHashSet, MapEntities};
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_components::{CardinalDirection};
use serde::{Deserialize, Serialize};
use sprite_animation_shared::{AnimationState, MoveAnimActive};
use sprite_shared::sprite_scale_offset::Offset2D;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(SparedFromHotReloading, AssetScoped, Replicated, Prefix::trunc("SpriteConfig"), )]
pub struct SpriteConfig;


#[derive(Component, Default, Deserialize, Serialize, Debug, Reflect, MapEntities)]
pub struct MappedAnimations (
    #[entities]pub HashMap<AnimType, Entity>
);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect, MapEntities)]
pub struct AnimType {
    pub direction: CardinalDirection,
    pub moving: MoveAnimActive,
    pub grounding: Grounding,
    pub state_id: Option<AnimationState>,
}
impl AnimType {
    pub fn from_tuple(tuple: (String, String, String, String)) -> Self {
        let (direction, moving, grounding, state_id) = tuple;
        AnimType {
            direction: CardinalDirection::from(direction),
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct ColorHolder(pub Color);



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct Exclusive;

#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct BecomeChildOfSpriteWithTag (pub Tag);



#[derive(Component, Debug, Deserialize, Serialize, Clone, MapEntities, Default )]
pub struct ScsToBuild(#[entities] pub EntityHashSet);
impl ScsToBuild {
    pub fn with_capacity(capacity: usize) -> Self {
        ScsToBuild(EntityHashSet::with_capacity(capacity))
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct OffsetForChildren(pub HashMap<Tag, (Offset2D, AppliesOnSpriteDirection)>);


#[derive(Component, Debug, Deserialize, Serialize,  Clone, Copy, Reflect)]
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
    pub fn applies_on_direction(&self, direction: CardinalDirection) -> bool {
        match self {
            AppliesOnSpriteDirection::None => false,
            AppliesOnSpriteDirection::Up => direction == CardinalDirection::North,
            AppliesOnSpriteDirection::Down => direction == CardinalDirection::South,
            AppliesOnSpriteDirection::Left => direction == CardinalDirection::West,
            AppliesOnSpriteDirection::Right => direction == CardinalDirection::East,
            AppliesOnSpriteDirection::UpDown => direction == CardinalDirection::North || direction == CardinalDirection::South,
            AppliesOnSpriteDirection::Sideways => direction == CardinalDirection::West || direction == CardinalDirection::East,
            AppliesOnSpriteDirection::Any => true,
        }
    }
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct SpriteConfigNotFound;
