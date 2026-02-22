use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;


use crate::sprite_components::SpriteConfig;

#[derive(serde::Deserialize, Default)]
pub struct SfxEveryNframesSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default = "default_animation_sfx_every_n_frame_changes")]
    pub n: f32,
}

#[derive(serde::Deserialize, Default)]
pub struct SfxTimeIntervalSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub condition: String,
    #[serde(default = "default_sfx_interval_secs")]
    pub secs: f32,
    #[serde(default)]
    pub shorten_with_anim_playing_speed: bool,
}

#[derive(serde::Deserialize, Default)]
pub struct SfxLoopSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub condition: String,
}

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct SpriteConfigSeri {
    pub id: String,
    pub name: String,
    pub mapped_anims: HashMap<(String, String, String, String), String>,
    #[serde(default)]
    pub parent_cat: String, //adds ChildOf referencing other brother entity sprite possessing this tag
    #[serde(default)]
    pub tags: HashSet<String>,
    #[serde(default)]
    pub shares_tag: Vec<bool>,//asignar un componente
    #[serde(default)]
    pub children_sprites: Vec<String>,// these will get spawned as children of the entity that has this sprite data
    #[serde(default)]
    pub sfx_every_n_frames: SfxEveryNframesSeri,
    #[serde(default)]
    pub loop_sfx: SfxLoopSeri,
    #[serde(default)]
    pub interval_sfx: SfxTimeIntervalSeri,
    #[serde(default)]
    pub directionable: bool,
    #[serde(default)]
    pub movement_based: bool,
    #[serde(default)]
    pub grounding_based: bool,

    //use fly animation when standing still
    pub visibility: Option<u8>, //0: inherited, 1: visible, 2: invisible
    #[serde(default)]
    pub offset4children: HashMap<String, (f32, f32, String)>,//k:tag, v:(offset, direction(s)) in which it is applied
    #[serde(default)]
    pub exclude_from_sys: bool,

    // Being's speed ratio over this is used as speedup multiplier for anim.
    // Sentinel: <= 0.01 means disabled.
    #[serde(default = "default_base_movement_speed")]
    pub base_movement_speed: f32,

    #[serde(default)]
    pub exclude_from_normal_size_modifier: bool,


    #[serde(default)]
    pub offset: (f32, f32),
    #[serde(default = "default_scale_2d")]
    pub scale: (f32, f32),
    #[serde(default = "default_scale_2d")]
    pub scale_up_down: (f32, f32),
    #[serde(default = "default_scale_2d")]
    pub scale_sideways: (f32, f32),
    pub flip_horiz_if_dir: Option<u8>, //Left, Right, Any
    #[serde(default)]
    pub offset_up_down: (f32, f32),
    #[serde(default)]
    pub offset_down: (f32, f32),
    #[serde(default)]
    pub offset_up: (f32, f32),
    #[serde(default)]
    pub offset_sideways: (f32, f32),

    /// TODO
    pub extra_y_offset_per_scale_inc: Option<f32>,

}
fn default_scale_2d() -> (f32, f32) { (1.0, 1.0) }
fn default_base_movement_speed() -> f32 { 0.0 }
fn default_animation_sfx_every_n_frame_changes() -> f32 { 1.0 }
fn default_sfx_interval_secs() -> f32 { 0.35 }
// PARA LAS BODY PARTS INTANGIBLES LASTIMABLES/CON HP, HACER Q EN LA DEFINICIÓN DE ESTOS SEAN ASOCIABLES A SPRITES CONCRETOS MEDIANTE SU ID O CATEGORY (AL DESTRUIR LA BODY PART SE INVISIBILIZA (NO BORRAR POR SI SE CURA DESP)). NO ASOCIAR BODY PARTS A SPRITE MEDIANTE EL PROPIO SPRITE PORQ AFECTA EL REUSO DE ESTE (P EJ EL CUERPO DE UN HUMANO PUEDE SER USADO EN OTRAS ESPECIES Q LE ASIGNAN OTRA HP U ÓRGANOS)


common::define_entity_map_systems!(
    SpriteConfig,
    (With<game_common::game_common_components::EntityZero>, ),
    Sc,
    "sprite_config",
    "",
    SpriteConfig,
    common::common_components::StrId,
    SpriteConfigSeri,
    "seri.sprite.config",
    "sprite.ron",
);
