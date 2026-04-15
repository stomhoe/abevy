use bevy::prelude::*;
use game_common::{GameplaySystems, HostSystems};

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
                request_macrochunk_biome_sampling,
                init_natural_wildlife_for_biomesampled_macrochunks,
            )
                .in_set(GameplaySystems)
                .in_set(HostSystems),
        )
    ;
}
