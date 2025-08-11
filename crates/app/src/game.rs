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
            .add_plugins((SetupMenusPlugin, PlayerPlugin, BeingsPlugin, 
                FactionPlugin, DimensionPlugin, MyTileMapPlugin, ClockPlugin, ))

            .add_systems(OnEnter(AppState::StatefulGameSession), (
                (server_or_singleplayer_setup,).run_if(server_or_singleplayer),
            ))
            
            .add_systems(OnEnter(GamePhase::ActiveGame), (
                (spawn_player_beings,).run_if(server_or_singleplayer),
            ))
            .add_systems(OnEnter(AppState::PreGame), (
                remove_server_resources,
            ))

            .add_systems(Update, (
                update_img_sizes_on_load,
                host_on_player_added.run_if(server_or_singleplayer),
                (
                    toggle_simulation, update_transform_z, tick_time_based_multipliers,
                
                ).in_set(GameplaySystems),
                set_as_self_faction,
            ))

            .add_systems(Update, (pass_time, reduce_remaining_days, ).in_set(ClockSystems).in_set(SimRunningSystems))


            .configure_sets(Update, (
                PlayerInputSystems,
                MovementSystems,

                GameplaySystems.run_if(in_state(GamePhase::ActiveGame).and(in_state(ReplicatedAssetsLoadingState::Finished))),
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
                SimPausedSystems.run_if(in_state(SimulationState::Paused).and(in_state(GamePhase::ActiveGame))),

                ClientSystems.run_if(not(server_or_singleplayer).or(in_state(GameSetupType::JoinerLobby))),

                ChunkSystems.in_set(GameplaySystems).in_set(SimRunningSystems)
            ))
            
            .configure_sets(FixedUpdate, (
                ClientSystems.run_if(not(server_or_singleplayer).or(in_state(GameSetupType::JoinerLobby))),//NO TOCAR
                SimRunningSystems.run_if(in_state(SimulationState::Running).and(in_state(GamePhase::ActiveGame))),
            ))
            .configure_sets(OnEnter(ReplicatedAssetsLoadingState::Finished), (
                SpriteSystemsSet.before(RaceSystemsSet),
            ))



            .add_loading_state(
                LoadingState::new(LocalAssetsLoadingState::InProcess).continue_to_state(LocalAssetsLoadingState::Finished)
                .load_collection::<SpriteSerisHandles>()
                .load_collection::<AnimSerisHandles>()
                .load_collection::<RaceSerisHandles>()
                .load_collection::<ShaderRepeatTexSerisHandles>()
                .load_collection::<TileSerisHandles>()
                .finally_init_resource::<SpriteCfgEntityMap>()
                .finally_init_resource::<TilingEntityMap>()
                .finally_init_resource::<TileShaderEntityMap>()
            )
            .add_loading_state(
                LoadingState::new(ReplicatedAssetsLoadingState::InProcess).continue_to_state(ReplicatedAssetsLoadingState::Finished)
                .load_collection::<NoiseSerisHandles>()
                .load_collection::<OpListSerisHandles>()
                //.load_collection::<DimensionSerisHandles>()
                .load_collection::<TileWeightedSamplerSerisHandles>()
                .finally_init_resource::<TerrGenEntityMap>()
                .finally_init_resource::<OpListEntityMap>()
                .finally_init_resource::<DimensionEntityMap>()
            )
            
            //https://github.com/NiklasEi/bevy_asset_loader?tab=readme-ov-file
            //https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/finally_init_resource.rs

            .replicate_bundle::<(Being, ChildOf)>()//NO FUNCIONA BIEN LO DE CHILDOF
            .replicate::<FacingDirection>()//PROVISORIO, VA A HABER Q REVISAR
            //.register_type::<MyType>()

        ;
    }
}

