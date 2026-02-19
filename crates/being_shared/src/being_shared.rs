
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;

#[derive(Component, Debug, Default, Clone)]
pub struct ControlledLocally;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ControlledByClient;

//CAN BE A BOT RUN IN THE CLIENT'S COMPUTER (P.EJ PATHFINDING)

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct IsHumanControlled(pub bool);

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = ControlledBy)]
pub struct ControlledBeings(Vec<Entity>);
impl ControlledBeings {pub fn being_ents(&self) -> &[Entity] {&self.0}}


#[derive(Component, Debug, Deserialize, Serialize, MapEntities, Clone)]
#[relationship(relationship_target = ControlledBeings)]
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


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
#[require(Replicated, Prefix::trunc("BeingInstTemplate"), AssetScoped, HotReload)]
pub struct BeingInstTemplate{
    pub points: u32,
    pub extra_health_multiplier: f32,
}







#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct Sentient;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct WallPhaser;
