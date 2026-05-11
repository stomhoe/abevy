use std::{collections::HashSet, };

use being_shared::Unloaded;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use game_common::game_common_components::{Templ, TemplEntiRef, };
use ::sprite_shared::*;
use ::tilemap_shared::*;
#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SpriteChangedScaleOrOffsetOrParent(pub Entity);

type SpriteOrMesh = Or<(With<Sprite>, With<Mesh2d>)>;
type ChangedDistResult = (Changed<SpriteGlobalNormalDistResult>, Changed<SpriteHoriNormalDistResult>, Changed<SpriteVertNormalDistResult>);
type ChangedScale = (Changed<Scale2D>, Changed<ScaleLookUpDown>, Changed<ScaleSideways>);
type ChangedSprite = (Changed<Sprite>, Changed<Mesh2d>);

#[allow(unused_parens)]
pub fn sprite_change_detection(
    sprite_query: Query<Entity, (Or<(ChangedScale, Changed<Rotation>, Changed<TemplEntiRef>, Changed<Offset2D>, ChangedSprite, Changed<ChildOf>, ChangedDistResult, Changed<FlippedTransform>)>, SpriteOrMesh)>,
    baseholder_query: Query<&HeldSprites, (Or<(Changed<CardinalDirection>, Changed<HeldSprites>, Added<GlobalTilePos>, Changed<Visibility>, ChangedDistResult, )>, Without<Unloaded>, )>,
    mut removed_unloaded: RemovedComponents<Unloaded>,
    mut removed_flipped_transform: RemovedComponents<FlippedTransform>,
    mut writer: MessageWriter<SpriteChangedScaleOrOffsetOrParent>,
    mut changed: Local<HashSet<SpriteChangedScaleOrOffsetOrParent>>,
)
{
    let sprite_iter = sprite_query.iter();
    let (sprite_lower, sprite_upper) = sprite_iter.size_hint();
    changed.reserve(sprite_upper.unwrap_or(sprite_lower));
    changed.extend(sprite_iter.map(SpriteChangedScaleOrOffsetOrParent));

    let baseholder_iter = baseholder_query.iter();
    let (base_lower, base_upper) = baseholder_iter.size_hint();
    changed.reserve(base_upper.unwrap_or(base_lower));
    for held_sprites in baseholder_iter {
        changed.extend(held_sprites.iter().map(SpriteChangedScaleOrOffsetOrParent));
    }

    let removed_iter = removed_unloaded.read().chain(removed_flipped_transform.read());
    let (removed_lower, removed_upper) = removed_iter.size_hint();
    changed.reserve(removed_upper.unwrap_or(removed_lower));
    changed.extend(removed_iter.map(SpriteChangedScaleOrOffsetOrParent));
    writer.write_batch(changed.drain());
}


#[allow(unused_parens)]
pub fn disable_held_sprites_of_disabled(
    mut cmd: Commands,
    templ_bases: Query<(&HeldSprites),(With<Templ>, Added<Disabled>)>,
    non_templ_bases: Query<(&HeldSprites),(Without<Templ>,)>,
    mut removed: RemovedComponents<Disabled>,
) {
    let iter = templ_bases.iter();
    let mut disableds = Vec::with_capacity(iter.size_hint().1.unwrap_or(iter.size_hint().0));
    for (held_sprites) in templ_bases.iter() {
        for sprite_ent in held_sprites.iter() {
            disableds.push((sprite_ent, Disabled));
        }
    }
    for ent in removed.read() {
        if let Ok((held_sprites)) = non_templ_bases.get(ent) {
            for sprite_ent in held_sprites.iter() {
                cmd.entity(sprite_ent).try_remove::<Disabled>();
            }
        }
    }
    cmd.try_insert_batch(std::mem::take(&mut disableds));
}
