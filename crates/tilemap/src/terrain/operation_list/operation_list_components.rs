
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
    pub biome_tags: Vec<(HashId, f32)>,
}
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(Prefix::trunc("OpList"), Replicated, AssetScoped, HotReload, AddSameHashedTags)]
#[component(map_entities)]
pub struct OperationList {
    /// Expression tree representation (slot-free runtime system)
    pub expr_tree: crate::terrain::terrgen_expression::ExprOpList,
    /// Variable names to keep in runtime debug capture for this oplist.
    pub hash_ids_mapped_to_strids: HashIdMap<StrId>,
    pub bifurcations: Vec<Bifurcation>,
    /// Precompiled branch tree with child oplists inlined for fast recursive eval.
    #[serde(skip, default)]
    pub compiled_branch_ast: Option<CompiledBranchNode>,
}

impl Default for OperationList {
    fn default() -> Self {
        Self {
            expr_tree: crate::terrain::terrgen_expression::ExprOpList {
                assignments: Vec::new(),
                output: crate::terrain::terrgen_expression::Expr::Literal(0.0),
            },
            hash_ids_mapped_to_strids: HashIdMap::default(),
            bifurcations: Vec::new(),
            compiled_branch_ast: None,
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
        if let Some(ast) = self.compiled_branch_ast.as_mut() {
            map_compiled_branch_entities(ast, entity_mapper);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledBranchNode {
    pub source_oplist: Entity,
    pub expr_tree: crate::terrain::terrgen_expression::ExprOpList,
    pub branches: Vec<CompiledBranch>,
}

#[derive(Debug, Clone)]
pub struct CompiledBranch {
    pub tiles: Vec<Entity>,
    pub biome_tags: Vec<(HashId, f32)>,
    pub child_size: Option<tilemap_shared::OplistSize>,
    pub child: Option<Box<CompiledBranchNode>>,
}

fn map_expr_entities<E: EntityMapper>(
    expr: &mut crate::terrain::terrgen_expression::Expr,
    entity_mapper: &mut E,
) {
    use crate::terrain::terrgen_expression::Expr;
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

fn map_compiled_branch_entities<E: EntityMapper>(
    node: &mut CompiledBranchNode,
    entity_mapper: &mut E,
) {
    node.source_oplist = entity_mapper.get_mapped(node.source_oplist);
    for assignment in node.expr_tree.assignments.iter_mut() {
        map_expr_entities(&mut assignment.expr, entity_mapper);
    }
    map_expr_entities(&mut node.expr_tree.output, entity_mapper);
    for branch in node.branches.iter_mut() {
        for tile in branch.tiles.iter_mut() {
            *tile = entity_mapper.get_mapped(*tile);
        }
        if let Some(child) = branch.child.as_mut() {
            map_compiled_branch_entities(child, entity_mapper);
        }
    }
}
