    use bevy::prelude::*;
    
    // Module things
    mod things_components;
    //mod things_systems;
    //mod things_events;
    //mod things_styles;
    //mod things_resources;
    pub struct thingsPlugin;
    #[allow(unused_parens)]
    impl Plugin for thingsPlugin {
        fn build(&self, app: &mut App) {
            app
                //.add_systems(Update, (somesystem))
                //.add_systems(OnEnter(SomeStateType::Literal), (setup))
                //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
            ;
        }
    }