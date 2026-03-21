#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn init_dimensions(
    mut cmd: Commands, map: Res<DimensionEntityMap>,
) {
    if !map.0.is_empty(){ return; }

    let db = match common::def_db::DefDatabase::<DimensionSeri>::load_from_assets_dir(
        &["dimension.ron"],
        |d| d.id.as_str(),
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(target: "dimension_loading", "{err:#}");
            return;
        }
    };
    if !db.overrides().is_empty() {
        for ov in db.overrides() {
            info!(
                target: "dimension_loading",
                "Dimension def '{}' overridden: '{}' -> '{}'",
                ov.id,
                ov.previous_source.rel_path,
                ov.replacement_source.rel_path
            );
        }
    }

    let mut common_components = Vec::new();
    let mut tagsets_to_insert = Vec::new();
    let mut whitelisted_structure_gen_tags_to_insert = Vec::new();
    let mut blacklisted_structure_gen_tags_to_insert = Vec::new();

    for record in db.iter() {
        let seri = &record.value;

        let str_id = match StrId::new_with_result(seri.id.clone(), 2) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for dimension {}: {}", seri.id, e));
                error!(target: "dimension_loading", "{}", err);
                continue;
            }
        };
        let dim_ent = cmd.spawn_empty().id();

        if !seri.tags.is_empty() {
            tagsets_to_insert.push((dim_ent, TagSet::new(seri.tags.clone())));
        }
        if !seri.whitelisted_structure_gen_tags.is_empty() {
            let tag_set = WhitelistedStructureGenTags(TagSet::new(seri.whitelisted_structure_gen_tags.clone()));
            whitelisted_structure_gen_tags_to_insert.push((dim_ent, tag_set));
        }
        if !seri.blacklisted_structure_gen_tags.is_empty() {
            let tag_set = BlacklistedStructureGenTags(TagSet::new(seri.blacklisted_structure_gen_tags.clone()));
            blacklisted_structure_gen_tags_to_insert.push((dim_ent, tag_set));
        }

        common_components.push((dim_ent, (
            HashId::from(str_id.as_ref()),
            str_id,
            Transform::default(),
            DisplayName::new(seri.name.clone()),
            Dimension,
            Gravity(seri.gravity.max(1.0)),
            Visibility::Visible,
        )))
    }
    cmd.insert_batch(common_components);
    cmd.insert_batch(tagsets_to_insert);
    cmd.insert_batch(whitelisted_structure_gen_tags_to_insert);
    cmd.insert_batch(blacklisted_structure_gen_tags_to_insert);
}

#[allow(unused_parens, )]
pub fn spawn_egui_macro_chunk_holders(
    mut cmd: Commands,
    query: Query<Entity, (With<Dimension>, Without<MacroChunkHolderRef>)>,
) {
    for dim_ent in query.iter() {
        let holder_ent = cmd
            .spawn((
                Name::new("EguiMacroChunkHolder"),
                ChildOf(dim_ent),
            ))
            .id();
        cmd.entity(dim_ent).try_insert(MacroChunkHolderRef(holder_ent));
    }
}
