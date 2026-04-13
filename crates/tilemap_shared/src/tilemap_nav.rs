use bevy::prelude::*;

use crate::DimensionRef;

#[derive(Message, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AiNavGridDirtyDim {
	pub dim: DimensionRef,
}
