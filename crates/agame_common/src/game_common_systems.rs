use crate::game_common_components::*;
use crate::game_common_bundles::DenyForTemplClonedChildren;
use crate::game_common_timers::*;
use ::common::*;
use bevy::prelude::*;
pub fn tick_time_based_multipliers(
    time: Res<Time>,
    mut query: Query<(
        &mut TimeBasedMultiplier,
        Option<&TickMultFactor>,
        Option<&TickMultFactors>,
    )>,
) {
    for (mut multiplier, tick_mult_factor, tick_mult_factors) in query.iter_mut() {
        let mut factor = tick_mult_factor.map(|f| f.value()).unwrap_or(1.0);
        if let Some(factors) = tick_mult_factors {
            factor *= factors.0.iter().map(|f| f.value()).product::<f32>();
        }
        multiplier.timer.tick(time.delta().mul_f32(factor));
    }
}

#[allow(unused_parens)]
pub fn despawn_sprites_without_childof(
    mut cmd: Commands,
    query: Query<(Entity), (Or<(With<Sprite>, With<Mesh2d>)>, Without<ChildOf>, common::AnyDisabling)>,
) {
    query
        .iter()
        .for_each(|sprite_ent| cmd.entity(sprite_ent).try_despawn());
}

#[allow(unused_parens)]
pub fn set_entity_name(
    templs_query: Query<
        AnyOf<(&Prefix, &StrId, &StrId20B, &HashId, &DisplayName,)>,
        (With<Templ>, common::AnyDisabling),
    >,
    mut changers_query: Query<
        (
            &mut Name,
            AnyOf<(&Prefix, &StrId, &StrId20B, &HashId, &DisplayName, &TemplEntiRef)>,
        ),
        (
            Without<ExcludedFromAutoRenamer>,
            Or<(
                Changed<Prefix>,
                Changed<StrId>,
                Changed<StrId20B>,
                Changed<HashId>,
                Changed<DisplayName>,
                Changed<TemplEntiRef>,
            )>,
            common::AnyDisabling,
        ),
    >,
) {
    for (mut name, (e_prefix, strid, strid20b, hash_id, display_name, templ_ref)) in
        changers_query.iter_mut()
    {
        let mut prefix = e_prefix.map(|p| p.as_str());
        let mut sid = strid.map(|s| s.as_str());
        let mut sid20 = strid20b.map(|s| s.as_str());
        let mut hash_id = hash_id;
        let mut display_name = display_name;

        let mut templ_id = String::new();
        if let Some(templ_ref) = templ_ref {
            if let Ok((z_prefix, z_strid, z_strid20, z_hash, z_display_name, )) =
                templs_query.get(templ_ref.0)
            {
                if prefix.is_none() {
                    prefix = z_prefix.map(|p| p.as_str());
                }
                if sid.is_none() {
                    sid = z_strid.map(|s| s.as_str());
                }
                if sid20.is_none() {
                    sid20 = z_strid20.map(|s| s.as_str());
                }
                if hash_id.is_none() {
                    hash_id = z_hash;
                }
                if display_name.is_none() {
                    display_name = z_display_name;
                }
            }
            templ_id = format!(" {:?}", templ_ref);
        }

        let prefix = prefix.unwrap_or("");
        let sid = sid.unwrap_or("");
        let sid20 = sid20.unwrap_or("");
        let hash_digits_buf = hash_id
            .map(|h| {
                let digits = h.as_u64().to_string();
                digits.chars().take(5).collect::<String>()
            })
            .unwrap_or_default();

        let mut new_name = String::with_capacity(
            prefix.len()
                + 1
                + sid.len()
                + sid20.len()
                + hash_digits_buf.len()
                + display_name.map(|d| d.0.len() + 2).unwrap_or(0)
                + templ_id.len(),
        );

        new_name.push_str(prefix);
        new_name.push(' ');
        new_name.push_str(sid);
        new_name.push_str(sid20);
        if !hash_digits_buf.is_empty() {
            new_name.push(' ');
            new_name.push_str(&hash_digits_buf);
        }
        if let Some(dn) = display_name {
            new_name.push(' ');
            new_name.push_str(dn.0.as_str());
        }
        if !templ_id.is_empty() {
            new_name.push_str(&templ_id);
        }

        if name.as_str() == new_name {
            continue;
        }
        name.set(new_name);
    }
}
