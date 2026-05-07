

use ::tilemap_shared::*;

use bevy::{prelude::*};
use rand::RngExt;

define_weightedsampler!(ColorSampler, [u8; 4], "ColorWeightedSampler");
