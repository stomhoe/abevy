use std::hash::{DefaultHasher, Hash, Hasher};

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::tilemap::{terrain_gen::terrgen_resources::WorldGenSettings, tile::{
    tile_components::*, tile_constants::*, tile_resources::*, ImageSizeSetState
}};



#[allow(unused_parens)]
pub fn add_tileimgs_to_map(asset_server: Res<AssetServer>, 
                            mut map: ResMut<HandleConfigMap>, 
) {
    map.insert(&asset_server, "white.png", false);
    map.insert(&asset_server, "bush/bush0.png", true);
    map.insert(&asset_server, "bush/bush1.png", true);
} 


pub fn update_img_sizes_on_load(
    mut state: ResMut<NextState<ImageSizeSetState>>,
    mut events: EventReader<AssetEvent<Image>>,
    assets: Res<Assets<Image>>,
    mut confmap: ResMut<HandleConfigMap>,
) {
    for ev in events.read() {
        match ev {
            AssetEvent::Added { id } => {

                if let Some(_) = confmap.get(&Tileimg(Handle::Weak(*id))) {
                    let img = assets.get(*id).unwrap();
    
                    let img_size = UVec2::new(
                        img.texture_descriptor.size.width,
                        img.texture_descriptor.size.height,
                    );
                    confmap.set_size(&Tileimg(Handle::Weak(*id)), img_size.as_u16vec2());
                    if confmap.all_tile_sizes_loaded() {
                        state.set(ImageSizeSetState::Done);
                    }
                }
            },
            _ => {

            }
        }
    }
}
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

#[allow(unused_parens)]
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
