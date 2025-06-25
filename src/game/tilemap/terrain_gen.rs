#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::game::{tilemap::{terrain_gen::{terrain_gen_components::FnlComp, terrain_gen_resources::*, terrain_gen_systems::*, terrain_gen_utils::*, terrain_materials::TextureOverlayMaterial}, tilemap_resources::*}, GamePhase, IngameSystems};

mod terrain_gen_systems;
pub mod terrain_materials;
pub mod terrain_gen_components;
pub mod terrain_gen_resources;
pub mod terrain_gen_utils;
//mod terrain_generation_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainGenSystems;

pub struct TerrainGenPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TerrainGenPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(TerrainGenerationSystems).in_set(IngameSystems))
            .add_systems(OnEnter(GamePhase::InGame), (setup, ))
            .init_resource::<WorldGenSettings>()
            .init_resource::<Textures>()
            .add_plugins(MaterialTilemapPlugin::<TextureOverlayMaterial>::default())
        ;
    }
}

#[allow(unused_parens)]
pub fn gather_all_tiles2spawn_within_chunk (
    mut commands: &mut Commands, 
    asset_server: &AssetServer, 
    noise_query: Query<&FnlComp>, 
    gen_settings: &WorldGenSettings,
    chunk_pos: IVec2,) -> Vec<TileDto> {
    
    let mut tiles2spawn: Vec<TileDto> = Vec::new();
    
    for x in 0..CHUNK_SIZE.x { 
        for y in 0..CHUNK_SIZE.y {
            let pos_within_chunk = UVec2::new(x, y);
            let tilepos = chunkpos_to_tilepos(chunk_pos) + pos_within_chunk.as_ivec2();
            add_tiles_for_tilepos(&mut tiles2spawn, &mut commands, asset_server, noise_query, tilepos, pos_within_chunk);
    }} tiles2spawn
}

fn add_tiles_for_tilepos(tiles2spawn: &mut Vec<TileDto>, mut commands: &mut Commands, asset_server: &AssetServer, 
    noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: UVec2,
) {
    let asd: Handle<Image> = asset_server.load("textures/world/white.png");

    //si una tile es suitable para una edificación, o spawnear una village o algo, se le puede añadir un componente SuitableForVillage o algo así, para que se pueda identificar la tile. después se puede hacer un sistema q borre los árboles molestos en un cierto radio. el problema es si hay múltiples marcadas adyacentemente, en ese caso va a haber q chequear distancias a otras villages

    let tileinfo = TileDto {
        layer_z: GRASS_Z_LEVEL,
        pos_within_chunk,
        used_handle: asd,
        flip: TileFlip::default(),
        color: TileColor::from(Color::srgb(1., 0., 0.)),
        needs_y_sort: false,
        entity: None,
        visible: TileVisible::default(),
        ..Default::default()
    };
    
    tiles2spawn.push(tileinfo);   
}

const RENDER_CHUNK_SIZE: UVec2 = UVec2 {x: CHUNK_SIZE.x * 2, y: CHUNK_SIZE.y * 2,};
const TILEMAP_GRID_SIZE: TilemapGridSize = TilemapGridSize { x: TILE_SIZE_PXS.x as f32, y: TILE_SIZE_PXS.y as f32 };


const RED: Color = Color::srgb(1., 0., 0.);
const GREEN: Color = Color::srgb(0., 1., 0.);
const BLUE: Color = Color::srgb(0., 0., 1.);


#[allow(unused_parens)]
pub fn fill_tilemap_data(commands: &mut Commands, 
    asset_server: &AssetServer,
    tmap_entity: Entity,
    texture: TilemapTexture, 
    chunk_ent: Entity, 
    _chunk_pos: IVec2,//lo dejo por si lo necesita algún shader
    storage: TileStorage,
    layer_z_level: i32, y_sort: bool, 
    tile_size: TilemapTileSize,
    texture_overley_mat: &mut Assets<TextureOverlayMaterial>,//buscar formas alternativas de agarrar shaders
)
{
    commands.entity(tmap_entity).insert(ChildOf(chunk_ent));
    let mut ent_commands = commands.entity(tmap_entity);
    let grid_size = TILEMAP_GRID_SIZE; let size = CHUNK_SIZE.into();  
    let transform = Transform::from_translation(Vec3::new(0.0, 0.0, (layer_z_level as f32/Z_DIVISOR as f32)));
    let render_settings = TilemapRenderSettings {render_chunk_size: RENDER_CHUNK_SIZE, y_sort,};

    match layer_z_level {
        GRASS_Z_LEVEL => {

            let material = MaterialTilemapHandle::from(texture_overley_mat.add(TextureOverlayMaterial {
                texture_overlay: asset_server.load("textures/world/terrain/temperate_grass/grass.png"),
                scale: 0.001,
                ..Default::default()
            }));

            ent_commands.insert((
                MaterialTilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, 
                    material, ..Default::default()
                },
            ));
        },
        _ => {
            ent_commands.insert(
                TilemapBundle {grid_size, size, storage, texture, tile_size, transform, render_settings, ..Default::default()}
            );
        }
    }
}
