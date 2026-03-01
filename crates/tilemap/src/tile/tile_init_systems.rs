
use ::game_common::{game_common_components::*, };
use game_common::game_common_samplers::GlobalTilePosWeightedSampler;
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
use common::{AnyDisabling, TILE_INIT, common_components::*, common_tag_components::TagSet};
use sprite::sprite_components::SpriteConfig;
use sprite_animation_shared::AcAnimationProgresses;
use std::{fs, path::PathBuf};

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

    let mut res_tile_tags = EzeroTileEntsWithinTag::default();

    for mut seri in load_tile_seri_defs() {

        let str_id = match TileStrId::new_with_result(seri.id.clone(), Tile::MIN_ID_LENGTH) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create TileStrId for tile '{}': {}", seri.id, err);
                return;
            }
        };
        let my_z = AcZ(seri.z);
        let size_in_tiles = SizeInTiles::new(&str_id, Some(seri.size_in_tiles), seri.is_spritetile);
        let tile_enti = cmd.spawn((
            Tile, Replicated, str_id.clone(), //PROBLEMA: EL DISABLED HACE Q EL DESPAWNONEXIT NO FUNCIONE
            Prefix::trunc("Tile"),
            my_z.clone(),
            EntityZero,
            AddHashIdFromStrId,
            ChildOf(holder),
            AssetScoped,
            size_in_tiles,
            //SparedFromHotReloading,
        )).id();
        cmd.entity(tile_enti).insert(OffsetForTerrgenPlacement(GlobalTilePos::new(
            seri.terrgen_offset.0 as i32,
            seri.terrgen_offset.1 as i32,
        )));

        let mut tag_set = TagSet::default();
        let self_tag = Tag::trunc(str_id.as_str());
        tag_set.insert(self_tag.clone());
        res_tile_tags.0.entry(self_tag).or_default().insert(tile_enti);
        if !seri.tags.is_empty() {
            for tag_string in &seri.tags {
                let tag_str = tag_string.trim();
                if tag_str.is_empty() { continue; }
                let tag = Tag::trunc(tag_str);
                tag_set.insert(tag.clone());
                res_tile_tags.0.entry(tag).or_default().insert(tile_enti);
            }
        }
        cmd.entity(tile_enti).insert(tag_set);

        let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
        let color = Color::srgba_u8(r, g, b, a);

        if ! seri.name.is_empty() {
            cmd.entity(tile_enti).insert(DisplayName(seri.name.clone()));
        }
        if seri.persisted || seri.portal.no_field_is_empty() {
            cmd.entity(tile_enti).insert(Persisted);
        }
        if seri.img_paths.is_empty() {
            warn!("Tile '{}' has no img_paths entries", str_id);
        }

        if let Some(ref mut adj_retex_config) = seri.adj_retex {
            cmd.entity(tile_enti).insert(AdjRetexConfig::new(std::mem::take(adj_retex_config)));
        }
        if !seri.interaction_zones.is_empty() {
            cmd.entity(tile_enti).insert(InteractionZones::new(std::mem::take(&mut seri.interaction_zones)));
        }

        if !seri.color_map.is_empty() {
            match color_map.0.get_cloned(&seri.color_map) {
                Ok(color_sampler_ent) => {
                    cmd.entity(tile_enti).insert(ColorSamplerRef(color_sampler_ent));
                }
                Err(_err) => {
                    error!("Tile '{}': Weighted color sampler with id '{}' not found in ColorSamplerEntityMap", str_id, seri.color_map);
                }
            }
        }
        if seri.randflipx {
            cmd.entity(tile_enti).insert(FlipHorizontallyBasedOnHash);
        }
        if seri.randflipy {
            cmd.entity(tile_enti).insert(FlipVerticallyBasedOnHash);
        }
        if seri.randflipd {
            cmd.entity(tile_enti).insert(FlipDiagonallyBasedOnHash);
        }
        if seri.portal.no_field_is_empty() {
            cmd.entity(tile_enti).insert((std::mem::take(&mut seri.portal), ChildOf(egui_portal_holder)));
        }

        let delete_other_tiles_seri = std::mem::take(&mut seri.delete_other_tiles);
        let delete_other_tiles = delete_other_tiles_seri.to_delete_other_tiles();
        if !delete_other_tiles.is_empty() {
            cmd.entity(tile_enti).insert(delete_other_tiles);
        }

        if !seri.offsets_for_portal_arrivals.is_empty() {
            let mut sampled_offsets = Vec::with_capacity(seri.offsets_for_portal_arrivals.len());
            for (weight, (x, y)) in &seri.offsets_for_portal_arrivals {
                sampled_offsets.push((GlobalTilePos::new(*x as i32, *y as i32), *weight));
            }
            cmd.entity(tile_enti).insert(GlobalTilePosWeightedSampler::new(&sampled_offsets));
        }

        cmd.entity(tile_enti).insert(WalkSpeedMultIfOnTop(seri.walk_speed));
        let mut weighted_paths = Vec::with_capacity(seri.step_sfx.groups.len() + 1);
        for (weight, paths) in &seri.step_sfx.groups {
            let paths = paths
                .iter()
                .filter_map(|path| {
                    let trimmed = path.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .collect::<Vec<_>>();
            if paths.is_empty() || *weight <= 0.0 {
                continue;
            }
            weighted_paths.push((paths, *weight));
        }
        if !seri.step_sfx.directory.trim().is_empty() && seri.step_sfx.directory_weight > 0.0 {
            let dir_paths = gather_step_sfx_paths_from_dir(&seri.step_sfx.directory);
            if !dir_paths.is_empty() {
                weighted_paths.push((dir_paths, seri.step_sfx.directory_weight));
            }
        }
        if !weighted_paths.is_empty() {
            cmd.entity(tile_enti).insert((
                TileStepSfx::new(&weighted_paths),
                TileStepSfxConfig {
                    prevent_repeat: seri.step_sfx.prevent_repeat,
                },
            ));
        }
        if ! seri.colmask.is_empty() {
            match TileCollisionMask::from_rows(&seri.colmask, size_in_tiles) {
                Ok(mask) => {
                    cmd.entity(tile_enti).insert(mask);
                }
                Err(err) => {
                    error!(
                        "Tile '{}' has invalid collision_mask: {}",
                        str_id,
                        err
                    );
                }
            }
        }

        if seri.blocks_projectiles {
            cmd.entity(tile_enti).insert(BlocksProjectiles);
        }



        if !seri.is_spritetile {
            cmd.entity(tile_enti).insert(TileImagePaths(std::mem::take(&mut seri.img_paths)));
            if seri.shader.trim() == "terrbl" {
                cmd.entity(tile_enti).insert(seri.terrbl_params.to_terrbl_params());
                let Ok(shader_ent) = shader_map.0.get_cloned("terrbl") else {
                    error!("Tile '{}' could not resolve single terrbl shader entity", str_id);
                    return;
                };
                cmd.entity(tile_enti).insert(TileShaderRef(shader_ent));
            }
            if let Some(y_sort_origin) = seri.y_sort {
                cmd.entity(tile_enti).insert(YSortOrigin(seri.offset.1 + y_sort_origin - 10.0));
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

                    if seri.offset != (0.0, 0.0) {
                        cmd.entity(child_sprite).insert(Offset2D::from(seri.offset));
                    }
                    if let Some(y_sort_origin) = seri.y_sort {
                        cmd.entity(child_sprite).insert(YSortOrigin(seri.offset.1 + y_sort_origin - 10.0));
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
    cmd.insert_resource(res_tile_tags);
}

fn gather_step_sfx_paths_from_dir(directory: &str) -> Vec<String> {
    let directory = directory.trim().trim_matches('/');
    if directory.is_empty() {
        return Vec::new();
    }

    let mut paths = Vec::new();
    let mut stack = vec![PathBuf::from("assets").join(directory)];
    while let Some(curr) = stack.pop() {
        let Ok(entries) = fs::read_dir(&curr) else { continue };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else { continue };
            let path = entry.path();
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else { continue };
            if !matches!(ext.to_ascii_lowercase().as_str(), "wav" | "ogg" | "mp3" | "flac") {
                continue;
            }
            let Ok(asset_rel) = path.strip_prefix("assets") else { continue };
            let Some(asset_rel) = asset_rel.to_str() else { continue };
            paths.push(asset_rel.replace('\\', "/"));
        }
    }
    paths.sort();
    paths
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
    tile_tags: Res<EzeroTileEntsWithinTag>,
) {
    let mut keep_away: EntityHashMap<EntityHashSet> = EntityHashMap::default();
    let all_seris = load_tile_seri_defs();
    let mut min_dist_comps = Vec::with_capacity(all_seris.len() / 10);
    let mut keep_dist_comps = Vec::with_capacity(all_seris.len() / 10);

    for seri in all_seris {

        let min_distances = seri.min_distances;
        if min_distances.is_empty() {
            continue;
        }

        let Ok(tile_ent) = tiles_map.0.get_cloned(&seri.id) else {
            continue;
        };

        let mut min_dists = MinDistancesMap::default();

        for (tile_id, min_dist) in min_distances {
            let lookup_tag = Tag::trunc(tile_id.as_str());
            let Some(tag_entities) = tile_tags.0.get(&lookup_tag) else {
                warn!(
                    "Tile '{}' min_distances references unknown tile id '{}'",
                    seri.id, tile_id
                );
                continue;
            };
            for tag_tile_ent in tag_entities {
                min_dists.0.insert(*tag_tile_ent, min_dist);
                if tag_tile_ent != &tile_ent {
                    keep_away.entry(*tag_tile_ent).or_default().insert(tile_ent);
                }
            }
        }
        if min_dists.0.is_empty() {
            continue;
        }
        min_dist_comps.push((tile_ent, min_dists));
    }

    for (tile_ent, ents) in keep_away {
        keep_dist_comps.push((tile_ent, KeepDistanceFrom(ents.into_iter().collect())));
    }
    cmd.try_insert_batch(min_dist_comps);
    cmd.try_insert_batch(keep_dist_comps);
}
#[allow(unused_parens)]
pub fn on_ezero_tile_despawn(
    on_despawn: On<Despawn, (Tile, EntityZero, TagSet)>,
    query: Query<(&TagSet), (AnyDisabling)>,
    mut tile_ents_within_tag: If<ResMut<EzeroTileEntsWithinTag>>
) {
    if let Ok(tag_set) = query.get(on_despawn.entity) {
        tag_set.iter().for_each(|tag| {
            if let Some(ents) = tile_ents_within_tag.0.0.get_mut(tag) {
                ents.remove(&on_despawn.entity);
            }
        });
    }
}
