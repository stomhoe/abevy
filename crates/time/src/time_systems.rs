#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::SettingsEntity;
use time_shared::*;

pub fn pass_time(
    time: Res<Time>,
    sim_timescale_query: Query<&SimTimeScale, With<SettingsEntity>>,
    mut curr_min: ResMut<CurrMin>,
    mut curr_hour: ResMut<CurrHour>,
    mut curr_day: ResMut<CurrDay>,
    mut curr_season: ResMut<CurrSeason>,
    mut curr_year: ResMut<CurrYear>,
    ingame_timing: Res<InGameTiming>,
) {
    let Ok(sim_timescale) = sim_timescale_query.single() else {
        return;
    };
    let day_length_minutes = ingame_timing.day_length_minutes().max(1.0);
    let time_scale = (24.0 / day_length_minutes) * sim_timescale.0;

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


#[allow(unused_parens)]
pub fn reduce_remaining_days(mut query: Query<(&mut RemainingDays),()>, curr_day: Res<CurrDay>) {
    
    for mut remaining_days in query.iter_mut() {
        if curr_day.is_changed() && remaining_days.0 > Days(0) {
            remaining_days.0 -= Days(1);
        } 
    }
}




