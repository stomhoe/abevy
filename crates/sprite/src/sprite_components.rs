use bevy::math::{Vec2, UVec2};
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::Spritesheet;
use common::common_components::*;
use common::common_types::*;
use game_common::game_common_components::Category;
use serde::{Deserialize, Serialize};

use crate::sprite_scale_offset_components::Offset2D;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(EntityPrefix::new("SpriteConfig"), AssetScoped,)]
pub struct SpriteConfig;
//todo agregarle path_to_sprite y hacer q si no hay un sprite component en la entity q lo instancie
//de esta forma no hay q pre-spawnear los spriteconfig en los clients y se pueden replicar normalmente



#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect)]
#[relationship(relationship_target = HeldSprites)]
#[require(EntityPrefix::new("Sprite"), Replicated,)]
pub struct SpriteHolderRef {#[relationship]#[entities]pub base: Entity, }

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = SpriteHolderRef)]
pub struct HeldSprites(Vec<Entity>);
impl HeldSprites {pub fn sprite_ents(&self) -> &Vec<Entity> { &self.0 }}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy )]
pub struct WalkAnim;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy )]
pub struct SwimAnim{pub use_still: bool,}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy)]
pub struct FlyAnim{pub use_still: bool,}


#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct ExcludedFromBaseAnimPickingSystem;


#[derive(Component, Debug, Deserialize, Serialize,  Clone, Copy)]
pub enum FlipHorizIfDir{Left, Right, Any,}





#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct SpriteConfigRef(#[entities] pub Entity);





#[derive(Component, Debug, Default, Deserialize, Serialize, )]
pub struct AtlasLayoutData {pub spritesheet_size: UVec2, pub frame_size: UVec2,}

impl AtlasLayoutData {
    pub fn new(spritesheet_size: [u32; 2], frame_size: [u32; 2]) -> Self {
        Self { spritesheet_size: spritesheet_size.into(), frame_size: frame_size.into(), }
    }
}
impl AtlasLayoutData {
    pub fn into_texture_atlas(
        self,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> TextureAtlas {
        let spritesheet_size = self.spritesheet_size;
        let frame_size = self.frame_size;
        TextureAtlas {
            layout: atlas_layouts.add(
                Spritesheet::new(
                    spritesheet_size.y as usize,
                    spritesheet_size.x as usize,
                )
                .atlas_layout(frame_size.x, frame_size.y)
            ),
            ..Default::default()
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct ColorHolder(pub Color);//NO HACER PARTE DE SpriteDataBundle

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct OffsetForChildren(pub HashMap<Category, Offset2D>);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct Exclusive;

#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct BecomeChildOfSpriteWithCategory (pub Category);

// NO USAR ESTOS DOS PARA BEINGS
#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(Replicated)]
pub struct SpriteConfigStrIds(Vec<StrId>);
impl SpriteConfigStrIds {
    pub fn new<S: AsRef<str>>(ids: impl IntoIterator<Item = S>) -> Result<SpriteConfigStrIds, BevyError> {
        let ids: Result<Vec<StrId>, _> = ids.into_iter().map(|s| StrId::new_with_result(s.as_ref(), 3)).collect();
        Ok(SpriteConfigStrIds(ids?))
    }
    pub fn ids(&self) -> &Vec<StrId> { &self.0 }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone )]
pub struct SpriteCfgsToBuild(#[entities] pub HashSet<Entity>);


#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy )]
pub struct BecomeChildOf(#[entities] pub Entity);




