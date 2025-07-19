use bevy::math::U16Vec2;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::{being::sprite::{animation_constants::*, sprite_constants::* }, game_components::*};


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct AnimationIdPrefix(pub String);
impl AnimationIdPrefix {
    pub fn new<S: Into<String>>(prefix: S) -> Self {
        Self(prefix.into())
    }
    pub fn prefix(&self) -> &str {
        &self.0
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize,)]
//NO VA REPLICATED, SE HACE LOCALMENTE EN CADA PC SEGÚN LOS INPUTS RECIBIDOS DE OTROS PLAYERS
pub struct AnimationState(pub String);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct HasWalkAnim;

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct HasSwimAnim{pub use_still: bool,}
#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct HasFlyAnim{pub use_still: bool,}


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ExcludedFromBaseAnimPickingSystem;


impl AnimationState {
    pub fn new_idle() -> Self {
        Self(IDLE.to_string())
    }
    pub fn set_idle(&mut self) {
        self.0 = IDLE.to_string();
    }
    pub fn set_walk(&mut self) {
        self.0 = WALK.to_string();
    }
    pub fn set_swim(&mut self) {
        self.0 = SWIM.to_string();
    }
    pub fn set_fly(&mut self) {
        self.0 = FLY.to_string();
    }

}


#[derive(Component, Debug, Deserialize, Serialize, )]
pub enum FlipXIfDir{Left, Right, Any,}

#[derive(Component, Debug, Deserialize, Serialize, )]
pub struct Directionable;

#[derive(Bundle)]
pub struct SpriteEssentialComps{
    pub img_path_holder: ImgPathHolder, pub animation_state: AnimationState, 
    pub atlas_layout_data: AtlasLayoutData,
}
impl SpriteEssentialComps {
    pub fn new<S: Into<String>>(img_path: S, sprite_sheet_size: U16Vec2, frame_size: U16Vec2,) -> Self {
        Self {
            img_path_holder: ImgPathHolder(img_path.into()),
            animation_state: AnimationState::default(),
            atlas_layout_data: AtlasLayoutData::new(sprite_sheet_size, frame_size),
        }
    }
}

#[derive(Bundle)]
pub struct DefaultHeadBundle{
    pub sprite_data_bundle: SpriteEssentialComps, directionable: Directionable, prefix: AnimationIdPrefix,
}
impl DefaultHeadBundle {
    pub fn new<S: Into<String>>(img_path: S) -> Self {
        Self {
            sprite_data_bundle: SpriteEssentialComps::new(img_path, BASE_HEAD_SPRITESHEET_SIZE, BASE_HEAD_FRAME_SIZE,),
            directionable: Directionable, prefix: AnimationIdPrefix::new(HEAD)
        }
    }
}


#[derive(Bundle)]
pub struct DefaultBodyBundle{
    pub sprite_data_bundle: SpriteEssentialComps, has_walk_anim: HasWalkAnim, directionable: Directionable, prefix: AnimationIdPrefix,
}
impl DefaultBodyBundle {
    pub fn new<S: Into<String>>(img_path: S) -> Self {
        Self {
            sprite_data_bundle: SpriteEssentialComps::new(img_path, BASE_BODY_SPRITESHEET_SIZE, BASE_BODY_FRAME_SIZE,),
            has_walk_anim: HasWalkAnim, directionable: Directionable, prefix: AnimationIdPrefix::new(BODY),
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct AtlasLayoutData{pub sprite_sheet_size: U16Vec2, pub frame_size: U16Vec2,}
impl AtlasLayoutData {
    pub fn new(sprite_sheet_size: U16Vec2, frame_size: U16Vec2) -> Self {
        Self { sprite_sheet_size, frame_size, }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ColorHolder(Color);//NO HACER PARTE DE SpriteDataBundle

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct SpriteDataId(String);
impl SpriteDataId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
    pub fn id(&self) -> &str {&self.0}
}
#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Scale(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct Offset(pub Vec3);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffsetLookingDown(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffsetLookingUp(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffsetLookingSideways(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffsetLookingRight(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OffsetLookingLeft(pub Vec2);


#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct SpriteDataSeri {
    pub id: String,
    pub name: String,
    pub path: String,
    pub category: String,
    pub shares_category: bool,
    pub children_spriteids: Vec<String>,
    pub offset: [f32; 3],
    pub offset_down: Option<[f32; 2]>,
    pub offset_up: Option<[f32; 2]>,
    pub offset_sideways: Option<[f32; 2]>,
    pub color: [f32; 4], 
    pub rows_cols: [u16; 2], 
    pub frame_size: [u16; 2],
    pub animation_id: String,
    pub directionable: bool,
    pub walk_anim: bool,
    pub swim_anim: bool,
    pub swim_anim_still: bool,
    pub fly_anim: bool,
    pub fly_anim_still: bool,
    pub exclude_auto_anim_switching: Option<bool>,
}

