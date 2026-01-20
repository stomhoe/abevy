
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use crate::{
    common_components::*, common_resources::ImageSizeMap, common_tag_components::{AddSameHashedTags, HashedTagsVec, TagSet},
//    common_resources::*,
//    common_constants::*,
//    common_layout::*,
//    common_events::*,
};


#[allow(unused_parens)]
pub fn add_hashed_tags(mut cmd: Commands, 
    query: Query<(Entity, &TagSet),(Changed<TagSet>, With<AddSameHashedTags>)>,
    mut removed: RemovedComponents<TagSet>,
) {
    let mut tags_to_add = Vec::new();
    query.iter().for_each(|(ent, tags)| {
        let hashed_tags = HashedTagsVec::from(tags);
        tags_to_add.push((ent, hashed_tags));
    });
    for ent in removed.read() {
        if let Ok((_, _)) = query.get(ent) {
            cmd.entity(ent).try_remove::<HashedTagsVec>();
        }
    }
    cmd.try_insert_batch(tags_to_add);
}