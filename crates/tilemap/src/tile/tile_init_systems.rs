
use ::game_common::{game_common_components::*, };
use ::sprite_shared::{sprite_scale_offset::Offset2D, *};
use ::tilemap_shared::*;
#[allow(unused_imports)]
use bevy::{
    prelude::*,
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::HashSet,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_replicon::prelude::*;
use color_sampler::{ColorSamplerEntityMap, ColorSamplerRef,};
use common::{TILE_INIT, common_components::*, common_tag_components::TagSet};
use sprite::sprite_components::SpriteConfig;
use sprite_animation_shared::AcAnimationProgresses;

use crate::{
    tile::{
        tile_components::*,
        tile_resources::*,
        tile_shader::{tile_shader_components::*, tile_shader_resources::*},
    },
};
#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands,
    shader_map: Res<TileShaderEntityMap>,
    tiling_map: Res<TileEntityMap>,
    color_map: Res<ColorSamplerEntityMap>,
    egui_tiles_holder_query: Query<Entity, With<EguiTilesHolder>>,
) {
    if !tiling_map.0.0.is_empty() {
        return;
    }
    let holder = if let Ok(first_holder) = egui_tiles_holder_query.single() {
        first_holder
    } else {
        cmd.spawn((EguiTilesHolder,)).id()
    };

    let egui_portal_holder = cmd.spawn((PortalsZeroEguiHolder, ChildOf(holder))).id();

    let mut res_tile_cats = TileEntsWithinTag::default();

    for mut seri in load_tile_seri_defs() {

        let str_id = match TileStrId::new_with_result(seri.id.clone(), Tile::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create TileStrId for tile '{}': {}", seri.id, err);
                return;
            }
        };
        let my_z = AcZ(seri.z);
        let tile_enti = cmd.spawn((
            Tile, Replicated, str_id.clone(), //PROBLEMA: EL DISABLED HACE Q EL DESPAWNONEXIT NO FUNCIONE
            Prefix::trunc("Tile"),
            my_z.clone(),
            EntityZero,
            AddHashIdFromStrId,
            ChildOf(holder),
            AssetScoped,
            SizeInTiles::new(seri.size_in_tiles),
            //SparedFromHotReloading,
        )).id();

        if let Some(tags) = &seri.tags {
            let mut tag_set = TagSet::default();
            for tag_string in tags {
                let tag_str = tag_string.trim();
                if tag_str.is_empty() { continue; }
                let tag = Tag::trunc(tag_str);
                tag_set.insert(tag.clone());
                res_tile_cats.0.entry(tag).or_default().insert(tile_enti);
            }
            cmd.entity(tile_enti).insert(tag_set);
        }

        let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
        let color = Color::srgba_u8(r, g, b, a);

        if ! seri.name.is_empty() {
            cmd.entity(tile_enti).insert(DisplayName(seri.name.clone()));
        }
        if seri.portal.is_some() {
            cmd.entity(tile_enti).insert(Persisted);
        }
        if seri.img_paths.is_empty() {
            warn!("Tile '{}' has no img_paths entries", str_id);
        }

        if let Some(ref mut adj_retex_config) = seri.adj_retex {
            cmd.entity(tile_enti).insert(AdjRetexConfig::new(std::mem::take(adj_retex_config)));
        }

        if let Some(ref color_map_str) = seri.color_map {
            if !color_map_str.is_empty() {
                match color_map.0.get_cloned(color_map_str) {
                    Ok(color_sampler_ent) => {
                        cmd.entity(tile_enti).insert(ColorSamplerRef(color_sampler_ent));
                    }
                    Err(_err) => {
                        error!("Tile '{}': Weighted color sampler with id '{}' not found in ColorSamplerEntityMap", str_id, color_map_str);
                    }
                }
            }
        }
        if seri.randflipx == Some(true) {
            cmd.entity(tile_enti).insert(FlipHorizontallyBasedOnHash);
        }
        if let Some(portal) = &mut seri.portal {
            cmd.entity(tile_enti).insert((std::mem::take(portal), ChildOf(egui_portal_holder)));
        }

        if let Some(ws) = seri.walk_speed {
            cmd.entity(tile_enti).insert(WalkSpeedMultIfOnTop(ws));
        } else{
            cmd.entity(tile_enti).insert(WalkSpeedMultIfOnTop(1.0));
        }

        if seri.blocks_projectiles == Some(true) {
            cmd.entity(tile_enti).insert(BlocksProjectiles);
        }



        if seri.sprite != Some(true) {
            cmd.entity(tile_enti).insert(TileImagePaths(std::mem::take(&mut seri.img_paths)));

            if let Some(shader_str) = &seri.shader {
                if shader_str.len() > 2 {
                    let Ok(shader_ent) = shader_map.0.get_cloned(shader_str) else {
                        error!("Tile '{}' references shader {} not found in TileShaderEntityMap", str_id, shader_str);
                        return;
                    };
                    cmd.entity(tile_enti).insert(TileShaderRef(shader_ent));
                } else if shader_str.len() > 0 {
                    warn!("Tile {} shader {} is too short for a shader", str_id, shader_str);
                }
            }
            if let Some(y_sort_origin) = seri.y_sort {
                cmd.entity(tile_enti).insert(YSortOrigin(seri.offset.unwrap_or_default().1 + y_sort_origin - 10.0));
            }

            cmd.entity(tile_enti).insert_if_new((TileColor::from(color), ));
        } else {
            cmd.entity(tile_enti).insert((
                Transform::from_translation(Vec2::splat(f32::INFINITY).extend(0.)),
                Visibility::default(),
                SpriteTile
            ));
            let mut sprite_cfgs = Vec::new();
            let mut processing_as_sprite_cfgs = None;

            let len = seri.img_paths.len();
            for (key, path) in seri.img_paths.iter_mut() {
                let path_holder = ImagePathHolder::new(path.clone());
                let spritecfg_str_id_present = !key.trim().is_empty();

                    if path_holder.is_err() && spritecfg_str_id_present
                && processing_as_sprite_cfgs != Some(false) {
                    sprite_cfgs.reserve(len);
                    sprite_cfgs.push(std::mem::take(key));
                    processing_as_sprite_cfgs = Some(true);
                } else if processing_as_sprite_cfgs != Some(true) {
                    let path_holder = path_holder.unwrap();

                    let child_sprite = cmd.spawn((
                        TileChildSprite,
                        ChildOf(tile_enti),
                        BaseHolderRef{ base: tile_enti },
                        StrId::trunc(format!("{}", path_holder).replace("texture/", "")),
                        EntityZero,
                        path_holder,
                        Replicated,
                        my_z.clone(),
                    )).id();

                    if let Some(offset) = seri.offset {
                        cmd.entity(child_sprite).insert(Offset2D::from(offset));
                    }
                    if let Some(y_sort_origin) = seri.y_sort {
                        cmd.entity(child_sprite).insert(YSortOrigin(seri.offset.unwrap_or_default().1 + y_sort_origin - 10.0));
                    }
                    processing_as_sprite_cfgs = Some(false);
                }
            }
            if !sprite_cfgs.is_empty() {
                let sprite_cfgs_str_ids = SampleSpritesFromStrIds::new(sprite_cfgs);
                cmd.entity(tile_enti).insert(sprite_cfgs_str_ids);
            }
        }
    }
    cmd.insert_resource(res_tile_cats);
}

#[allow(unused_parens)]
pub fn init_childrensprite(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    ezero_img_path: Query<(Option<&ImagePathHolder>, Has<SpriteConfig>), (With<EntityZero>,)>,
    childrensprite_query: Query<
        (Entity, AnyOf<(&ImagePathHolder, &EntityZeroRef)>),
        (
            Without<AcAnimationProgresses>,
            Or<(Changed<ImagePathHolder>, Changed<EntityZeroRef>)>,
            With<TileChildSprite>,
            Without<Sprite>,
            Without<TilemapId>,
            Without<Children>,
            Without<TileShader>,
            common::AnyDisabling,
        ),
    >,
) {
    let mut to_insert = Vec::new();
    for (entity, (image_path_holder, ezero_ref)) in childrensprite_query.iter() {
        if let Some(img_path_holder) = image_path_holder {
            trace!(target: "childrensprite_init","Inserting Sprite for entity {:?} with direct ImagePathHolder: {:?}", entity, img_path_holder.path());
            to_insert.push((
                entity,
                Sprite {
                    image: asset_server.load(img_path_holder.path()),
                    ..Default::default()
                },
            ));
        } else if let Some(ezero_ref) = ezero_ref {
            let Ok((img_path_holder, is_ezero_a_spriteconfig)) = ezero_img_path.get(ezero_ref.0)
            else {
                error!(target: "childrensprite_init","Entity {:?} has EntityZeroRef {:?} but the referenced entity doesn't exist", entity, ezero_ref.0);
                continue;
            };
            if is_ezero_a_spriteconfig {
                continue;
            }
            let Some(img_path_holder) = img_path_holder else {
                error!(target: "childrensprite_init","Entity {:?} has EntityZeroRef {:?} but the referenced entity has no ImagePathHolder", entity, ezero_ref.0);
                continue;
            };

            trace!(target: "childrensprite_init","Inserting Sprite for entity {:?} via EntityZeroRef {:?}, path: {:?}", entity, ezero_ref.0, img_path_holder.path());
            to_insert.push((
                entity,
                Sprite {
                    image: asset_server.load(img_path_holder.path()),
                    ..Default::default()
                },
            ));
        } else {
            error!(target: "childrensprite_init","Entity {:?} has neither ImagePathHolder nor EntityZeroRef", entity);
        }
    }
    cmd.try_insert_batch(to_insert);
}

#[allow(unused_parens)]
pub fn add_handles(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    ezero_id_query: Query<
        (Entity, &TileStrId, &TileImagePaths),
        (
            With<EntityZero>,
            Without<TileHashIdsHandles>,
            Changed<TileImagePaths>,
        ),
    >,
) {
    let mut comps = Vec::new();
    for (enti, str_id, tile_image_paths) in ezero_id_query.iter() {
        let tile_handles = TileHashIdsHandles::from_paths(&asset_server, tile_image_paths.clone());

        match tile_handles {
            Ok(tile_handles) => {
                trace!(target: TILE_INIT, "Adding TileHandles for tile '{}'", str_id);
                comps.push((enti, tile_handles));
            }
            Err(err) => {
                error!(target: TILE_INIT, "Failed to create TileHandles for tile '{}': {}", str_id, err);
            }
        }
    }
    cmd.try_insert_batch(comps);
}

#[allow(unused_parens)]
pub fn map_min_dist_tiles(
    mut cmd: Commands,
    tiles_map: Res<TileEntityMap>,
    tile_cats: Res<TileEntsWithinTag>,
) {
    let mut keep_away: EntityHashMap<HashSet<Entity>> = EntityHashMap::default();
    let all_seris = load_tile_seri_defs();
    let mut comps = Vec::with_capacity(all_seris.len() / 10);
    let mut comps2 = Vec::with_capacity(all_seris.len() / 10);

    for seri in all_seris {

        let Some(min_distances) = seri.min_distances else {
            continue;
        };

        if min_distances.is_empty() {
            continue;
        }

        let Ok(tile_ent) = tiles_map.0.get_cloned(&seri.id) else {
            continue;
        };

        let mut min_dists = MinDistancesMap::default();

        for (tile_id, min_dist) in min_distances {
            if let Some(cat) = tile_id.strip_prefix("c.")
                && let Some(cat_entities) = tile_cats.0.get(&Tag::trunc(cat))
            {
                for cat_tile_ent in cat_entities {
                    min_dists.0.insert(*cat_tile_ent, min_dist);
                    if cat_tile_ent != &tile_ent {
                        keep_away.entry(*cat_tile_ent).or_default().insert(tile_ent);
                    }
                }
            } else if let Ok(other_tile_ent) = tiles_map.0.get_cloned(&tile_id) {
                min_dists.0.insert(other_tile_ent, min_dist);
                if other_tile_ent != tile_ent {
                    keep_away
                        .entry(other_tile_ent)
                        .or_default()
                        .insert(tile_ent);
                }
            } else {
                warn!(
                    "Tile '{}' min_distances references unknown tile id '{}'",
                    seri.id, tile_id
                );
                continue;
            };
        }

        if min_dists.0.is_empty() {
            continue;
        }

        comps.push((tile_ent, min_dists));
    }

    for (tile_ent, ents) in keep_away {
        comps2.push((tile_ent, KeepDistanceFrom(ents.into_iter().collect())));
    }
    cmd.try_insert_batch(comps);
    cmd.try_insert_batch(comps2);
}
