

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::{prelude::*, shared::RepliconSharedPlugin};
use common::{common_components::*, common_states::*};
use game::{being_components::*, faction_components::*, movement_components::*, player::*};
use game_common::game_common_components::{BeingAltitude, Directionable, FacingDirection};
use crate::{multiplayer_events::*, multiplayer_shared_systems::*};
use sprite_shared::{animation_shared::MoveAnimActive , sprite_shared::*};
use tilemap::{chunking_components::{ActivatingChunks, ProducedTiles}, terrain_gen::terrgen_components::*, tile::{tile_components::*, tile_resources::*}};


pub const PROTOCOL_ID: u64 = 7;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct HostSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((RepliconSharedPlugin::default()))
  

    .configure_sets(OnEnter(ConnectionAttempt::Triggered), (
        HostSystems.run_if(in_state(GameSetupType::AsHost)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner)),
    ))
    .configure_sets(OnEnter(AppState::StatefulGameSession), (
        HostSystems.run_if(in_state(GameSetupType::AsHost)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner)),
    ))
    .configure_sets(OnExit(AppState::StatefulGameSession), (
        HostSystems.run_if(in_state(GameSetupType::AsHost)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner)),
    ))
    .configure_sets(Update, (
        HostSystems.run_if(in_state(GameSetupType::AsHost).or(server_running)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner).or(not(server_or_singleplayer))),
    ))
    .configure_sets(FixedUpdate, (
        HostSystems.run_if(in_state(GameSetupType::AsHost).or(server_running)),
        ClientSystems.run_if(in_state(GameSetupType::AsJoiner).or(not(server_or_singleplayer))),
    ))
    .add_systems(OnExit(AppState::StatefulGameSession), (
        all_clean_resources
    ))

    .add_server_trigger::<HostStartedGame>(Channel::Unordered)
    
    .add_server_trigger::<SpriteCfgEntityMap>(Channel::Unordered)
    .make_trigger_independent::<SpriteCfgEntityMap>()
    
    .add_mapped_server_trigger::<MoveStateUpdated>(Channel::Unordered)
    
    
    .replicate::<Name>()
    .replicate::<EntityPrefix>().replicate::<StrId>().replicate::<HashId>()
    .replicate::<DisplayName>()
    
    
    .replicate_with((
        (RuleFns::<ChildOf>::default(), SendRate::EveryTick),
        (RuleFns::<SpriteHolderRef>::default(), SendRate::EveryTick),
        (RuleFns::<SpriteConfigRef>::default(), SendRate::EveryTick),
    ))
    
    .replicate::<Player>()
    .replicate::<HostPlayer>()
    .replicate::<CharacterCreatedBy>()
    .replicate::<HumanControlled>()


    .replicate::<Faction>()
    .replicate::<BelongsToFaction>()
    
    .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Periodic((64*3))),
    ))
    
    .replicate::<FacingDirection>()
    .replicate::<Directionable>()
    
    .replicate::<BeingAltitude>()
    .replicate::<Being>()
    .replicate::<ControlledBy>()


    .replicate_with((
        (RuleFns::<MoveAnimActive>::default(), SendRate::Once),//NO TIENE Q SER FRECUENTE ESTE, ES RELIABLE. HACER OTRO NO RELIABLE
    ))
    
    .replicate::<HashPosEntiWeightedSampler>()
    


    .replicate_once::<ActivatingChunks>()
    .replicate::<ProducedTiles>()
    
    .replicate::<FnlNoise>()
    .replicate::<RootOpList>()
    .replicate_with((
        RuleFns::<ProducedTiles>::default(),
        (RuleFns::<OperationList>::default(), SendRate::Once),
    ))
    
    .add_server_trigger::<TilingEntityMap>(Channel::Unordered)
    .make_trigger_independent::<TilingEntityMap>()


    .add_client_trigger::<SendUsername>(Channel::Ordered)



    ;
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Once),
    ))
*/