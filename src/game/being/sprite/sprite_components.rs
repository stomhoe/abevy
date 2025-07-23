use bevy::ecs::component;
use bevy::math::{Vec2, Vec3, U16Vec2, UVec2};
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use crate::game::{being::sprite::{animation_constants::*, sprite_constants::* }, game_components::*};


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct AnimationIdPrefix(pub String);
impl AnimationIdPrefix {
    pub fn new<S: Into<String>>(prefix: S) -> Self {
        Self(prefix.into())
    }
    pub fn prefix(&self) -> &str {
        &self.0
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct SpriteHolderRef(#[entities] pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize,)]
//NO VA REPLICATED, SE HACE LOCALMENTE EN CADA PC SEGÚN LOS INPUTS RECIBIDOS DE OTROS PLAYERS
pub struct AnimationState(pub String);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy )]
pub struct WalkAnim;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy )]
pub struct SwimAnim{pub use_still: bool,}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct FlyAnim{pub use_still: bool,}


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


#[derive(Component, Debug, Deserialize, Serialize,  Clone, Copy)]
pub enum FlipHorizIfDir{Left, Right, Any,}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directionable;


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct AtlasLayoutData{pub spritesheet_size: UVec2, pub frame_size: UVec2,}
impl AtlasLayoutData {
    pub fn new(spritesheet_size: [u32; 2], frame_size: [u32; 2]) -> Self {
        Self { spritesheet_size: spritesheet_size.into(), frame_size: frame_size.into(), }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ColorHolder(pub Color);//NO HACER PARTE DE SpriteDataBundle

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct SpriteDataId(String);
impl SpriteDataId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
    pub fn id(&self) -> &str { &self.0 }
}
impl Into<String> for SpriteDataId {
    fn into(self) -> String {self.0}
}
#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct Scale(pub Vec2);
impl Default for Scale {fn default() -> Self {Self(Vec2::ONE)}}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct ScaleLookUpDown(pub Vec2);
impl Default for ScaleLookUpDown {fn default() -> Self {Self(Vec2::ONE)}}

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct ScaleLookSideWays(pub Vec2);
impl Default for ScaleLookSideWays {fn default() -> Self {Self(Vec2::ONE)}}


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct OtherCompsToBuild{
    pub display_name: Option<DisplayName>,
    pub anim_prefix: Option<AnimationIdPrefix>,
    pub directionable: Option<Directionable>,
    pub walk_anim: Option<WalkAnim>,
    pub swim_anim: Option<SwimAnim>,
    pub fly_anim: Option<FlyAnim>,
    pub to_become_child_of_category: Option<ToBecomeChildOfCategory>,
    pub offset: Option<Offset>,
    pub offset_looking_down: Option<OffsetLookDown>,
    pub offset_looking_up: Option<OffsetLookUp>,
    pub offset_looking_left: Option<OffsetLookLeft>,
    pub offset_looking_right: Option<OffsetLookRight>,
    pub offset_looking_sideways: Option<OffsetLookSideways>,
    pub offset_looking_up_down: Option<OffsetLookUpDown>,
    pub scale: Option<Scale>,
    pub scale_looking_up_down: Option<ScaleLookUpDown>,
    pub scale_looking_sideways: Option<ScaleLookSideWays>,
    pub flip_horiz_if_dir: Option<FlipHorizIfDir>,
    pub color: Option<ColorHolder>,

}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct Offset(pub Vec3);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetLookUpDown(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetLookDown(pub Vec2);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetLookUp(pub Vec2);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetLookSideways(pub Vec2);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetLookRight(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetLookLeft(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OffsetGivenByParent(pub Vec2);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct OffsetForChildren(pub HashMap<Category, Vec2>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ToBecomeChildOfCategory (pub Category);
impl ToBecomeChildOfCategory {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(Category::new(id))
    }
    pub fn category(&self) -> &Category {
        &self.0
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash,)]
pub struct Category { pub id: u64, /*pub shared: bool,*/ }//importante: el equal tiene en cuenta el shared
impl Category {
    pub fn new<S: Into<String>>(id: S, /*shared: bool*/) -> Self {
        let id_str = id.into();
        let mut hasher = DefaultHasher::new();
        id_str.hash(&mut hasher);
        Self { id: hasher.finish(), /*shared*/ }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Category({})", self.id)
    }
}


// NO USAR ESTOS DOS PARA BEINGS
#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct SpriteDatasChildrenStringIds(pub Vec<String>);
impl SpriteDatasChildrenStringIds {
    pub fn new<S: Into<String>>(ids: impl IntoIterator<Item = S>) -> Self {
        Self(ids.into_iter().map(|s| s.into()).collect())
    }
    pub fn ids(&self) -> &Vec<String> {
        &self.0
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone )]
pub struct SpriteDatasChildrenRefs(#[entities] pub Vec<Entity>);

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct SpriteDataSeri {
    pub id: String,
    pub name: String,
    pub path: String,
    pub category: String,
    pub shares_category: bool,
    pub rows_cols: [u32; 2], 
    pub frame_size: [u32; 2],
    pub offset: [f32; 3],
    pub directionable: bool,
    pub walk_anim: bool,
    pub swim_anim: bool,
    pub swim_anim_still: bool,
    pub fly_anim: bool,
    pub fly_anim_still: bool,
    pub flip_horiz: u8, //0: none, 1: any, 2: if looking left, 3: if looking right
    pub anim_prefix: String,
    pub visibility: u8, //0: inherited, 1: visible, 2: invisible
    pub parent_cat: String, //adds ChildOf referencing other brother entity sprite possessing this category
    pub offset_for_others: HashMap<String, [f32; 2]>,//category, offset
    pub children_sprites: Vec<String>,// these will get spawned as children of the entity that has this sprite data
    pub offset_down: Option<[f32; 2]>,
    pub offset_up: Option<[f32; 2]>,
    pub offset_sideways: Option<[f32; 2]>,
    pub offset_up_down: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub scale_up_down: Option<[f32; 2]>,
    pub scale_sideways: Option<[f32; 2]>,
    pub color: Option<[u8; 4]>, 
    pub exclude_from_sys: Option<bool>,
}
// TODO: hacer shaders aplicables? (para meditacion por ej)
// TODO: hacer que se puedan aplicar colorses sobre máscaras como en humanoid alien races del rimworld. hacer un mapa color-algo 

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct AnimationSeri {
    pub id: String,
    pub sheet_rows_cols: [u32; 2], //rows, cols
    pub target: u32,
    pub is_row: bool, //true: target is a row , false: target is a column
    pub partial: Option<[u32; 2]>, //start, end inclusive (0-indexed)
}