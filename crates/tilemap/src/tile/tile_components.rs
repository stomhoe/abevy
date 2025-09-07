use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::entity_disabling::Disabled;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::*, common_states::*};
use dimension_shared::DimensionRef;
use game_common::game_common_components::{Description, EntityZero, MyZ, YSortOrigin};

use std::hash::{DefaultHasher, Hash, Hasher};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ::tilemap_shared::*;

use crate::{terrain_gen::{terrgen_components::Terrgen, terrgen_events::StudiedOp}, tile::tile_materials::* };

#[derive(Bundle)]
struct ToDenyOnTileClone(
    DisplayName, MinDistancesMap, KeepDistanceFrom, Replicated, TileHidsHandles, 
    TileShaderRef, MyZ, YSortOrigin, ChunkOrTilemapChild, ChildOf, Description, 
    ToDenyOnReleaseBuild,
/*
     
*/ 
);

#[derive(Bundle)]
struct ToDenyOnReleaseBuild( Name, EntityPrefix, TileStrId  );

#[derive(Bundle, Debug, Default)]
pub struct ToAddToTile{
    pub initial_pos: InitialPos,
    pub global_pos: GlobalTilePos,
    pub tile_pos: TilePos,
    pub oplist_size: OplistSize,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
//NO PONER REQUIRE ENTITYPREFIX ACA PORQ SE LO FUERZA A LOS CLONES
#[require(AssetScoped, )]
pub struct Tile;
impl Tile {
    pub const MIN_ID_LENGTH: u8 = 3;
    // for non-sprite tiles
    pub const MAX_Z: MyZ = MyZ(1_000);

    pub fn spawn_from_ref(
        cmd: &mut Commands, tile_ref: EntityZero, global_pos: GlobalTilePos, oplist_size: OplistSize,
    ) -> Entity {
        cmd.entity(tile_ref.0).clone_and_spawn_with(|builder|{
            builder.deny::<ToDenyOnTileClone>();
            //builder.deny::<BundleToDenyOnReleaseBuild>();
        })
        .try_insert((tile_ref, InitialPos(global_pos), global_pos, global_pos.to_tilepos(oplist_size), oplist_size))
        .id()
    }

    pub fn clonespawn_many(
        cmd: &mut Commands, mut tile_refs: Vec<(Entity, (GlobalTilePos, OplistSize))>, 
    ) -> Vec<Entity> {
        let mut new_entities = Vec::with_capacity(tile_refs.len());
        for (entity, _) in tile_refs.iter_mut() {
            *entity = cmd.entity(*entity).clone_and_spawn_with(|builder|{
                builder.deny::<ToDenyOnTileClone>();
                //builder.deny::<BundleToDenyOnReleaseBuild>();
            }).id();
            new_entities.push(*entity);
        }
        cmd.insert_batch(tile_refs);
        new_entities
    }
    
      
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(EntityPrefix::new("Tile Instances"), Name, Transform )]
pub struct TileInstancesHolder;

/*
           .replicate::<TileChildOf>()
           .register_type::<TileChildOf>()
           .register_type::<ChildrenTiles>()
*/
// #[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
// #[relationship(relationship_target = ChildrenTiles)]
// pub struct TileChildOf(#[relationship]#[entities]pub Entity);

// #[derive(Component, Debug, Reflect)]
// #[relationship_target(relationship = TileChildOf)]
// pub struct ChildrenTiles(Vec<Entity>);
// impl ChildrenTiles { pub fn entities(&self) -> &[Entity] { &self.0 } }

pub type TileStrId = StrId20B;

//TODO HACER Q LAS TILES CAMBIEN AUTOMATICAMENTE DE TINTE SEGUN VALOR DE NOISES RELEVANTES COMO HUMEDAD O LO Q SEA
//SE PUEDE MODIFICAR EL SHADER PARA Q TOME OTRO VEC3 DE COLOR MÁS COMO PARÁMETRO Y SE LE MULTIPLIQUE AL PIXEL DE LA TEXTURA SAMPLEADO

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
pub struct ChunkOrTilemapChild;


#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
pub struct PortalTemplate { #[entities]pub dest_dimension: Entity,#[entities]pub root_oplist: Entity, #[entities] pub oe_portal_tile: Entity, 
    #[entities] pub checked_oplist: Entity, pub op_i: i8, pub lim_below: f32, pub lim_above: f32 }
impl PortalTemplate {
    pub fn to_studied_op(&self, start_pos: GlobalTilePos) -> StudiedOp {
        StudiedOp {
            root_oplist: self.root_oplist,
            checked_oplist: self.checked_oplist,
            op_i: self.op_i,
            lim_below: self.lim_below,
            lim_above: self.lim_above,
            search_start_pos: start_pos,
        }
    }
}

impl Default for PortalTemplate {
    fn default() -> Self {
        Self { dest_dimension: Entity::PLACEHOLDER, root_oplist: Entity::PLACEHOLDER, oe_portal_tile: Entity::PLACEHOLDER, checked_oplist: Entity::PLACEHOLDER, op_i: -1, lim_below: 0.0, lim_above: 0.0 }
    }
}



#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
pub struct PortalInstance { #[entities]pub dest_dimension: Entity, pub dest_pos: GlobalTilePos }
impl PortalInstance {
    pub fn new(dest_dimension: Entity, dest_pos: GlobalTilePos) -> Self {
        Self { dest_dimension, dest_pos }
    }
}

pub fn tile_pos_hash_rand(initial_pos: InitialPos, settings: &AaGlobalGenSettings) -> f32 {
    let mut hasher = DefaultHasher::new();
    initial_pos.hash(&mut hasher);
    settings.seed.hash(&mut hasher);
    (hasher.finish() as f64 / u64::MAX as f64).abs() as f32
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct FlipAlongX;

#[derive(Component, Debug,  Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct TileShaderRef(pub Entity);
impl Default for TileShaderRef { fn default() -> Self { Self(Entity::PLACEHOLDER) } }

#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect, )]
#[require(EntityPrefix::new("TileShader"), AssetScoped)]
pub enum TileShader{
    TexRepeat(MonoRepeatTextureOverlayMat),
    TwoTexRepeat(TwoOverlaysExample),
    Voronoi(VoronoiTextureOverlayMat),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}


#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, Debug)]
pub struct InitialPos(pub GlobalTilePos);




#[derive(Component, Debug, Clone, Default)]
pub struct TileHidsHandles { ids: Vec<HashId>, handles: Vec<Handle<Image>>,}

impl TileHidsHandles {
    pub fn from_paths(asset_server: &AssetServer, img_paths: HashMap<String, String>, ) -> Result<Self, BevyError> {

        if img_paths.is_empty() {
            return Err(BevyError::from("TileImgsMap cannot be created with an empty image paths map"));
        }
        let mut ids = Vec::with_capacity(img_paths.len());
        let mut handles = Vec::with_capacity(img_paths.len());
        for (key, path) in img_paths {
            let image_holder = ImageHolder::new(asset_server, &path)?;
            ids.push(HashId::from(key));
            handles.push(image_holder.0);
        }
        Ok(Self { ids, handles, })
    }

    pub fn first_handle(&self) -> Handle<Image> {
        self.handles.first().cloned().unwrap_or_else(|| Handle::default())
    }

    // NO HACER take() porque lo necesitan multiples isntancias de tiles
    pub fn handles(&self) -> &Vec<Handle<Image>> { &self.handles }

    pub fn iter(&self) -> impl Iterator<Item = (HashId, &Handle<Image>)> {
        self.ids.iter().cloned().zip(self.handles.iter())
    }
}



#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Reflect, Default)]
pub struct MinDistancesMap(pub EntityHashMap<u32>);

impl MinDistancesMap {
    #[allow(unused_parens, )]
    pub fn check_min_distances(&self, 
        my_pos: (DimensionRef, GlobalTilePos), new: (EntityZero, DimensionRef, GlobalTilePos)
    ) -> bool {
        self.0.get(&new.0.0).map_or(true, |&min_dist| {
            my_pos.0 != new.1 || my_pos.1.distance_squared(&new.2) > min_dist * min_dist
        })
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct KeepDistanceFrom(#[entities] pub Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Terrgen, EntityPrefix::new("TileSamplers"), )]
pub struct TileSamplerHolder;



