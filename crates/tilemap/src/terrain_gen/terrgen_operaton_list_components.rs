
use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_tag_components::AddSameHashedTags;

use {common::common_components::*, };
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, Clone, MapEntities)]
pub struct Bifurcation{
    #[entities] pub oplist: Option<Entity>,
    #[entities]pub tiles: Vec<Entity>,
}
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(Prefix::trunc("OpList"), Replicated, AssetScoped, AddSameHashedTags)]
#[component(map_entities)]
pub struct OperationList {
    /// Expression tree representation (slot-free runtime system)
    pub expr_tree: crate::terrain_gen::terrgen_expression::ExprOpList,
    pub bifurcations: Vec<Bifurcation>,
}

impl Default for OperationList {
    fn default() -> Self {
        Self {
            expr_tree: crate::terrain_gen::terrgen_expression::ExprOpList {
                assignments: Vec::new(),
                output: crate::terrain_gen::terrgen_expression::Expr::Literal(0.0),
            },
            bifurcations: Vec::new(),
        }
    }
}

impl MapEntities for OperationList {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for assignment in self.expr_tree.assignments.iter_mut() {
            map_expr_entities(&mut assignment.expr, entity_mapper);
        }
        map_expr_entities(&mut self.expr_tree.output, entity_mapper);
        for bifur in self.bifurcations.iter_mut() {
            bifur.oplist = bifur.oplist.map(|oplist_entity| entity_mapper.get_mapped(oplist_entity));
            bifur.tiles.iter_mut().for_each(|tile_entity| *tile_entity = entity_mapper.get_mapped(*tile_entity));
        }
    }
}

fn map_expr_entities<E: EntityMapper>(
    expr: &mut crate::terrain_gen::terrgen_expression::Expr,
    entity_mapper: &mut E,
) {
    use crate::terrain_gen::terrgen_expression::Expr;
    match expr {
        Expr::Noise { entity, .. } => {
            *entity = entity_mapper.get_mapped(*entity);
        }
        Expr::Add { left, right }
        | Expr::Subtract { left, right }
        | Expr::Multiply { left, right }
        | Expr::Divide { left, right }
        | Expr::MultiplyNormalized { left, right }
        | Expr::MultiplyNormalizedAbs { left, right } => {
            map_expr_entities(left, entity_mapper);
            map_expr_entities(right, entity_mapper);
        }
        Expr::MultiplyOpo { value }
        | Expr::Abs { value }
        | Expr::Complement { value } => {
            map_expr_entities(value, entity_mapper);
        }
        Expr::Min { values }
        | Expr::Max { values }
        | Expr::Average { values }
        | Expr::IndexMax { values }
        | Expr::Linear { values } => {
            for value in values.iter_mut() {
                map_expr_entities(value, entity_mapper);
            }
        }
        Expr::IndexNorm { value, multiplier } => {
            map_expr_entities(value, entity_mapper);
            map_expr_entities(multiplier, entity_mapper);
        }
        Expr::Clamp { value, min, max } => {
            map_expr_entities(value, entity_mapper);
            map_expr_entities(min, entity_mapper);
            map_expr_entities(max, entity_mapper);
        }
        Expr::Literal(_)
        | Expr::NoiseByName { .. }
        | Expr::HashPos { .. }
        | Expr::PoissonDisk { .. }
        | Expr::Variable { .. } => {}
    }
}
