use std::default;

use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug, Default, )]
pub struct ActivatesChunks(pub HashSet<Entity>,);


#[derive(Component, Debug,)]
#[require(Visibility::Hidden)]
//DEJARLO COMO IVec2 ASÍ LOS OBJETOS CON TRANSFORM Q NO SEA EXACTAMENTE EL MISMO PUEDEN INDEXAR EL CHUNK MÁS CERCANO, SINO REQUIERE EXACTITUD
pub struct Chunk(pub IVec2);

