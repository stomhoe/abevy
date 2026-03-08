use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt};

#[allow(unused_imports, )]
use crate::prelude::*;



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {

    app
    .replicate::<Player>()
    .replicate::<HostPlayer>()

    .init_resource::<PlayerData>()


    ;
}
