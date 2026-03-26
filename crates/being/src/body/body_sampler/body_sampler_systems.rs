use being_shared::BeingInstTemplate;
use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use tilemap_shared::tilemap_shared_samplers::EntityWeightedSampler;

use crate::{
    body::{
        body_sampler::{body_sampler_components::BodyWeightedSampler, body_sampler_resources::BodyWeightedSamplerRef},
        body_tree_components::*,
        body_tree_resources::*,
    }, race::race_components::Race
};

#[allow(unused_parens)]
/// recursively samples samplers until some entity which is not sampler is found, then that is inserted into the being as a bodytreerref
pub fn sample_nested_body_samplers_until_body_tree_is_found(
    mut cmd: Commands,
    being_query: Query<(Entity, &BodyWeightedSamplerRef), (Changed<BodyWeightedSamplerRef>, Without<BeingInstTemplate>, Without<Race>)>,
    body_samplers_query: Query<(&EntityWeightedSampler), (With<BodyWeightedSampler>,)>,
    bodies_query: Query<(), (With<BodyTree>, )>,
) {
    let mut body_tree_refs_to_insert: Vec<(Entity, BodyTreeRef)> = Vec::new();
    let mut rng = rand::rng();

    for (being_ent, sampler_ref) in being_query {
        let mut curr_ent = sampler_ref.0;
        let mut visited: EntityHashSet = EntityHashSet::default();
        while visited.insert(curr_ent) {
            if bodies_query.get(curr_ent).is_ok() {
                body_tree_refs_to_insert.push((being_ent, BodyTreeRef(curr_ent)));
                break;
            }
            let Ok(body_sampler) = body_samplers_query.get(curr_ent) else { break; };
            let Some(next_ent) = body_sampler.sample_with_rng(&mut rng) else { break; };
            curr_ent = next_ent;
        }
    }

    cmd.try_insert_batch(body_tree_refs_to_insert);
}
