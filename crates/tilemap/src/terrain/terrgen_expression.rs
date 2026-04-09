use bevy::prelude::*;
use bevy::platform::collections::HashSet as BevyHashSet;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use tilemap_shared::{GlobalTilePos, OplistSize, GlobalGenSettings, HashablePosVec, PoissonDisk};
use crate::terrain::terrgen_components::FnlNoiseComp;
use common::common_components::{HashId, HashIdMap, StrId};

impl Default for Expr {
    fn default() -> Self {
        Self::Literal(0.0)
    }
}
/// Expression tree for terrain generation operations
/// Replaces the slot-based system with a composable AST
#[derive(Debug, Clone, Serialize, Deserialize, )]
pub enum Expr {
    /// Literal constant value
    Literal(f32),

    /// Reference to a noise entity by hash id
    Noise {
        hash_id: HashId,
        sample_range: fnl::NoiseSampleRange,
        complement: bool,
        seed_offset: i32,
    },

    /// Reference to a noise by name hash
    NoiseByName {
        name: HashId,
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
    Variable { name: StrId },

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

    /// Index of maximum score where each island value competes against
    /// the mean of the remaining islands.
    /// Operand order is [ocean_threshold, island_0, island_1, ...].
    IndexMaxIslands { values: Vec<Expr> },

    /// Stable per-island score:
    /// islanddiff(index, island_0, island_1, ...) = island[index] - mean(other islands)
    IslandDiff { values: Vec<Expr> },

    /// Remap a value from one range into another, clamped to the output range.
    /// remap(value, input_min, input_max, output_min, output_max)
    RemapRange { value: Box<Expr>, input_min: Box<Expr>, input_max: Box<Expr>, output_min: Box<Expr>, output_max: Box<Expr> },

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
    fn remap_clamped(value: f32, input_min: f32, input_max: f32, output_min: f32, output_max: f32) -> f32 {
        let input_span = input_max - input_min;
        if !input_span.is_finite() || input_span.abs() < f32::EPSILON {
            return output_min;
        }

        let t = ((value - input_min) / input_span).clamp(0.0, 1.0);
        output_min + t * (output_max - output_min)
    }

    /// Evaluate the expression recursively
    pub fn eval(&self, context: &EvalContext) -> f32 {
        match self {
            Expr::Literal(v) => *v,

            Expr::Noise { hash_id, sample_range, complement, seed_offset }
            | Expr::NoiseByName { name: hash_id, sample_range, complement, seed_offset } => {
                let Ok(noise) = context.noises.get(*hash_id) else {
                    return 0.0;
                };
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
                let var_id = HashId::from(name.as_str());
                if let Ok(v) = context.variables.get(var_id) {
                    *v
                } else {
                    static MISSING_VARS_WARNED: OnceLock<Mutex<BevyHashSet<HashId>>> = OnceLock::new();
                    let warned = MISSING_VARS_WARNED.get_or_init(|| Mutex::new(BevyHashSet::new()));
                    if let Ok(mut set) = warned.lock() && set.insert(var_id) {
                        warn!(
                            target: "oplist_eval",
                            "Missing variable '{}' ({:?}) during expr eval; defaulting to 0.0",
                            name,
                            var_id
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

            Expr::IndexMaxIslands { values } => {
                if values.is_empty() {
                    return 0.0;
                }

                let ocean_threshold = values[0].eval(context);
                if values.len() == 1 {
                    return 0.0;
                }

                let island_values: Vec<f32> = values[1..].iter().map(|v| v.eval(context)).collect();
                let islands_count = island_values.len();
                if islands_count == 0 {
                    return 0.0;
                }
                let total_island_sum: f32 = island_values.iter().sum();

                let mut max_idx = 0usize;
                let mut max_val = ocean_threshold;

                for (island_i, island_value) in island_values.iter().enumerate() {
                    let score = if islands_count > 1 {
                        let other_mean = (total_island_sum - island_value) / (islands_count as f32 - 1.0);
                        island_value - other_mean
                    } else {
                        *island_value
                    };
                    if score > max_val {
                        max_val = score;
                        max_idx = island_i + 1;
                    }
                }

                max_idx as f32
            }

            Expr::IslandDiff { values } => {
                if values.len() < 2 {
                    return 0.0;
                }
                let raw_index = values[0].eval(context);
                if !raw_index.is_finite() {
                    return 0.0;
                }
                let island_values: Vec<f32> = values[1..].iter().map(|v| v.eval(context)).collect();
                if island_values.is_empty() {
                    return 0.0;
                }

                let index = raw_index.round() as isize;
                if index < 0 || index >= island_values.len() as isize {
                    return 0.0;
                }
                let index = index as usize;
                let self_value = island_values[index];
                if island_values.len() == 1 {
                    return self_value;
                }

                let total_sum: f32 = island_values.iter().sum();
                let other_mean = (total_sum - self_value) / (island_values.len() as f32 - 1.0);
                self_value - other_mean
            }

            Expr::RemapRange { value, input_min, input_max, output_min, output_max } => {
                let value = value.eval(context);
                let input_min = input_min.eval(context);
                let input_max = input_max.eval(context);
                let output_min = output_min.eval(context);
                let output_max = output_max.eval(context);
                Self::remap_clamped(value, input_min, input_max, output_min, output_max)
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
    pub fn collect_noise_hash_ids(&self, out: &mut BevyHashSet<HashId>) {
        match self {
            Expr::Noise { hash_id, .. } | Expr::NoiseByName { name: hash_id, .. } => {
                out.insert(*hash_id);
            }
            Expr::Add { left, right }
            | Expr::Subtract { left, right }
            | Expr::Multiply { left, right }
            | Expr::Divide { left, right }
            | Expr::MultiplyNormalized { left, right }
            | Expr::MultiplyNormalizedAbs { left, right } => {
                left.collect_noise_hash_ids(out);
                right.collect_noise_hash_ids(out);
            }
            Expr::MultiplyOpo { value }
            | Expr::Abs { value }
            | Expr::Complement { value } => {
                value.collect_noise_hash_ids(out);
            }
            Expr::Min { values }
            | Expr::Max { values }
            | Expr::Average { values }
            | Expr::IndexMax { values }
            | Expr::IndexMaxIslands { values }
            | Expr::IslandDiff { values }
            | Expr::Linear { values } => {
                for v in values {
                    v.collect_noise_hash_ids(out);
                }
            }
            Expr::RemapRange { value, input_min, input_max, output_min, output_max } => {
                value.collect_noise_hash_ids(out);
                input_min.collect_noise_hash_ids(out);
                input_max.collect_noise_hash_ids(out);
                output_min.collect_noise_hash_ids(out);
                output_max.collect_noise_hash_ids(out);
            }
            Expr::IndexNorm { value, multiplier } => {
                value.collect_noise_hash_ids(out);
                multiplier.collect_noise_hash_ids(out);
            }
            Expr::Clamp { value, min, max } => {
                value.collect_noise_hash_ids(out);
                min.collect_noise_hash_ids(out);
                max.collect_noise_hash_ids(out);
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
    pub noises: &'a HashIdMap<FnlNoiseComp>,
    pub variables: &'a HashIdMap<f32>,
}

/// Variable assignment in an oplist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub name: HashId,
    pub expr: Expr,
}

/// Oplist definition using expression trees
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExprOpList {
    /// Named variable assignments
    pub assignments: Vec<Assignment>,

    /// Final output expression (what becomes 'out')
    pub output: Expr,
}

impl ExprOpList {
    /// Evaluate the oplist, returning the output value and computed variables
    pub fn eval(&self, parent_vars: &HashIdMap<f32>, context: &EvalContext) -> (f32, HashIdMap<f32>) {
        let mut variables = parent_vars.clone();

        // Evaluate assignments in order
        for assignment in &self.assignments {
            let local_context = EvalContext {
                variables: &variables,
                ..*context
            };
            let value = assignment.expr.eval(&local_context);
            let _ = variables.overwrite(assignment.name, value);
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
