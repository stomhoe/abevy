#[allow(unused_imports)] use bevy::prelude::*;





//NO OLVIDARSE DE AGREGARLO AL PLUGIN
#[allow()]
pub fn tick_adj2simspeed(
    time: Timer,
    mut sim_timescale: Res<SimTimeScale>,
) {
    sim_timescale.0 = time.delta_secs() * time_scale;
}
