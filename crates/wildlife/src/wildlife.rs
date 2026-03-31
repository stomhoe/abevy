use bevy::prelude::*;
use game_common::HostSystems;
use tilemap_shared::NewMacrochunkLoaded;

use crate::{
    wildlife_cleanup_systems::*,
    wildlife_seeding_systems::*,
};

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_observer(on_pending_natural_spawn_unfreeze_despawn)
        .add_systems(
            Update,
            (
                request_macrochunk_biome_sampling.run_if(on_message::<NewMacrochunkLoaded>),
                seed_natural_wildlife_for_biomesampled_macrochunks,
            )
                .chain()
                .in_set(HostSystems),
        )
    ;
}
