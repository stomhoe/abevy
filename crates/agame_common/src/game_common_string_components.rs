
use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use common::common_components::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;
use splines::{Interpolation, Key, Spline};
use strum_macros::{AsRefStr, Display, };
use std::hash::Hash;


#[derive(Bundle)]
/// for denying on cloning
pub struct GameCommonStringComponentsBundle {
    pub str_id: StrId,
    pub display_name: DisplayName,
    pub description: Description,
    pub demonym: Demonym,
    pub singular_denomination: SingularDenomination,
    pub plural_denomination: PluralDenomination,

}

#[allow(unused_parens, dead_code)]
#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct Description(pub String);

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct Demonym(pub StrId);

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct SingularDenomination(pub StrId);

#[derive(Component, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct PluralDenomination(pub StrId);