use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component, Default)]
pub struct EguiInputBindingsHolder;

#[derive(Component, Default)]
pub struct EguiInputActionsHolder;

#[derive(Component, Default)]
pub struct EguiObserverHolder;

#[allow(unused_parens, )]
pub fn spawn_egui_holders(mut commands: Commands) {
    commands.spawn((EguiInputBindingsHolder, Name::new("EguiInputBindingsHolder")));
    commands.spawn((EguiInputActionsHolder, Name::new("EguiInputActionsHolder")));
    commands.spawn((EguiObserverHolder, Name::new("EguiObserverHolder")));
}

#[allow(unused_parens, )]
pub fn sync_egui_input_holders(
    mut commands: Commands,
    bindings_holder_query: Query<Entity, With<EguiInputBindingsHolder>>,
    actions_holder_query: Query<Entity, With<EguiInputActionsHolder>>,
    bindings_query: Query<(Entity), Added<Binding>>,
    trigger_states_query: Query<(Entity), Added<TriggerState>>,
) {
    let Ok(bindings_holder) = bindings_holder_query.single() else {
        warn!("No EguiInputBindingsHolder found");
        return;
    };
    let Ok(actions_holder) = actions_holder_query.single() else {
        warn!("No EguiInputActionsHolder found");
        return;
    };

    for entity in bindings_query.iter() {
        commands.entity(entity).try_insert_if_new(ChildOf(bindings_holder));
    }
    for entity in trigger_states_query.iter() {
        commands.entity(entity).try_insert_if_new(ChildOf(actions_holder));
    }
}

#[allow(unused_parens, )]
pub fn make_observers_be_children_of_egui_holder(
    mut commands: Commands,
    observer_holder_query: Query<Entity, With<EguiObserverHolder>>,
    observer_query: Query<(Entity), Added<Observer>>,
) {
    let Ok(observer_holder) = observer_holder_query.single() else {
        return;
    };

    for entity in observer_query.iter() {
        commands.entity(entity).try_insert_if_new(ChildOf(observer_holder));
    }
}
