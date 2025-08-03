use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use crate::{common::common_components::DisplayName, game::{being::sprite::sprite_components::*, game_components::ImgPathHolder}};


#[derive(Resource, Debug, Default )]
pub struct SpriteDataIdEntityMap(pub HashMap<String, Entity>);


#[allow(unused_parens)]
impl SpriteDataIdEntityMap {
    
    pub fn new_spritedata_from_seri(
        &mut self, cmd: &mut Commands,
        handle: Handle<SpriteDataSeri>,
        assets: &mut Assets<SpriteDataSeri>,
    ) {
        if let Some(mut seri) = assets.remove(&handle) {
            
            if self.0.contains_key(&seri.id) {
                error!(target: "sprite_loading", "SpriteDataSeri with id {:?} already exists in map, skipping", seri.id);
                return;
            }
            let img_path = format!("assets/{}", seri.img_path);
            if !std::path::Path::new(&img_path).exists() {
                error!(target: "sprite_loading", "Image path does not exist: {}", img_path);
                return
            }
            let img_path = take(&mut seri.img_path);
            if seri.id.len() <= 2 {
                error!(target: "sprite_loading", "SpriteDataSeri id is too short or empty, skipping");
                return;
            }

            use std::mem::take;

            //TODO METER TODO ACÁ
            
            let spritedata_id = SpriteDataId::new(seri.id.clone());
            let path_holder = ImgPathHolder(img_path);

            let concatenated_cats = seri.categories.join(",");

            let categories: Categories = Categories::new(take(&mut seri.categories));
            
            let atlas_data = AtlasLayoutData::new(seri.rows_cols, seri.frame_size);

            let visib = match seri.visibility {
                0 => Visibility::default(), // inherited
                1 => Visibility::Visible,   // visible
                2 => Visibility::Hidden,    // hidden
                _ => {
                    error!(target: "sprite_loading", "Invalid visibility value: {}, falling back to inherited", seri.visibility);
                    Visibility::default()
                },
            };
            
            let sprdat_enti = cmd.spawn((
                spritedata_id.clone(), 
                path_holder,
                categories,
                atlas_data,
                visib,
            )).id();
            
            let mut comps_to_build: OtherCompsToBuild = OtherCompsToBuild::default();

            if seri.name.is_empty() {
                warn!(target: "sprite_loading", "SpriteDataSeri name is empty");
            } else {
                let disp_name = DisplayName::new(seri.name.clone());
                comps_to_build.display_name = Some(disp_name);
            }

            cmd.entity(sprdat_enti).insert((
                Name::new(format!("sprdatid:'{}' cats:'{}'", spritedata_id, concatenated_cats)),
            ));

            //if seri.exclusive { comps_to_build.exclusive = Some(Exclusive); }


            if seri.directionable {comps_to_build.directionable = Some(Directionable);}

            if ! seri.parent_cat.is_empty() {
                let to_become_child = ToBecomeChildOfCategory::new(take(&mut seri.parent_cat));
                comps_to_build.to_become_child_of_category = Some(to_become_child);
            }

            if ! seri.anim_prefix.is_empty() {
                comps_to_build.anim_prefix = Some(AnimationIdPrefix::new(take(&mut seri.anim_prefix)));
            }
            
            if ! seri.children_sprites.is_empty(){
                cmd.entity(sprdat_enti).insert(SpriteDatasChildrenStringIds(take(&mut seri.children_sprites)));
            }

            if ! seri.offset4children.is_empty(){
                let mut offset_children = OffsetForChildren::default();
                for (cat, offset_arr) in take(&mut seri.offset4children) {
                    offset_children.0.insert(Category::new(cat), Vec2::from_array(offset_arr));
                }
                comps_to_build.offset_children = Some(offset_children);
            }
            
            if let Some(color) = seri.color {
                let (red, green, blue, alpha) = color.into();
                comps_to_build.color = Some(ColorHolder(Color::srgba_u8(red, green, blue, alpha)));
            }


            if seri.walk_anim {comps_to_build.walk_anim = Some(WalkAnim);}
            if seri.swim_anim {comps_to_build.swim_anim = Some(SwimAnim{use_still: seri.swim_anim_still});}
            if seri.fly_anim {comps_to_build.fly_anim = Some(FlyAnim{use_still: seri.fly_anim_still});}

            let offset = Vec3::from_array(seri.offset);
            if offset != Vec3::ZERO {
                comps_to_build.offset = Some(Offset(offset));
            }

            if let Some(offset_looking_down) = seri.offset_down {
                comps_to_build.offset_looking_down = Some(OffsetLookDown(Vec2::from_array(offset_looking_down)));
            }
            if let Some(offset_looking_up) = seri.offset_up {
                comps_to_build.offset_looking_up = Some(OffsetLookUp(Vec2::from_array(offset_looking_up)));
            }

            if let Some(offset_looking_up_down) = seri.offset_up_down {
                comps_to_build.offset_looking_up_down = Some(OffsetLookUpDown(Vec2::from_array(offset_looking_up_down)));
            }
            if let Some(offset_looking_sideways) = seri.offset_sideways {
                comps_to_build.offset_looking_sideways = Some(OffsetLookSideways(Vec2::from_array(offset_looking_sideways)));

            if let Some(scale) = seri.scale {
                let vec = Vec2::from_array(scale);
                if vec.x <= 0.0 || vec.y <= 0.0 {
                    warn!(target: "sprite_loading", "SpriteDataSeri scale has non-positive component: {:?}", vec);
                } else {
                    comps_to_build.scale = Some(Scale(vec));
                }
            }
            }
            if let Some(scale_looking_sideways) = seri.scale_sideways {
                let vec = Vec2::from_array(scale_looking_sideways);
                if vec.x <= 0.0 || vec.y <= 0.0 {
                    warn!(target: "sprite_loading", "SpriteDataSeri scale_sideways has non-positive component: {:?}", vec);
                } else {
                    comps_to_build.scale_looking_sideways = Some(ScaleLookSideWays(vec));
                }
            }

            if let Some(scale_looking_up_down) = seri.scale_up_down {
                let vec = Vec2::from_array(scale_looking_up_down);
                if vec.x <= 0.0 || vec.y <= 0.0 {
                    warn!(target: "sprite_loading", "SpriteDataSeri scale_up_down has non-positive component: {:?}", vec);
                } else {
                    comps_to_build.scale_looking_up_down = Some(ScaleLookUpDown(vec));
                }
            }

            match seri.flip_horiz {
                1 => { comps_to_build.flip_horiz_if_dir = Some(FlipHorizIfDir::Any); },
                2 => { comps_to_build.flip_horiz_if_dir = Some(FlipHorizIfDir::Left); },
                3 => { comps_to_build.flip_horiz_if_dir = Some(FlipHorizIfDir::Right); },
                _ => {},
            };

            cmd.entity(sprdat_enti).insert(comps_to_build);
            
            self.0.insert(take(&mut seri.id), sprdat_enti);
        }
        else {
            warn!(target: "sprite_loading", "SpriteDataSeri with handle {:?} not found in assets", handle);
        }
    }

    pub fn get_entity<S: Into<String>>(&self, spritedata_id: S) -> Option<Entity> {
        self.0.get(&spritedata_id.into()).copied()
    }

    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity>
    where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(AssetCollection, Resource)]
pub struct SpriteSerisHandles {
    #[asset(path = "ron/sprite/spritedata", collection(typed))]
    pub handles: Vec<Handle<SpriteDataSeri>>,
}

