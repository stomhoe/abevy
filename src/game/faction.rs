use bevy_replicon::prelude::AppRuleExt;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::faction::{faction_resources::FactionEntityMap, faction_systems::*};
use crate::game::faction::faction_components::*;
//use crate::game::faction::factions_resources::*;
//use crate::game::faction::factions_events::*;
mod faction_systems;
pub mod faction_components;
pub mod faction_resources;
//mod factions_layout;
//mod factions_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FactionSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Factions)) del módulo parent_module_name !!
pub struct FactionPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (set_as_self_faction, ).in_set(FactionSystems))
            //.add_systems(Update, (set_as_self_faction, ).in_set(FactionsSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            .init_resource::<FactionEntityMap>()
            .add_plugins((
                superstate_plugin::<NonNeutralState, (AtWar, Truce, Ally)>,
            ),)
            .replicate::<Faction>()
            .replicate::<BelongsToFaction>()
            

        ;
    }
}