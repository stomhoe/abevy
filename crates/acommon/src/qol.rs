use bevy::{ecs::component::Component, prelude::App};
use bevy_replicon::{prelude::AppRuleExt, shared::replication::registry::command_fns::MutWrite};
use serde::{Serialize, de::DeserializeOwned};

pub trait AppRegisterAndReplicateExt {

    fn regrepli<C: Component<Mutability: MutWrite<C>> + Serialize + DeserializeOwned + bevy::reflect::Reflect + bevy::reflect::GetTypeRegistration + 'static>(self) -> Self;
    fn regrepli_once<C: Component<Mutability: MutWrite<C>> + Serialize + DeserializeOwned + bevy::reflect::Reflect + bevy::reflect::GetTypeRegistration + 'static>(self) -> Self;
}

impl AppRegisterAndReplicateExt for &mut App {
    fn regrepli<C: Component<Mutability: MutWrite<C>> + Serialize + DeserializeOwned + bevy::reflect::Reflect + bevy::reflect::GetTypeRegistration + 'static>(self) -> Self {
        self.register_type::<C>()
            .replicate::<C>()
    }
    fn regrepli_once<C: Component<Mutability: MutWrite<C>> + Serialize + DeserializeOwned + bevy::reflect::Reflect + bevy::reflect::GetTypeRegistration + 'static>(self) -> Self {
        self.register_type::<C>()
            .replicate_once::<C>()
    }
}
