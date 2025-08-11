#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
#[allow(unused_imports)]
use superstate::superstate_plugin;

use bevy_asset_loader::prelude::*;





pub struct GamePlugin;

#[allow(unused_parens, )]
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {

        app
       
            // .add_systems(OnEnter(AppState::StatefulGameSession), (
            //     (server_or_singleplayer_setup,).run_if(server_or_singleplayer),
            // ))
            
            // .add_systems(OnEnter(GamePhase::ActiveGame), (
            //     (spawn_player_beings,).run_if(server_or_singleplayer),
            // ))
            // .add_systems(OnEnter(AppState::PreGame), (
            //     remove_server_resources,
            // ))

            .add_systems(Update, (
                //update_img_sizes_on_load,
                //host_on_player_added.run_if(server_or_singleplayer),
                // (
                //     toggle_simulation, update_transform_z, tick_time_based_multipliers,
                
                // ).in_set(GameplaySystems),
                // set_as_self_faction,
            ))

            // .add_systems(Update, (pass_time, reduce_remaining_days, ).in_set(ClockSystems).in_set(SimRunningSystems))


            .configure_sets(Update, (
                // PlayerInputSystems,
                // MovementSystems,

                // GameplaySystems.run_if(in_state(GamePhase::ActiveGame).and(in_state(ReplicatedAssetsLoadingState::Finished))),
                // SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
                // SimPausedSystems.run_if(in_state(SimulationState::Paused).and(in_state(GamePhase::ActiveGame))),

                //ClientSystems.run_if(not(server_or_singleplayer).or(in_state(GameSetupType::JoinerLobby))),

                //ChunkSystems.in_set(GameplaySystems).in_set(SimRunningSystems)
            ))
            
            // .configure_sets(FixedUpdate, (
            //     ClientSystems.run_if(not(server_or_singleplayer).or(in_state(GameSetupType::AsJoiner))),//NO TOCAR
            //     SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
            // ))
            // .configure_sets(OnEnter(ReplicatedAssetsLoadingState::Finished), (
            //     SpriteSystemsSet.before(RaceSystemsSet),
            // ))



           
            
            //https://github.com/NiklasEi/bevy_asset_loader?tab=readme-ov-file
            //https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/finally_init_resource.rs

            //.register_type::<MyType>()

        ;
    }
}

