use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use common::log_targets;
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use item_shared::{clone_item_from_ezero, dropped_scs_to_build, Item, ItemHeldIn};
use param_sets::BlockingTileParamSet;
use ::sprite_shared::AcZ;
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(SystemParam)]
pub struct ItemGroundMaterializeParamSet<'w, 's> {
    blocking_tiles: BlockingTileParamSet<'w, 's>,
    item_cfg_query: Query<'w, 's, &'static item_shared::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
    item_ground_query: Query<'w, 's, (&'static EntityZeroRef, &'static DimensionRef, &'static GlobalTilePos), (With<Item>, Without<EntityZero>)>,
    ac_z_query: Query<'w, 's, &'static AcZ>,
    occupied_nonstackable: Local<'s, HashSet<GlobalTilePos>>,
}

impl<'w, 's> ItemGroundMaterializeParamSet<'w, 's> {
    fn find_nonstackable_drop_gpos(&mut self, dim_ref: Option<DimensionRef>, drop_gpos: GlobalTilePos) -> GlobalTilePos {
        let occupied_nonstackable: &mut HashSet<GlobalTilePos> = &mut self.occupied_nonstackable;
        let mut next = drop_gpos;
        if let Some(dim_ref) = dim_ref {
            'nonstack_search: for dist in 0..i32::MAX {
                for dx in -dist..=dist {
                    let dy_abs = dist - dx.abs();
                    let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
                    if !occupied_nonstackable.contains(&cand_a)
                        && !self.blocking_tiles.is_blocked_at_tiles_only(dim_ref, cand_a, Entity::PLACEHOLDER)
                    {
                        next = cand_a;
                        break 'nonstack_search;
                    }
                    if dy_abs == 0 {
                        continue;
                    }
                    let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
                    if !occupied_nonstackable.contains(&cand_b)
                        && !self.blocking_tiles.is_blocked_at_tiles_only(dim_ref, cand_b, Entity::PLACEHOLDER)
                    {
                        next = cand_b;
                        break 'nonstack_search;
                    }
                }
            }
        } else {
            for dist in 0..i32::MAX {
                for dx in -dist..=dist {
                    let dy_abs = dist - dx.abs();
                    let cand_a = drop_gpos + GlobalTilePos::new(dx, dy_abs);
                    if !occupied_nonstackable.contains(&cand_a) {
                        next = cand_a;
                        break;
                    }
                    if dy_abs == 0 {
                        continue;
                    }
                    let cand_b = drop_gpos + GlobalTilePos::new(dx, -dy_abs);
                    if !occupied_nonstackable.contains(&cand_b) {
                        next = cand_b;
                        break;
                    }
                }
                if !occupied_nonstackable.contains(&next) {
                    break;
                }
            }
        }
        occupied_nonstackable.insert(next);
        next
    }

    pub fn materialize_item_on_ground(&mut self, cmd: &mut Commands, item_ent: Entity) {
        let Ok((&item_ezero_ref, &dim_ref, &gpos)) = self.item_ground_query.get(item_ent) else {
            warn!(
                target: log_targets::ITEM_SYSTEM,
                "Skipping ground materialization: item {:?} is missing EntityZeroRef/DimensionRef/GlobalTilePos",
                item_ent,
            );
            return;
        };
        self.materialize_item_on_ground_from_ezero(cmd, Some(item_ent), item_ezero_ref, dim_ref, gpos);
    }

    pub fn materialize_item_on_ground_from_ezero(
        &mut self,
        cmd: &mut Commands,
        item_ent: Option<Entity>,
        item_ezero_ref: EntityZeroRef,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
    ) {
        let item_ent = item_ent.unwrap_or_else(|| clone_item_from_ezero(cmd, item_ezero_ref, dim_ref));
        let gpos = self.find_nonstackable_drop_gpos(Some(dim_ref), gpos);
        let z = self
            .blocking_tiles
            .gather_tiles_at(dim_ref, gpos)
            .iter()
            .copied()
            .filter_map(|ent| self.ac_z_query.get(ent).ok().map(|&AcZ(z)| z))
            .max_by(f32::total_cmp)
            .unwrap_or_default()
            + 1.0;
        let mut item_cmd = cmd.entity(item_ent);
        item_cmd.remove::<ItemHeldIn>();
        item_cmd.insert((
            gpos,
            AcZ(z),
            ChildOf(dim_ref.0),
        ));
        let Some(scs_to_build) = dropped_scs_to_build(&self.item_cfg_query, item_ezero_ref) else {
            return;
        };
        item_cmd.insert(scs_to_build);
        debug!(
            target: log_targets::ITEM_SYSTEM,
            "Materialized item {:?} (ezero {:?}) on ground at dim={:?} gpos={:?}",
            item_ent,
            item_ezero_ref.0,
            dim_ref.0,
            gpos,
        );
    }
}
