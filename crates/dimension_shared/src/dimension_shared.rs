use bevy::platform::collections::HashSet;
use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap, prelude::*};
use bevy::{ecs::entity::MapEntities, prelude::*};

use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DimensionSystems;


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct PrevDimensionRef(#[entities] pub Entity);




#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect, MapEntities)]
#[relationship(relationship_target = RootInDimensions)]
pub struct DimensionRootOplist(#[relationship]#[entities]pub Entity);

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = DimensionRootOplist)]
pub struct RootInDimensions(EntityHashSet);
impl RootInDimensions { pub fn entities(&self) -> &EntityHashSet { &self.0 } }



#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
pub struct MultipleDimensionStringRefs(Vec<String>);

impl MultipleDimensionStringRefs {
    pub fn new(strings: Vec<String>) -> Self {
        let filtered = strings.into_iter().filter(|s| !s.is_empty()).collect();
        MultipleDimensionStringRefs(filtered)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }
}

#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect, MapEntities, )]
pub struct MultipleDimensionRefs(#[entities] pub EntityHashSet,);

#[derive(Debug, Message)]
pub struct ReassignDimensionToEntity (pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct WhitelistedStructureGenTags(pub TagSet);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, )]
pub struct BlacklistedStructureGenTags(pub TagSet);
