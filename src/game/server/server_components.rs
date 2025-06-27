use bevy::prelude::*;
//use bevy_renet::renet::ClientId;


// #[derive(Component, Debug)]
// pub enum MpAuthority {
//     Player(Entity),
//     Any,
//     Whitelist(Vec<Entity>),
// }

use superstate::{SuperstateInfo};

#[derive(Component, Default)]
#[require(SuperstateInfo<MpAuthority>)]
struct MpAuthority;

#[derive(Component)]
#[require(MpAuthority)]
enum Allowed {
    Exclusive,//NO VA (ENTITY PORQ ESO SIGNIFICARÍA Q PUEDE VARIAR ENTRE BEINGS PERO ES SIEMPRE EL MISMO LOCAL PLAYER)
    Shared{players: Vec<Entity>}
}



#[derive(Component)]
#[require(MpAuthority)]
enum Disallowed {
    Exclusive{player: Entity},
    Shared{players: Vec<Entity>}
}

//todo hacer wrapper para Entity?
//PlayerEntity

//TODO EN VEZ DE ESTO MIRAR 
// https://github.com/projectharmonia/bevy_replicon