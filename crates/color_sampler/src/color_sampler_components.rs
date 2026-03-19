

use game_common::define_weightedsampler;
use ::tilemap_shared::*;

use bevy::{prelude::*};

define_weightedsampler!(ColorSampler, [u8; 4], "ColorWeightedSampler");
