use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use tilemap_shared::{GlobalTilePos, OplistSize, GlobalGenSettings, HashablePosVec, PoissonDisk};
use crate::terrain_gen::terrgen_components::FnlNoiseComp;
use common::common_components::HashId;

/// Expression tree for terrain generation operations
/// Replaces the slot-based system with a composable AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal constant value
    Literal(f32),

    /// Reference to a noise entity
    Noise {
        entity: Entity,
        sample_range: fnl::NoiseSampleRange,
        complement: bool,
        seed_offset: i32,
    },

    /// Reference to a noise by name (resolved later to entity)
    NoiseByName {
        name: String,
        sample_range: fnl::NoiseSampleRange,
        complement: bool,
        seed_offset: i32,
    },

    /// Hash-based positional value
    HashPos { seed: u64 },

    /// Poisson disk sampling
    PoissonDisk {
        min_dist: u8,
        seed: u64,
    },

    /// Variable reference (inherited from parent oplist or locally defined)
    Variable { name: String },

    /// Binary operations
    Add { left: Box<Expr>, right: Box<Expr> },
    Subtract { left: Box<Expr>, right: Box<Expr> },
    Multiply { left: Box<Expr>, right: Box<Expr> },
    Divide { left: Box<Expr>, right: Box<Expr> },

    /// Multiply by opposite (1 - x)
    MultiplyOpo { value: Box<Expr> },

    /// Min/Max operations
    Min { values: Vec<Expr> },
    Max { values: Vec<Expr> },

    /// Average of values
    Average { values: Vec<Expr> },

    /// Absolute value
    Abs { value: Box<Expr> },

    /// Multiply by normalized value (-0.5 to 0.5 range)
    MultiplyNormalized { left: Box<Expr>, right: Box<Expr> },
    MultiplyNormalizedAbs { left: Box<Expr>, right: Box<Expr> },

    /// Index of maximum value
    IndexMax { values: Vec<Expr> },

    /// Index normalized to range
    IndexNorm { value: Box<Expr>, multiplier: Box<Expr> },

    /// Linear interpolation
    Linear { values: Vec<Expr> },

    /// Clamp value between min and max
    Clamp { value: Box<Expr>, min: Box<Expr>, max: Box<Expr> },

    /// Complement (1 - x)
    Complement { value: Box<Expr> },
}

impl Expr {
    /// Evaluate the expression recursively
    pub fn eval(&self, context: &EvalContext) -> f32 {
        match self {
            Expr::Literal(v) => *v,

            Expr::NoiseByName { .. } => {
                // NoiseByName should be resolved to Noise during init
                // If we reach here, it means resolution failed
                warn!(target: "oplist_eval", "Unresolved NoiseByName reached runtime eval; returning 0.0");
                0.0
            }

            Expr::Noise { entity, sample_range, complement, seed_offset } => {
                if let Some(noise) = context.noises.get(entity) {
                    let mut value = noise.sample(
                        context.global_pos,
                        context.dimension_hash,
                        *sample_range,
                        false, // complement handled separately
                        *seed_offset,
                        context.gen_settings,
                    );
                    if *complement {
                        value = 1.0 - value;
                    }
                    value
                } else {
                    0.0
                }
            }

            Expr::HashPos { seed } => {
                context.global_pos.normalized_hash_value(
                    context.gen_settings,
                    context.dimension_hash,
                    *seed,
                ) as f32
            }

            Expr::PoissonDisk { min_dist, seed } => {
                match PoissonDisk::new(*min_dist, *seed) {
                    Ok(pd) => pd.sample(
                        context.global_pos,
                        context.gen_settings,
                        context.dimension_hash,
                        true,
                        context.oplist_size,
                    ) as f32,
                    Err(_) => 0.0,
                }
            }

            Expr::Variable { name } => {
                if let Some(v) = context.variables.get(name) {
                    *v
                } else {
                    static MISSING_VARS_WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
                    let warned = MISSING_VARS_WARNED.get_or_init(|| Mutex::new(HashSet::new()));
                    if let Ok(mut set) = warned.lock() && set.insert(name.clone()) {
                        warn!(
                            target: "oplist_eval",
                            "Missing variable '{}' during expr eval; defaulting to 0.0",
                            name
                        );
                    }
                    0.0
                }
            }

            Expr::Add { left, right } => left.eval(context) + right.eval(context),
            Expr::Subtract { left, right } => left.eval(context) - right.eval(context),
            Expr::Multiply { left, right } => left.eval(context) * right.eval(context),
            Expr::Divide { left, right } => {
                let r = right.eval(context);
                if r != 0.0 {
                    left.eval(context) / r
                } else {
                    left.eval(context)
                }
            }

            Expr::MultiplyOpo { value } => 1.0 - value.eval(context),

            Expr::Min { values } => {
                values.iter()
                    .map(|v| v.eval(context))
                    .fold(f32::INFINITY, f32::min)
            }

            Expr::Max { values } => {
                values.iter()
                    .map(|v| v.eval(context))
                    .fold(f32::NEG_INFINITY, f32::max)
            }

            Expr::Average { values } => {
                if values.is_empty() {
                    return 0.0;
                }
                let sum: f32 = values.iter().map(|v| v.eval(context)).sum();
                sum / values.len() as f32
            }

            Expr::Abs { value } => value.eval(context).abs(),

            Expr::MultiplyNormalized { left, right } => {
                left.eval(context) * ((right.eval(context) - 0.5) * 2.0)
            }

            Expr::MultiplyNormalizedAbs { left, right } => {
                left.eval(context) * ((right.eval(context) - 0.5) * 2.0).abs()
            }

            Expr::IndexMax { values } => {
                let mut max_idx = 0;
                let mut max_val = f32::NEG_INFINITY;
                for (idx, expr) in values.iter().enumerate() {
                    let val = expr.eval(context);
                    if val > max_val {
                        max_val = val;
                        max_idx = idx;
                    }
                }
                max_idx as f32
            }

            Expr::IndexNorm { value, multiplier } => {
                value.eval(context) * multiplier.eval(context)
            }

            Expr::Linear { values } => {
                // Legacy terrain-gen "lin" behavior:
                // lin(x, a, b, m1, m2, ...) = sigmoid(a*x + b) * m1 * m2 * ...
                if values.len() < 3 {
                    return 0.0;
                }
                let x = values[0].eval(context);
                let a = values[1].eval(context);
                let b = values[2].eval(context);
                let mut result = 1.0 / (1.0 + (-(a * x + b)).exp());
                for expr in &values[3..] {
                    result *= expr.eval(context);
                }
                result
            }

            Expr::Clamp { value, min, max } => {
                let v = value.eval(context);
                let min_v = min.eval(context);
                let max_v = max.eval(context);
                v.max(min_v).min(max_v)
            }

            Expr::Complement { value } => {
                1.0 - value.eval(context)
            }
        }
    }

    /// Collect all noise entities referenced in this expression
    pub fn collect_noise_entities(&self, out: &mut Vec<Entity>) {
        match self {
            Expr::NoiseByName { .. } => {
                // NoiseByName entities will be collected after resolution
            }
            Expr::Noise { entity, .. } => {
                if !out.contains(entity) {
                    out.push(*entity);
                }
            }
            Expr::Add { left, right }
            | Expr::Subtract { left, right }
            | Expr::Multiply { left, right }
            | Expr::Divide { left, right }
            | Expr::MultiplyNormalized { left, right }
            | Expr::MultiplyNormalizedAbs { left, right } => {
                left.collect_noise_entities(out);
                right.collect_noise_entities(out);
            }
            Expr::MultiplyOpo { value }
            | Expr::Abs { value }
            | Expr::Complement { value } => {
                value.collect_noise_entities(out);
            }
            Expr::Min { values }
            | Expr::Max { values }
            | Expr::Average { values }
            | Expr::IndexMax { values }
            | Expr::Linear { values } => {
                for v in values {
                    v.collect_noise_entities(out);
                }
            }
            Expr::IndexNorm { value, multiplier } => {
                value.collect_noise_entities(out);
                multiplier.collect_noise_entities(out);
            }
            Expr::Clamp { value, min, max } => {
                value.collect_noise_entities(out);
                min.collect_noise_entities(out);
                max.collect_noise_entities(out);
            }
            _ => {}
        }
    }
}

/// Evaluation context for expressions
pub struct EvalContext<'a> {
    pub global_pos: GlobalTilePos,
    pub dimension_hash: HashId,
    pub gen_settings: &'a GlobalGenSettings,
    pub oplist_size: OplistSize,
    pub noises: &'a HashMap<Entity, FnlNoiseComp>,
    pub variables: &'a HashMap<String, f32>,
}

/// Variable assignment in an oplist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub name: String,
    pub expr: Expr,
}

/// Oplist definition using expression trees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprOpList {
    /// Named variable assignments
    pub assignments: Vec<Assignment>,

    /// Final output expression (what becomes 'out')
    pub output: Expr,
}

impl ExprOpList {
    /// Evaluate the oplist, returning the output value and computed variables
    pub fn eval(&self, parent_vars: &HashMap<String, f32>, context: &EvalContext) -> (f32, HashMap<String, f32>) {
        let mut variables = parent_vars.clone();

        // Evaluate assignments in order
        for assignment in &self.assignments {
            let local_context = EvalContext {
                variables: &variables,
                ..*context
            };
            let value = assignment.expr.eval(&local_context);
            variables.insert(assignment.name.clone(), value);
        }

        // Evaluate output expression
        let output_context = EvalContext {
            variables: &variables,
            ..*context
        };
        let output = self.output.eval(&output_context);

        (output, variables)
    }
}
