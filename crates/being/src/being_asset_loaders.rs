use std::path::Path;

use bevy::prelude::*;
use common::{def_db, log_targets::{BEING_TEMPLATE_INIT, RACE_INIT}};
use serde::Deserialize;

use crate::being_def_parser::parse_typed_def;
use crate::pack::pack_seris::PackSeri;
use being_shared::{BitSeri, RaceSeri};

enum LoaderTarget {
    Race,
    Bit,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RaceAssetSeri {
    #[serde(flatten)]
    pub race: RaceSeri,
    #[serde(default)]
    pub pack: Option<common::def_db::DefValue>,
}

fn load_custom_defs<T: for<'de> Deserialize<'de>>(
    def_type: &str,
    target: LoaderTarget,
    suffixes: &[&str],
) -> Vec<(String, T)> {
    let Ok(mut discovered) = def_db::discover_assets_files_by_suffixes(suffixes) else {
        match target {
            LoaderTarget::Race => error!(target: RACE_INIT, "Failed discovering {} defs", def_type),
            LoaderTarget::Bit => error!(target: BEING_TEMPLATE_INIT, "Failed discovering {} defs", def_type),
        }
        return Vec::new();
    };
    discovered.sort_by(|(a, _), (b, _)| {
        a.precedence_rank()
            .cmp(&b.precedence_rank())
            .then_with(|| a.rel_path.cmp(&b.rel_path))
    });

    let mut out = Vec::with_capacity(discovered.len());
    for (source, abs_path) in discovered {
        let Ok(content) = std::fs::read_to_string(&abs_path) else {
            match target {
                LoaderTarget::Race => error!(target: RACE_INIT, "Failed reading {} file '{}'", def_type, source.rel_path),
                LoaderTarget::Bit => error!(target: BEING_TEMPLATE_INIT, "Failed reading {} file '{}'", def_type, source.rel_path),
            }
            continue;
        };
        let Ok(parsed) = parse_typed_def::<T>(&content) else {
            match target {
                LoaderTarget::Race => error!(target: RACE_INIT, "Failed parsing {} file '{}'", def_type, source.rel_path),
                LoaderTarget::Bit => error!(target: BEING_TEMPLATE_INIT, "Failed parsing {} file '{}'", def_type, source.rel_path),
            }
            continue;
        };
        out.push((abs_path.to_string_lossy().into_owned(), parsed));
    }
    out
}

pub(crate) fn load_race_asset_seri_defs() -> Vec<RaceAssetSeri> {
    load_custom_defs::<RaceAssetSeri>(
        "RaceSeri",
        LoaderTarget::Race,
        &[".race"],
    )
    .into_iter()
    .map(|(_, value)| value)
    .collect()
}

pub(crate) fn load_bit_asset_seri_defs() -> Vec<BitSeri> {
    load_custom_defs::<BitSeri>(
        "BitSeri",
        LoaderTarget::Bit,
        &[".bit"],
    )
    .into_iter()
    .map(|(_, value)| value)
    .collect()
}

pub(crate) fn parse_inline_pack_def(
    pack_value: &common::def_db::DefValue,
    default_id: Option<&str>,
    path: &Path,
) -> Result<PackSeri, String> {
    let mut pack_seri = crate::pack::pack_seris::parse_pack_seri_value(pack_value, default_id, path)?;
    let Some(default_id) = default_id else {
        return Ok(pack_seri);
    };
    let default_id = default_id.trim();
    if default_id.is_empty() {
        return Ok(pack_seri);
    }
    pack_seri
        .ids
        .entry(default_id.to_string())
        .or_insert_with(|| crate::pack::pack_seris::PackMemberConfigSeri {
            race_first: true,
            ..crate::pack::pack_seris::PackMemberConfigSeri::default()
        });
    Ok(pack_seri)
}
