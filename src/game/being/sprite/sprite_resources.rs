use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, Serialize};

use crate::game::{being::sprite::sprite_components::*, game_components::{DisplayName, ImgPathHolder}};




#[derive(Resource, Debug, Default )]
pub struct SpriteDataIdEntityMap {map: HashMap<String, Entity>,}


#[allow(unused_parens)]
impl SpriteDataIdEntityMap {
    
    pub fn new_spritedata_from_seri(
        &mut self, cmd: &mut Commands,
        handle: Handle<SpriteDataSeri>,
        assets: &mut Assets<SpriteDataSeri>,
    ) {
        if let Some(mut seri) = assets.remove(&handle) {
            use std::mem::take;

            let spritedata_id = SpriteDataId::new(take(&mut seri.id));
            let disp_name = DisplayName(take(&mut seri.name));
            let path_holder = ImgPathHolder(take(&mut seri.path));
            let category = Category::new(take(&mut seri.category), seri.shares_category);
            let offset = Offset(Vec3::from_array(seri.offset));

            let atlas_data = AtlasLayoutData::new(seri.rows_cols, seri.frame_size);
            
            let entity = cmd.spawn((
                spritedata_id, 
                disp_name,
                path_holder,
                category,
                offset,
                atlas_data,
                
            )).id();

            if seri.directionable {cmd.entity(entity).insert(Directionable);}

            if seri.walk_anim {cmd.entity(entity).insert(WalkAnim);}
            if seri.swim_anim {cmd.entity(entity).insert(SwimAnim{use_still: seri.swim_anim_still});}
            if seri.fly_anim {cmd.entity(entity).insert(FlyAnim{use_still: seri.fly_anim_still});}
            match seri.flip_horiz{
                1 => {cmd.entity(entity).insert(FlipHorizIfDir::Any);},
                2 => {cmd.entity(entity).insert(FlipHorizIfDir::Left);},
                3 => {cmd.entity(entity).insert(FlipHorizIfDir::Right);},
                _ => {},
            };
            if let Some(scale) = seri.scale {
                let vec = Vec2::from_array(scale);
                if vec.x <= 0.0 || vec.y <= 0.0 {
                    warn!("SpriteDataSeri scale has non-positive component: {:?}", vec);
                } else {
                    cmd.entity(entity).insert(Scale(vec));
                }
            }

            if ! seri.anim_prefix.is_empty() {
                let anim_prefix = AnimationIdPrefix::new(take(&mut seri.anim_prefix));
                cmd.entity(entity).insert(anim_prefix);
            }
            
            if ! seri.children_sprites.is_empty(){
                let children_sprites = SpriteDatasChildrenStringIds(take(&mut seri.children_sprites));
                cmd.entity(entity).insert(children_sprites);
            }

            if let Some(color) = seri.color {
                let (red, green, blue, alpha) = color.into();
                cmd.entity(entity).insert(ColorHolder(Color::srgba_u8(red, green, blue, alpha)));
            }

            if let Some(offset_looking_down) = seri.offset_down {
                cmd.entity(entity).insert(OffsetLookDown(Vec2::from_array(offset_looking_down)));
            }
            if let Some(offset_looking_up) = seri.offset_up {
                cmd.entity(entity).insert(OffsetLookUp(Vec2::from_array(offset_looking_up)));
            }

            if let Some(offset_looking_up_down) = seri.offset_up_down {
                cmd.entity(entity).insert(OffsetLookUpDown(Vec2::from_array(offset_looking_up_down)));
            }
            if let Some(offset_looking_sideways) = seri.offset_sideways {
                cmd.entity(entity).insert(OffsetLookSideways(Vec2::from_array(offset_looking_sideways)));
            }
            if let Some(scale_looking_sideways) = seri.scale_sideways {
                let vec = Vec2::from_array(scale_looking_sideways);
                if vec.x <= 0.0 || vec.y <= 0.0 {
                    warn!("SpriteDataSeri scale_sideways has non-positive component: {:?}", vec);
                } else {
                    cmd.entity(entity).insert(ScaleLookSideWays(vec));
                }
            }

            if let Some(scale_looking_up_down) = seri.scale_up_down {
                let vec = Vec2::from_array(scale_looking_up_down);
                if vec.x <= 0.0 || vec.y <= 0.0 {
                    warn!("SpriteDataSeri scale_up_down has non-positive component: {:?}", vec);
                } else {
                    cmd.entity(entity).insert(ScaleLookUpDown(vec));
                }
            }

            self.map.insert(seri.id.clone(), entity);
        }
        else {
            warn!("SpriteDataSeri with handle {:?} not found in assets", handle);
        }
    }

    pub fn get_entity<S: Into<String>>(&self, spritedata_id: S) -> Option<Entity> {
        self.map.get(&spritedata_id.into()).copied()
    }

    pub fn get_entities(&self, spritedatas: &Vec<String>) -> Vec<Entity> {
        let mut entities = Vec::new();
        for spritedata in spritedatas {
            if let Some(entity) = self.get_entity(spritedata) {
                entities.push(entity);
            }
        }
        entities
    }
}

#[derive(AssetCollection, Resource)]
pub struct SpriteSerisHandles {
    #[asset(path = "spritedata", collection(typed))]
    pub handles: Vec<Handle<SpriteDataSeri>>,
}

