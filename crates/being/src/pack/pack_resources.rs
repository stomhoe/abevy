use bevy::{prelude::*};

use crate::pack::pack_components::Pack;
pub use crate::pack::pack_seris::*;

common::define_entity_map_systems!(Pack, PackSeri, "seri.being.pack", "pack.ron",);
