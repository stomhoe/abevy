use bevy::prelude::*;
use tilemap::terrain_gen::terrgen_resources::*;


pub fn all_clean_resources(
    mut cmd: Commands,
){

    cmd.remove_resource::<TerrGenEntityMap>();
    cmd.remove_resource::<OpListEntityMap>();
}
