use bevy::prelude::*;
use bevy_renet::renet::ClientId;


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

//para usar cuando es una lista de 2-6 jugadores del lobby
fn maybe_push<T>(vec: &mut Vec<T>, val: T) {
    if vec.len() == vec.capacity() {
        let new_cap = if vec.capacity() == 0 {
            2
        } else {
            vec.capacity() + 2
        };
        vec.reserve_exact(new_cap - vec.capacity());
    }
    vec.push(val);
}
