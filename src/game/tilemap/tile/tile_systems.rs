use std::hash::{DefaultHasher, Hash, Hasher};

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::tilemap::{terrain_gen::terrgen_resources::WorldGenSettings, tile::tile_components::*};




#[allow(unused_parens)]
pub fn update_tile_hash_value(
    settings: Res<WorldGenSettings>,
    mut query: Query<(&GlobalTilePos, &mut TileposHashRand),(Added<GlobalTilePos>)>) {
    for (pos, mut hash) in query.iter_mut() {
        let mut hasher = DefaultHasher::new();
        pos.hash(&mut hasher);
        settings.seed.hash(&mut hasher);
        hash.0 = (hasher.finish() as f64 / u64::MAX as f64) as f32;
    }
}

#[allow(unused_parens)]//TODO PONER ESTO EN LOS BEINGS TMB, PERO USANDO SU TRANSFORM
pub fn update_tile_name(mut query: Query<(&mut Name, &GlobalTilePos),(Changed<GlobalTilePos>)>) {
    for (mut name, pos) in query.iter_mut() {
        let prev_name = name.as_str().split(GlobalTilePos::TYPE_DEBUG_NAME).next().unwrap_or("Tile").to_string();
        name.set(format!("{} {:?}", prev_name, pos));
    }
}

#[allow(unused_parens)]
pub fn flip_tile_along_x(mut query: Query<(&mut TileFlip, &TileposHashRand),(With<FlipAlongX>, Changed<TileposHashRand>)>) {
    for (mut flip, &TileposHashRand(val)) in query.iter_mut() {
        if val > 0.5 { flip.x = !flip.x; }
    }
}
