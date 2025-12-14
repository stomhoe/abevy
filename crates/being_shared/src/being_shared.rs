
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, )]
pub struct ControlledLocally;

//CAN BE A BOT RUN IN THE CLIENT'S COMPUTER (P.EJ PATHFINDING)

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, )]
pub struct HumanControlled(pub bool);

#[derive(Component, Debug, Reflect, )]
#[relationship_target(relationship = ControlledBy)]
pub struct Controls(Vec<Entity>);
impl Controls {pub fn being_ents(&self) -> &[Entity] {&self.0}}


#[derive(Component, Debug, Deserialize, Serialize, Reflect, )]
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
