use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//use bevy_renet::renet::ClientId;


#[derive(Component, Clone, Copy, Deref)]
pub struct MpAuthority(pub Entity);


//TODO EN VEZ DE ESTO MIRAR 
// https://github.com/projectharmonia/bevy_replicon