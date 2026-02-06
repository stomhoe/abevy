
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::{Prefix, StrId};
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;

#[derive(Component, Debug, Default, )]
pub struct ControlledLocally;

//CAN BE A BOT RUN IN THE CLIENT'S COMPUTER (P.EJ PATHFINDING)

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, )]
pub struct HumanControlled(pub bool);

#[derive(Component, Debug, Reflect, )]
#[relationship_target(relationship = ControlledBy)]
pub struct Controls(Vec<Entity>);
impl Controls {pub fn being_ents(&self) -> &[Entity] {&self.0}}


#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities, )]
#[relationship(relationship_target = Controls)]
pub struct ControlledBy  {
    #[relationship] #[entities]
    pub client: Entity
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, Eq, Clone, Copy, Hash, PartialEq)]
pub enum Grounding {
    #[default]
    Grounded,
    Swimming,
    Floating,
}

impl From<u8> for Grounding {
    fn from(value: u8) -> Self {
        match value {
            0 => Grounding::Grounded,
            1 => Grounding::Swimming,
            2 => Grounding::Floating,
            _ => Grounding::Grounded, // unreachable, but for completeness
        }
    }
}

impl From<String> for Grounding {
    fn from(s: String) -> Self {
        Grounding::from(s.as_str())
    }
}

impl From<&str> for Grounding {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "swimming" | "swim" | "s" | "1" => Grounding::Swimming,
            "floating" | "float" | "f" | "2" => Grounding::Floating,
            _ => Grounding::Grounded,
        }
    }
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect, )]
#[require(Replicated, Prefix::trunc("BeingInstTemplate"))]
pub struct BeingInstTemplate{
    pub points: u32,
    pub extra_health_multiplier: f32,
}




#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, MapEntities, )]
pub struct BitRef(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, MapEntities, )]
pub struct RaceRef(#[entities] pub Entity);

#[derive(Component, Debug, Clone, Reflect, )]
pub struct RaceStrIdRef(pub StrId);

#[derive(Component, Debug, Clone, Reflect, )]
pub struct BitStrIdRef(pub StrId);

#[derive(Component, Debug, Clone, Reflect, )]
pub struct BodyTreeStrIdRef(pub StrId);


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect, MapEntities)]
pub struct Sentient;
