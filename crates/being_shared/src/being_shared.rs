
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



#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect, )]
pub enum BeingAltitude{
    #[default] OnGround, Swimming, Floating,
}
