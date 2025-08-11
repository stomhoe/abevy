#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetServerPlugin;
use common::{components::*, states::{AppState, ConnectionAttempt, GameSetupType, LocalAssetsLoadingState}};
use game::{faction_components::{BelongsToFaction, Faction}, player::{HostPlayer, Player}};
use multiplayer_shared::{multiplayer_events::MoveStateUpdated, multiplayer_shared::HostSystems};
use sprite_shared::{animation_shared::{AnimationSystems, MoveAnimActive }, sprite_shared::{Directionable, SpriteCfgEntityMap, SpriteConfigRef, SpriteHolderRef}};
use tilemap::{chunking_components::{ActivatingChunks, ProducedTiles}, tile::{tile_components::HashPosEntiWeightedSampler, tile_resources::*}};

use crate::host_systems::*;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_plugins((RepliconPlugins, RepliconRenetServerPlugin, ))
            
    .add_observer(host_on_player_connect)
    .add_observer(host_receive_client_name)
    

    .add_systems(OnExit(AppState::StatefulGameSession), (
        clean_resources
    ))
    .add_systems(
        OnEnter(ConnectionAttempt::Triggered),
        (
            (attempt_host,).in_set(HostSystems),
        ),
    )
     .add_systems(Update, ((

        update_animstate_for_clients

        ).in_set(SimRunningSystems).run_if(server_running),
    ))
    

 

    .add_server_trigger::<SpriteCfgEntityMap>(Channel::Unordered)
    .make_trigger_independent::<SpriteCfgEntityMap>()
    
    .add_mapped_server_trigger::<MoveStateUpdated>(Channel::Ordered)
    

    .replicate::<Name>()
    .replicate::<EntityPrefix>()
    .replicate::<DisplayName>()
    .replicate_with((
        (RuleFns::<ChildOf>::default(), SendRate::EveryTick),
        (RuleFns::<SpriteHolderRef>::default(), SendRate::EveryTick),
        (RuleFns::<SpriteConfigRef>::default(), SendRate::EveryTick),
    ))


    .replicate_bundle::<(Player,DisplayName)>()
    .replicate::<Player>()
    .replicate::<HostPlayer>()
    .replicate::<Faction>()
    .replicate::<BelongsToFaction>()
    .replicate::<Directionable>()
    .register_type::<SpriteCfgEntityMap>()
    .register_type::<SpriteHolderRef>()
    .replicate::<DisplayName>()
    .replicate::<HashId>()
    .replicate::<StrId>()
    .replicate::<EntityPrefix>()

    .add_mapped_server_trigger::<MoveStateUpdated>(Channel::Ordered)
    

    .replicate_with((
        (RuleFns::<MoveAnimActive>::default(), SendRate::Once),//NO TIENE Q SER FRECUENTE ESTE, ES RELIABLE. HACER OTRO NO RELIABLE
    ))
    
    .add_server_trigger::<TilingEntityMap>(Channel::Unordered)
    .make_trigger_independent::<TilingEntityMap>()
    .replicate::<HashPosEntiWeightedSampler>()

    .replicate_once::<ActivatingChunks>()
    .replicate::<ProducedTiles>()

    ;
}
