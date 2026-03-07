
#[allow(unused_imports)] use bevy::prelude::*;
use crate::prelude::*;


#[allow(unused_parens)]
pub fn add_hashed_tags(mut cmd: Commands, 
    mut query: Query<(&TagSet, &mut HashedTagsVec),(Changed<TagSet>, With<AddSameHashedTags>)>,
    mut removed: RemovedComponents<TagSet>,
) {
    query.iter_mut().for_each(|(tags, mut hashed_tags)| {
        *hashed_tags = HashedTagsVec::from(tags);
    });
    for ent in removed.read() {
        if query.get(ent).is_ok() {
            cmd.entity(ent).try_remove::<HashedTagsVec>();
        }
    }
}
