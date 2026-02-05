
use bevy::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize, };


#[derive(Bundle)]
/// for denying on cloning
pub struct GameCommonStringComponentsBundle {
    pub str_id: StrId,
    pub display_name: DisplayName,
    pub description: Description,
    pub demonym: Demonym,
    pub singular_denomination: SingularDenomination,
    pub plural_denomination: PluralDenomination,
    pub prefix: Prefix,

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