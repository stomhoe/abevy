#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::time::{time_resources::*, time_types::*};



//NO OLVIDARSE DE AGREGARLO AL PLUGIN
pub fn pass_time(
    time: Res<Time>,
    sim_timescale: Res<SimTimeScale>,
    mut curr_min: ResMut<CurrMin>,
    mut curr_hour: ResMut<CurrHour>,
    mut curr_day: ResMut<CurrDay>,
    mut curr_season: ResMut<CurrSeason>,
    mut curr_year: ResMut<CurrYear>,
    ingame_timing: Res<InGameTiming>,
) {
    let time_scale = ingame_timing.time_scale_rela2irl() * sim_timescale.0;

    let days_per_year = ingame_timing.days_per_year();

    curr_min.0.0 += time.delta_secs() * time_scale;
    if curr_min.0.0 > 60.0 {
        curr_hour.0 += Hours(curr_min.0.0 as u32 / 60);
        curr_min.0.0 %= 60.0;
    }
    if curr_hour.0 >= Hours(24) {
        curr_day.0 += Days(curr_hour.0 .0  / 24);
        curr_hour.0.0 %= 24;
    }
    if curr_day.is_changed() && curr_day.0.0 % ingame_timing.days_per_season().0 == 0 && curr_day.0 .0 > 1 {
        curr_season.0 = curr_season.0.next();
    }
    if curr_day.0 > days_per_year {
        curr_day.0 -= days_per_year;
        curr_year.0 += Years(1);
    }
}
