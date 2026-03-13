#[macro_export]
macro_rules! define_input_actions {
    () => {};
    ($name:ident => $output:ty $(, $rest_name:ident => $rest_output:ty)* $(,)?) => {
        #[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
        #[action_output($output)]
        pub struct $name;

        $crate::define_input_actions!($($rest_name => $rest_output),*);
    };
}

#[macro_export]
macro_rules! define_player_action_request_module {
    (
        base: $base:ident,
        extra_query: $extra_query_ty:ty,
        extra_binding: $extra_binding:pat,
        log_target: $log_target:expr
        $(,)?
    ) => {
        $crate::define_player_action_request_module!(
            base: $base,
            extra_query: $extra_query_ty,
            extra_binding: $extra_binding,
            log_target: $log_target,
            continuous: false,
        );
    };
    (
        base: $base:ident,
        extra_query: $extra_query_ty:ty,
        extra_binding: $extra_binding:pat,
        log_target: $log_target:expr,
        continuous: $continuous:expr
        $(,)?
    ) => {
        $crate::paste::paste! {
            #[derive(Debug, bevy_enhanced_input::prelude::InputAction)]
            #[action_output(bool)]
            pub struct [<Dc $base Action>];

            #[derive(bevy::prelude::Message, Clone, Copy)]
            pub struct [<Local $base Request>] {
                pub being_ent: bevy::prelude::Entity,
            }

            #[derive(serde::Deserialize, bevy::prelude::Message, serde::Serialize, Clone, bevy::ecs::entity::MapEntities)]
            pub struct [<Client $base Request>] {
                #[entities]
                pub being_ent: bevy::prelude::Entity,
            }

            pub fn [<trigger_ $base:snake _requests_from_player_input>](
                action_query: bevy::prelude::Query<
                    (
                        bevy::prelude::Ref<bevy_enhanced_input::prelude::Action<[<Dc $base Action>]>>,
                        &bevy_enhanced_input::prelude::ActionEvents,
                    ),
                >,
                player_query: bevy::prelude::Query<
                    (
                        &bevy_enhanced_input::prelude::Actions<crate::ac_input_contexts::BeingDirectControlInputContext>,
                        &being_shared::ComputedBeings,
                    ),
                    (
                        bevy::prelude::With<player::player_components::Mine>,
                        bevy::prelude::With<player::player_components::Player>,
                    ),
                >,
                controlled_beings: bevy::prelude::Query<(&being_shared::ComputedBy, $extra_query_ty)>,
                mut writer: bevy::prelude::MessageWriter<[<Local $base Request>]>,
                mut local_msgs_to_write_out: bevy::prelude::Local<Vec<[<Local $base Request>]>>,
            ) {
                for (actions, computed_beings) in player_query.iter() {
                    let Some((action, action_events)) = action_query.iter_many(actions).next() else {
                        continue;
                    };
                    let should_send = if $continuous {
                        **action
                    } else {
                        action_events.contains(bevy_enhanced_input::prelude::ActionEvents::START)
                    };
                    if !should_send {
                        continue;
                    }
                    for &being_ent in computed_beings.being_ents() {
                        let Ok((controlled_by, $extra_binding)) = controlled_beings.get(being_ent) else {
                            continue;
                        };
                        if !controlled_by.human_dc_input {
                            continue;
                        }
                        local_msgs_to_write_out.push([<Local $base Request>] { being_ent });
                    }
                }
                writer.write_batch(local_msgs_to_write_out.drain(..));
            }

            pub fn [<send_ $base:snake _request_to_server>](
                mut client_request_writer: bevy::prelude::MessageWriter<[<Client $base Request>]>,
                mut local_requests: bevy::prelude::MessageReader<[<Local $base Request>]>,
                beings: bevy::prelude::Query<(), bevy::prelude::With<being_shared::ComputedLocally>>,
                mut local_msgs_to_write_out: bevy::prelude::Local<Vec<[<Client $base Request>]>>,
            ) {
                for request in local_requests.read() {
                    let being_ent = request.being_ent;
                    let Ok(()) = beings.get(being_ent) else {
                        continue;
                    };
                    local_msgs_to_write_out.push([<Client $base Request>] { being_ent });
                }
                client_request_writer.write_batch(local_msgs_to_write_out.drain(..));
            }

            pub fn [<receive_ $base:snake _from_client>](
                mut events: bevy::prelude::MessageReader<bevy_replicon::prelude::FromClient<[<Client $base Request>]>>,
                controlled_beings_query: bevy::prelude::Query<(&being_shared::ComputedBy, $extra_query_ty)>,
                mut writer: bevy::prelude::MessageWriter<[<Local $base Request>]>,
                mut local_msgs_to_write_out: bevy::prelude::Local<Vec<[<Local $base Request>]>>,
            ) {
                for from_client in events.read() {
                    let [<Client $base Request>] { being_ent } = from_client.message.clone();
                    let Ok((controlled_by, $extra_binding)) = controlled_beings_query.get(being_ent) else {
                        warn!(target: $log_target, "Client tried to trigger {} with missing/uncontrolled being {}", stringify!($base), being_ent);
                        continue;
                    };
                    let Some(client_entity) = from_client.client_id.entity() else {
                        continue;
                    };
                    if controlled_by.client_ent != client_entity {
                        warn!(
                            target: $log_target,
                            "Client tried to trigger {} with a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                            stringify!($base),
                            being_ent,
                            controlled_by.client_ent,
                            client_entity
                        );
                        continue;
                    }
                    local_msgs_to_write_out.push([<Local $base Request>] { being_ent });
                }
                writer.write_batch(local_msgs_to_write_out.drain(..));
            }

            pub fn [<$base:snake _plugin>](app: &mut bevy::prelude::App) {
                app.add_message::<[<Local $base Request>]>()
                    .add_systems(
                        bevy::prelude::Update,
                        (
                            [<trigger_ $base:snake _requests_from_player_input>],
                            [<send_ $base:snake _request_to_server>]
                                .run_if(bevy::prelude::in_state(bevy_replicon::prelude::ClientState::Connected)),
                            [<receive_ $base:snake _from_client>]
                                .run_if(bevy::prelude::in_state(bevy_replicon::prelude::ServerState::Running))
                                .run_if(bevy::prelude::on_message::<
                                    bevy_replicon::prelude::FromClient<[<Client $base Request>]>,
                                >),
                        )
                            .in_set(game_common::game_common::GameplaySystems),
                    )
                    .add_mapped_client_message::<[<Client $base Request>]>(bevy_replicon::prelude::Channel::Unordered);//<-- ojo !
            }
        }
    };
}
