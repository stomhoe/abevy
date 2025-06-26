
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TileFlip, TileTextureIndex, TileVisible};
use fastnoise_lite::FastNoiseLite;
use superstate::{SuperstateInfo};

use crate::game::tilemap::{terrain_gen::terrain_gen_utils::UniqueTileDto, tile_imgs::*};

#[derive(Component, Default, )]
pub struct FnlComp(pub FastNoiseLite);

#[derive(Component, Debug, Default, )]
pub struct Thresholds(pub Vec<(f32, Entity)>); //usar menor igual valor -> entidad. Entidad-> tiledist?



#[derive(Component, Default)]
#[require(SuperstateInfo<TileDeterminism>)]
pub struct TileDeterminism;

#[derive(Component, Debug, Default, )]
#[require(TileDeterminism)]
pub struct TileDistribution(
    pub HashMap<Entity, f32>, //Entity: una instancia de Fill
);


#[derive(Component, Debug, Default, )]
#[require(SuperstateInfo<Fill>, TileDeterminism)]
pub struct Fill;//no sé si ponerle id o q se referencie la entity instanciada 



#[derive(Component, Debug, Default, )]
pub struct Tree();


//hacerlo parte de una entity 
#[derive(Component, Debug, )]
pub struct TileInstantiationData{
    pub image_nid: TileImgNid,
    pub flip: TileFlip,
    pub color: Color,
    pub visible: bool,
    pub used_shader: UsedShader, 
    
}
impl TileInstantiationData {
    pub fn new(image_nid: TileImgNid, flip: TileFlip, color: Color, visible: bool, used_shader: UsedShader ) -> Self {
        Self { image_nid, flip, color, visible, used_shader }
    }
    pub fn tile_visible(&self) -> TileVisible {TileVisible(self.visible)}

    pub fn tile_color(&self) -> TileColor {TileColor(self.color)}
}


impl Default for TileInstantiationData {
    fn default() -> Self {
        Self { 
            image_nid: TileImgNid::default(),
            flip: TileFlip::default(),
            color: Color::default(),
            visible: true,
            used_shader: UsedShader::default(),
        }
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum UsedShader{
    #[default]
    None,
    Grass
}