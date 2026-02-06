use bevy::ecs::query::{Changed, With};



#[allow(unused_parens)]
pub fn build_beings_from_race_ref(mut cmd: Commands,
    query: Query<(Entity, ), (Changed<RaceRef>, With<Being>)>,
){
    for ent in query.iter() {

    }
}
