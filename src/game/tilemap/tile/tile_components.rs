use bevy::math::U8Vec4;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::tilemap::{chunking_components::ChunkPos, tile::tile_constants::* };
use bevy_ecs_tilemap::tiles::*;

use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;


#[derive(Bundle, Debug)]
pub struct MyTileBundle {
    pub name: Name,
    pub img_id: Tileimg,
    pub flip: TileFlip,
    pub color: TileColor,
    pub z_index: TileZIndex,
    pub visible: TileVisible,
    pub shader: AppliedShader,
}

impl Default for MyTileBundle {
    fn default() -> Self {
        let rand_string: String = format!("Tile {}", nano_id::base64::<3>());
        Self {
            name: Name::new(rand_string),
            img_id: Tileimg::default(),
            flip: TileFlip::default(),
            color: TileColor::default(),
            z_index: TileZIndex::default(),
            visible: TileVisible::default(),
            shader: AppliedShader::default(),
        }
    }
}

impl MyTileBundle {
    pub fn new(
        name: Name,
        img_id: Tileimg,
        flip: TileFlip,
        color: TileColor,
        z_index: TileZIndex,
        visible: bool,
        shader: AppliedShader,
    ) -> Self {
        Self {
            name,
            img_id,
            flip,
            color,
            z_index,
            visible: TileVisible(visible),
            shader,
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct TileposHashRand(pub f32);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(TileposHashRand)]
pub struct FlipAlongX;


#[derive(Component, Debug, Default, )]
pub struct Tree;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Eq, PartialEq, Hash, Copy)]
pub struct TileZIndex(pub i32);


#[derive(Component, Debug, Default, Hash, PartialEq, Eq, Clone, )]
pub enum AppliedShader{
    #[default]
    None,
    MonoRepeating(RepeatingTexture),
    BiRepeating(RepeatingTexture, RepeatingTexture),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, )]
pub struct RepeatingTexture{
    img: Handle<Image>,
    scale: u32, //scale to be divided by 1M
    mask_color: U8Vec4,
}

impl RepeatingTexture {
    pub fn new<S: Into<String>>(asset_server: &AssetServer, path: S, scale: u32, mask_color: U8Vec4) -> Self {
        Self { img: asset_server.load(path.into()), scale, mask_color }
    }
    pub fn new_w_red_mask<S: Into<String>>(asset_server: &AssetServer, path: S, scale: u32) -> Self {
        Self { img: asset_server.load(path.into()), scale, mask_color: U8Vec4::new(255, 0, 0, 255) }
    }
    pub fn cloned_handle(&self) -> Handle<Image> {
        self.img.clone()
    }

    #[allow(non_snake_case)]
    pub fn scale_div_1M(&self) -> f32 {
        self.scale as f32 / 1_000_000.0
    }
    pub fn mask_color(&self) -> Vec4 {
        self.mask_color.as_vec4()/255.0 
    }
}


#[derive(Component, Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct Tileimg(pub Handle<Image>);
impl Tileimg {
    pub fn new<S: Into<String>>(asset_server: &AssetServer, path: S) -> Self {
        let path = format!("{}{}", TILEIMG_BASE_PATH, path.into());
        Self(asset_server.load(path))
    }

}

#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy)]
pub struct GlobalTilePos(pub IVec2);

impl GlobalTilePos {
    pub fn get_pos_within_chunk(self, chunk_pos: ChunkPos) -> TilePos {
        let pos_within_chunk = self - chunk_pos.to_tilepos();
        TilePos::new(pos_within_chunk.x() as u32, pos_within_chunk.y() as u32)
    }
    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }

    pub const TYPE_DEBUG_NAME: &'static str = "GlobalTilePos";
}

impl std::fmt::Display for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_DEBUG_NAME, self.0.x, self.0.y)
    }
}
impl std::fmt::Debug for GlobalTilePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}, {})", Self::TYPE_DEBUG_NAME, self.0.x, self.0.y)
    }
}

impl From<Vec2> for GlobalTilePos {
    fn from(pixelpos: Vec2) -> Self {
        GlobalTilePos(pixelpos.div_euclid(TILE_SIZE_PXS.as_vec2()).as_ivec2())
    }
}

impl std::ops::Add for GlobalTilePos {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GlobalTilePos(self.0 + other.0)
    }
}

impl std::ops::Sub for GlobalTilePos {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        GlobalTilePos(self.0 - other.0)
    }
}
