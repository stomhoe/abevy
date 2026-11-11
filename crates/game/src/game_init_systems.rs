
#[allow(unused_imports, )]use being::being_bundles::{BeingBundle, };
use ::common::*;
use ::sprite_shared::*;
use faction::{faction_resources::*};
use ::being_shared::*;
use faction_shared::Faction;
use game_common::game_common_components::Templ;
use player_shared::player_components::*;
use multiplayer_shared::multiplayer_events::*;
use bevy_replicon::prelude::*;
use being::class::ClassEntityMap;
use being::body::{BodyRef, body_sampler::body_sampler_resources::BodyWeightedSamplerRef};
use being::pet::PetSetEntityMap;
use tilemap::{
    terrain::{
        terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_resources::TerrProbeTemplEntityMap},
    },
};

use bevy::prelude::*;
use serde::Deserialize;
use modifier_shared::WallPhaser;
use tilemap::{
    run_oneshot_suitable_pos_search_logic,
    terrain::{
        terrprobe::terrprobe_messages::TerrProbeJob,
        terrprobe::terrprobe_systems::SearchParams,
    },
};
use ::tilemap_shared::*;

#[derive(Debug, Event, Copy, Clone)]
pub struct CommonSpawnOriginFound {
    pub dim_ref: DimensionRef,
    pub pos: GlobalTilePos,
}

#[derive(Resource, Debug, Default, Copy, Clone)]
pub struct CommonSpawnOriginCache(pub Option<CommonSpawnOriginFound>);

#[derive(Resource, Clone, Debug)]
pub struct GameInitSettings {
    pub players_spawn_probe_id: StrId,
}
impl Default for GameInitSettings {
    fn default() -> Self {
        Self {
            players_spawn_probe_id: StrId::new_with_result("land", 0).expect("default players_spawn_probe_id must be a valid StrId"),
        }
    }
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct GameInitSettingsSeri {
    pub id: String,
    #[serde(default = "default_players_spawn_probe_id")]
    pub players_spawn_probe_id: String,
}
impl GameInitSettingsSeri {
    pub fn to_settings(&self) -> GameInitSettings {
        let players_spawn_probe_id = match StrId::new_with_result(self.players_spawn_probe_id.trim(), 0) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: GAME_INIT,
                    "Invalid players_spawn_probe_id '{}' in GameInitSettingsSeri '{}': {}. Falling back to 'land'",
                    self.players_spawn_probe_id,
                    self.id,
                    err,
                );
                StrId::new_with_result("land", 0).expect("fallback players_spawn_probe_id must be a valid StrId")
            }
        };
        GameInitSettings {
            players_spawn_probe_id,
        }
    }
}
fn default_players_spawn_probe_id() -> String { "land".to_string() }

fn place_player_being_at_position(
    cmd: &mut Commands,
    being_ent: Entity,
    spawn_dim: DimensionRef,
    gpos: GlobalTilePos,
) {
    cmd.entity(being_ent).try_insert((
        Transform::from_translation(gpos.to_translation(0.0)),
        gpos,
        GridLockedMovement::default(),
        GridLockedMovementVisual {
            visual_origin_tile: gpos.0,
            ..default()
        },
        DimensionRef(spawn_dim.0),
    ));
    debug!(target: GAME_INIT, "Placed player being {:?} in dimension {} at {:?}", being_ent, spawn_dim.0, gpos);
}

pub fn load_game_init_settings(
    mut settings: ResMut<GameInitSettings>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
) {
    let db = match DefDatabase::<GameInitSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(GameInitSettingsSeri),
        &["game_init.settings.ron"],
        |_| "game_init_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(target: GAME_INIT, "Failed loading GameInitSettingsSeri defs: {err:#}");
            return;
        }
    };
    let Some(first) = db.into_records().into_iter().next() else {
        return;
    };
    let mut loaded_settings = first.value.to_settings();
    let requested_probe_id = loaded_settings.players_spawn_probe_id.clone();
    if terrprobe_entity_map.0.get_cloned(requested_probe_id.clone()).is_err() {
        error!(
            target: GAME_INIT,
            "Configured players_spawn_probe_id '{}' in GameInitSettingsSeri '{}' does not match any loaded terrain probe. Falling back to 'land'",
            requested_probe_id,
            first.value.id,
        );
        let fallback_probe_id = StrId::new_with_result("land", 0)
            .expect("fallback players_spawn_probe_id must be a valid StrId");
        if terrprobe_entity_map.0.get_cloned(fallback_probe_id.clone()).is_err() {
            error!(
                target: GAME_INIT,
                "Fallback terrain probe 'land' is also missing after loading GameInitSettingsSeri '{}'; keeping default GameInitSettings",
                first.value.id,
            );
            return;
        }
        loaded_settings.players_spawn_probe_id = fallback_probe_id;
    }
    *settings = loaded_settings;
}


#[allow(unused_parens, )]
pub fn server_or_singleplayer_setup(mut cmd: Commands,
    mut map: ResMut<FactionEntityMap>,
    mut settings: Query<&mut GlobalGenSettings>,
    mut app_state: ResMut<NextState<AppState>>,
)
{
    let Ok(mut _____settings) = settings.single_mut()
    else {
        error!(target: GAME_INIT, "Failed to get AaGlobalGenSettings");
        return;
    };



    let host_faction_id = match StrId::new_with_result("host", 0) {
        Ok(host_faction_id) => host_faction_id,
        Err(err) => {
            error!(target: GAME_INIT, "Invalid hardcoded host faction id 'host': {}", err);
            return;
        }
    };
    let host_faction_hash = HashId::from(host_faction_id.as_str());
    let host_faction = cmd.spawn((Faction, host_faction_id.clone(), host_faction_hash, Mine)).id();

    map.0.overwrite(host_faction_id, host_faction);


    let host_player_id = match StrId::new_with_result("HOOOOOST", 0) {
        Ok(host_player_id) => host_player_id,
        Err(err) => {
            error!(target: GAME_INIT, "Invalid hardcoded host player id 'HOOOOOST': {}", err);
            return;
        }
    };
    cmd.spawn((
        Mine, HostPlayer,
        host_player_id,
        FactionRef(host_faction_hash),
    ));
    app_state.set(AppState::StatefulGameSession);
    cmd.trigger(InitDone);
}

#[allow(unused_parens)]
//TODO chequear el estado actual de la partida (new game o loaded (cargar su character si ya tiene)) y los Res<State<GamePhase>> antes de hacer esto

pub fn host_on_player_added(mut cmd: Commands,
    query: Query<(Entity, &StrId, Has<HostPlayer>, Option<&CharacterSelection>), (With<Player>, Or<(Added<StrId>, Added<Player>)>, )>,
    player_query: Query<&CreatedCharacters>,
    wall_phaser_on_spawn: Res<::being_shared::WallPhaserOnSpawn>,
    invulnerable_on_spawn: Res<::being_shared::InvulnerableOnSpawn>,
    host_player_faction_ref: Query<&FactionRef, (With<HostPlayer>, With<Mine>, )>,
    host_faction_query: Query<Entity, (With<Faction>, With<Mine>, Without<Templ>, )>,
    faction_map: Res<FactionEntityMap>,
) {
    if query.is_empty() {
        return;
    }

    let host_faction = host_player_faction_ref
        .single()
        .ok()
        .and_then(|host_faction_ref| faction_map.0.get_cloned(host_faction_ref.0).ok())
        .or_else(|| host_faction_query.iter().next());
    let Some(host_faction) = host_faction else {
        error!(target: GAME_INIT, "Failed to get host faction: no HostPlayer FactionRef could be resolved and no fallback Faction+Mine entity was found");
        return;
    };
    for (player_ent, username, is_host_player, selection) in query.iter() {

        if player_query.get(player_ent).is_err() {

            //USAR EL DEFAULT ASE Q SE DESPAWNEE

            let created_character = cmd.spawn((Being, username.clone(),
                CharacterCreatedBy { player: player_ent },
                JoinedGroups::single(host_faction),
                ComputedBy {
                    client_ent: player_ent,
                    human_dc_input: true,
                },
            )).id();
            if let Some(selection) = selection.filter(|selection| !selection.sprites.is_empty()) {
                cmd.entity(created_character).insert((
                    RaceRef(selection.race),
                    ClassRef(selection.class),
                    ScsToBuild(selection.sprites.iter().map(|(_, sprite)| *sprite).collect()),
                ));
            } else {
                debug!(target: GAME_INIT, "Player '{}' has no accepted character selection; its being is waiting for race, class and sprite choices", username);
            }
            if wall_phaser_on_spawn.0 && is_host_player {
                cmd.entity(created_character).insert(WallPhaser);
            }
            if invulnerable_on_spawn.0 && is_host_player {
                cmd.entity(created_character).insert(::modifier_shared::Invulnerable);
            }
            //cmd.spawn(SpeedModifier::new(created_character, created_character, 1000.0, ApplyMode::Add));

        }else{
            //TODO ASIGNARLE SU CHARACTER SI TIENE EL MISMO OWNER
        }
    }
}

#[allow(unused_parens, )]
pub fn apply_character_selection(
    mut cmd: Commands,
    selections: Query<(Entity, &CharacterSelection), (Changed<CharacterSelection>, With<Player>, )>,
    beings: Query<(Entity, &CharacterCreatedBy, Option<&RaceRef>, Option<&ScsToBuild>, Option<&HeldSprites>, Option<&HeldBody>, Has<SampleSpritesamplers>, ), (With<Being>, )>,
) {
    for (player, selection) in &selections {
        if selection.sprites.is_empty() {
            continue;
        }
        let race = selection.race;
        let selected_sprites = ScsToBuild(selection.sprites.iter().map(|(_, sprite)| *sprite).collect());
        for (being, owner, current_race, current_sprites, held_sprites, held_body, has_sample_sprites) in &beings {
            if owner.player == player {
                cmd.entity(being).insert(ClassRef(selection.class));
                let race_changed = current_race.is_none_or(|current| current.0 != race);
                if race_changed {
                    cmd.entity(being).insert(RaceRef(race));
                    if let Some(held_body) = held_body {
                        let body = held_body.entity();
                        cmd.entity(body).despawn_related::<Children>();
                        cmd.entity(body).try_despawn();
                    }
                    cmd.entity(being).try_remove::<(BodyRef, BodyWeightedSamplerRef)>();
                }
                let sprites_changed = current_sprites.is_none_or(|current| current.0 != selected_sprites.0);
                if sprites_changed {
                    debug!(target: GAME_INIT, "Applying character selection to being {:?}: {} explicit sprite configs", being, selected_sprites.0.len());
                    if let Some(held_sprites) = held_sprites {
                        for sprite in held_sprites.iter() {
                            cmd.entity(sprite).despawn_related::<Children>();
                            cmd.entity(sprite).try_despawn();
                        }
                    }
                    cmd.entity(being).insert(selected_sprites.clone());
                }
                if has_sample_sprites {
                    cmd.entity(being).try_remove::<SampleSpritesamplers>();
                }
            }
        }
    }
}

#[allow(unused_parens, )]
pub fn receive_character_selection(
    mut requests: MessageReader<FromClient<CharacterCreationRequest>>,
    mut cmd: Commands,
    race_map: Res<RaceEntityMap>,
    class_map: Res<ClassEntityMap>,
    races: Query<(&SetsOfPlayerMonoChoosableSprites, Has<PlayerSelectableRace>, &TagSet), (With<Race>, )>,
    classes: Query<(&ClassData, &ClassRequirements, Has<PlayerSelectableClass>), (With<Class>, )>,
    sprite_ids: Query<&HashId, ()>,
    pet_sets: Query<&PetSetData, (With<PetSet>, )>,
    pet_set_map: Res<PetSetEntityMap>,
    mut rejected: MessageWriter<ToClients<CharacterCreationRejected>>,
    mut accepted: MessageWriter<ToClients<CharacterCreationAccepted>>,
    host_player: Query<(Entity, ), (With<HostPlayer>, With<Player>, )>,
    mut rejections: Local<Vec<ToClients<CharacterCreationRejected>>>,
    mut acceptances: Local<Vec<ToClients<CharacterCreationAccepted>>>,
) {
    'requests: for request in requests.read() {
        let player = match request.client_id {
            ClientId::Client(player) => player,
            ClientId::Server => {
                let Ok((player, )) = host_player.single() else {
                    error!(target: GAME_INIT, "Cannot apply local character selection: expected one host player");
                    continue 'requests;
                };
                player
            }
        };
        let selection = &request.message;
        let Ok(race_entity) = race_map.0.get_cloned(selection.race) else {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::UnknownRace) });
            continue 'requests;
        };
        let Ok((groups, race_selectable, race_tags)) = races.get(race_entity) else {
            error!(target: GAME_INIT, "Cannot validate character selection: race {} is missing sprite groups or tags", selection.race);
            continue 'requests;
        };
        if !race_selectable {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::RaceNotSelectable) });
            continue 'requests;
        }
        let Ok(class_entity) = class_map.0.get_cloned(selection.class) else {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::UnknownClass) });
            continue 'requests;
        };
        let Ok((class_data, requirements, class_selectable)) = classes.get(class_entity) else {
            error!(target: GAME_INIT, "Cannot validate character selection: class {} is missing class data or requirements", selection.class);
            continue 'requests;
        };
        if !class_selectable {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::ClassNotSelectable) });
            continue 'requests;
        }
        if !requirements.allowed_race_tags.is_empty()
            && !requirements.allowed_race_tags.iter().any(|allowed| race_tags.iter().any(|tag| HashId::from(tag.as_ref()) == *allowed))
        {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::IncompatibleRace) });
            continue 'requests;
        }
        let mut selected_groups = Vec::with_capacity(selection.sprites.len());
        for (group_id, sprite_id) in &selection.sprites {
            if selected_groups.contains(group_id) {
                rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidSpriteSelection) });
                continue 'requests;
            }
            selected_groups.push(*group_id);
            let Some((_, group_sprites)) = groups.0.iter().find(|(group, _)| group.hash_id() == *group_id) else {
                rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidSpriteSelection) });
                continue 'requests;
            };
            if !group_sprites.iter().any(|sprite_entity| sprite_ids.get(*sprite_entity).is_ok_and(|id| *id == *sprite_id)) {
                rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidSpriteSelection) });
                continue 'requests;
            }
        }
        if selected_groups.len() != groups.0.len() {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidSpriteSelection) });
            continue 'requests;
        }
        if let Some(pet_set_id) = class_data.pet_set {
            let Ok(pet_set_entity) = pet_set_map.0.get_cloned(pet_set_id) else {
                rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidPetSelection) });
                continue 'requests;
            };
            let Ok(pet_set) = pet_sets.get(pet_set_entity) else {
                error!(target: GAME_INIT, "Cannot validate character selection: pet set {} is missing PetSetData", pet_set_id);
                continue 'requests;
            };
            let mut total_cost = 0u32;
            let mut per_race = bevy::platform::collections::HashMap::<HashId, u32>::default();
            for pet_id in &selection.pets {
                let Some(pet) = pet_set.pets.iter().find(|pet| pet.id == *pet_id) else {
                    rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidPetSelection) });
                    continue 'requests;
                };
                total_cost = total_cost.saturating_add(pet.cost);
                *per_race.entry(pet.race_id).or_default() += 1;
            }
            if selection.pets.len() as u32 > pet_set.max_selected || total_cost > pet_set.budget
                || per_race.iter().any(|(race, count)| pet_set.per_race_max.get(race).is_some_and(|max| count > max))
            {
                rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidPetSelection) });
                continue 'requests;
            }
        } else if !selection.pets.is_empty() {
            rejections.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationRejected(CharacterCreationRejection::InvalidPetSelection) });
            continue 'requests;
        }
        debug!(target: GAME_INIT, "Accepted character selection for player {:?}: race {}, class {}, {} sprites", player, selection.race, selection.class, selection.sprites.len());
        let accepted_selection = selection.clone();
        cmd.entity(player).insert(CharacterSelection {
            race: selection.race,
            class: selection.class,
            sprites: selection.sprites.clone(),
            pets: selection.pets.clone(),
        });
        cmd.entity(player).insert(ClassRef(selection.class));
        acceptances.push(ToClients { targets: SendTargets::Single(request.client_id), message: CharacterCreationAccepted(accepted_selection) });
    }
    for rejection in rejections.iter() {
        error!(target: GAME_INIT, "Rejected character selection from {:?}: {:?}", rejection.targets, rejection.message.0);
    }
    rejected.write_batch(rejections.drain(..));
    accepted.write_batch(acceptances.drain(..));
}

#[allow(unused_parens, )]
pub fn find_common_player_spawn_origin(
    mut cmd: Commands,
    dimension_entity_map: Res<DimensionEntityMap>,
    dimension_hash_query: Query<&HashId, With<Dimension>>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl>,
    mut search_params: SearchParams,
    mut active_probe_ent: Local<Option<Entity>>,
    mut search_finished: Local<bool>,
    settings: Res<GameInitSettings>,
) {
    let make_search_request = |_cmd: &mut Commands| -> Option<TerrProbeJob> {
    let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            return None;
        };
        let Ok(&ow_dimension_hash) = dimension_hash_query.get(ow_dimension) else {
            return None;
        };

        let Ok(probe_template_ent) = terrprobe_entity_map.0.get_cloned(settings.players_spawn_probe_id.clone()) else {
            error_once!(
                target: GAME_INIT,
                "Failed to find TerrProbe template for players_spawn_probe_id '{}'; no common player spawn origin can be found",
                settings.players_spawn_probe_id.as_str(),
            );
            return None;
        };
        let Ok(probe_template) = terrprobe_query.get(probe_template_ent) else {
            return None;
        };
        Some(probe_template.to_probe(probe_template_ent, DimensionRef(ow_dimension_hash), GlobalTilePos::default()))
    };
    let handle_success = |cmd: &mut Commands,
                              found_pos: GlobalTilePos,
                              _requester: Entity,
                              _sampled_val: f32|
     -> bool {
        let Ok(ow_dimension) = dimension_entity_map.0.get_cloned(Dimension::overworld()) else {
            return false;
        };
        let Ok(&ow_dimension_hash) = dimension_hash_query.get(ow_dimension) else {
            return false;
        };
        cmd.trigger(CommonSpawnOriginFound {
            dim_ref: DimensionRef(ow_dimension_hash),
            pos: found_pos,
        });
        true
    };

    let handle_failure = |_cmd: &mut Commands, _failed_probe_ent: Entity| {
    };

    run_oneshot_suitable_pos_search_logic!(
        target: GAME_INIT,
        searched_label: "common character spawn origin",
        cmd: cmd,
        search_params: search_params,
        active_probe_ent: active_probe_ent,
        search_finished: search_finished,
        make_search_request: make_search_request,
        handle_success: handle_success,
        handle_failure: handle_failure,
    );
}

#[allow(unused_parens, )]
pub fn put_player_beings_on_map(
    trigger: On<CommonSpawnOriginFound>,
    mut cmd: Commands,
    beings_to_place: Query<(Entity, &CharacterCreatedBy, ), (With<Being>, Without<DimensionRef>, )>,
    mut next_spawn_offset_x: Local<i32>,
    mut spawn_cache: ResMut<CommonSpawnOriginCache>,
) {
    let found = trigger.event();
    spawn_cache.0 = Some(*found);
    let spawn_dim = found.dim_ref;
    let origin = found.pos;

    let compute_spawn_pos = |origin: GlobalTilePos, next_x: &mut i32| -> GlobalTilePos {
        let spawn_pos = origin + GlobalTilePos::new(*next_x, 0);
        *next_x += 1;
        spawn_pos
    };

    for (being_ent, _, ) in beings_to_place.iter() {
        let gpos = compute_spawn_pos(origin, &mut *next_spawn_offset_x);
        place_player_being_at_position(&mut cmd, being_ent, spawn_dim, gpos);
    }
}

#[allow(unused_parens, )]
pub fn place_unpositioned_player_beings_with_cached_origin(
    mut cmd: Commands,
    beings_to_place: Query<(Entity, &CharacterCreatedBy, ), (With<Being>, Without<DimensionRef>, )>,
    spawn_cache: Res<CommonSpawnOriginCache>,
    mut next_spawn_offset_x: Local<i32>,
) {
    let Some(found) = spawn_cache.0
    else {
        return;
    };
    let compute_spawn_pos = |origin: GlobalTilePos, next_x: &mut i32| -> GlobalTilePos {
        let spawn_pos = origin + GlobalTilePos::new(*next_x, 0);
        *next_x += 1;
        spawn_pos
    };
    for (being_ent, _, ) in beings_to_place.iter() {
        let gpos = compute_spawn_pos(found.pos, &mut *next_spawn_offset_x);
        place_player_being_at_position(&mut cmd, being_ent, found.dim_ref, gpos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_spawn_places_player_without_character_selection() {
        let mut app = App::new();
        let origin = GlobalTilePos::new(7, 9);
        let dimension = DimensionRef(HashId::from("test_dimension"));
        app.insert_resource(CommonSpawnOriginCache(Some(CommonSpawnOriginFound { dim_ref: dimension, pos: origin })))
            .add_systems(Update, place_unpositioned_player_beings_with_cached_origin);
        let player = app.world_mut().spawn(HostPlayer).id();
        let being = app.world_mut().spawn((Being, CharacterCreatedBy { player })).id();

        app.update();

        assert_eq!(app.world().get::<GlobalTilePos>(being), Some(&origin));
        assert_eq!(app.world().get::<DimensionRef>(being), Some(&dimension));
        assert!(!app.world().entity(being).contains::<Unloaded>());
        assert!(!app.world().entity(being).contains::<BitRef>());
    }

    #[test]
    fn host_message_applies_selection_and_invalid_update_preserves_it() {
        let mut app = App::new();
        app.init_resource::<RaceEntityMap>()
            .init_resource::<ClassEntityMap>()
            .init_resource::<PetSetEntityMap>()
            .add_message::<FromClient<CharacterCreationRequest>>()
            .add_message::<ToClients<CharacterCreationAccepted>>()
            .add_message::<ToClients<CharacterCreationRejected>>()
            .add_systems(Update, (receive_character_selection, apply_character_selection).chain());
        let player = app.world_mut().spawn(HostPlayer).id();
        let being = app.world_mut().spawn((Being, CharacterCreatedBy { player })).id();
        let head_id = HashId::from("selected_head");
        let head = app.world_mut().spawn(head_id).id();
        let race_id = HashId::from("test_race");
        let class_id = HashId::from("test_class");
        let group = StrId::trunc("Head");
        let race = app.world_mut().spawn((Race, PlayerSelectableRace, TagSet::default(), SetsOfPlayerMonoChoosableSprites(vec![(group.clone(), [head].into_iter().collect())]))).id();
        let class = app.world_mut().spawn((Class, PlayerSelectableClass, ClassData::default(), ClassRequirements::default())).id();
        app.world_mut().resource_mut::<RaceEntityMap>().0.overwrite(race_id, race);
        app.world_mut().resource_mut::<ClassEntityMap>().0.overwrite(class_id, class);
        let request = CharacterCreationRequest { race: race_id, class: class_id, sprites: vec![(group.hash_id(), head_id)], pets: vec![] };
        app.world_mut().write_message(FromClient { client_id: ClientId::Server, message: request.clone() });

        app.update();

        assert_eq!(app.world().get::<CharacterSelection>(player).unwrap().race, race_id);
        assert_eq!(app.world().get::<RaceRef>(being).unwrap().0, race_id);
        assert_eq!(app.world().get::<ClassRef>(being).unwrap().0, class_id);
        assert!(app.world().get::<ScsToBuild>(being).unwrap().0.contains(&head_id));
        let mut invalid = request;
        invalid.sprites[0].1 = HashId::from("unavailable_head");
        app.world_mut().write_message(FromClient { client_id: ClientId::Server, message: invalid });

        app.update();

        assert_eq!(app.world().get::<CharacterSelection>(player).unwrap().sprites[0].1, head_id);
        assert_eq!(app.world().resource::<Messages<ToClients<CharacterCreationRejected>>>().len(), 1);

        let next_head_id = HashId::from("next_head");
        let next_head = app.world_mut().spawn(next_head_id).id();
        let next_race_id = HashId::from("next_race");
        let next_race = app.world_mut().spawn((Race, PlayerSelectableRace, TagSet::default(), SetsOfPlayerMonoChoosableSprites(vec![(group.clone(), [next_head].into_iter().collect())]))).id();
        app.world_mut().resource_mut::<RaceEntityMap>().0.overwrite(next_race_id, next_race);
        app.world_mut().entity_mut(being).insert(BodyRef(HashId::from("old_body")));
        let old_body = app.world_mut().spawn((BodyOf { being }, ChildOf(being))).id();
        let old_part = app.world_mut().spawn(ChildOf(old_body)).id();
        app.world_mut().write_message(FromClient {
            client_id: ClientId::Server,
            message: CharacterCreationRequest { race: next_race_id, class: class_id, sprites: vec![(group.hash_id(), next_head_id)], pets: vec![] },
        });

        app.update();

        assert_eq!(app.world().get::<RaceRef>(being).unwrap().0, next_race_id);
        assert_eq!(app.world().get::<ScsToBuild>(being).unwrap().0, [next_head_id].into_iter().collect());
        assert!(!app.world().entity(being).contains::<BodyRef>());
        assert!(app.world().get_entity(old_body).is_err());
        assert!(app.world().get_entity(old_part).is_err());
    }
}
