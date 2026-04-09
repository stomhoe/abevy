use bevy::math::U16Vec2;
#[allow(unused_imports)] use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_asset_loader::dynamic_asset::{DynamicAssetCollection, DynamicAssets};
use bevy_asset_loader::standard_dynamic_asset::StandardDynamicAsset;
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

#[derive(Resource, Default)]
pub struct ImageSizeMap(pub HashMap<AssetId<Image>, U16Vec2>);



#[derive(Resource, Debug, Default )]
pub struct GlobalEntityMap(pub HashMap<String, Entity>);
impl GlobalEntityMap {
    pub fn insert<S: Into<String>>(&mut self, id: S, entity: Entity) {
        self.0.insert(id.into(), entity);
    }

    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }

    #[allow(dead_code)]
    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct AppendedStandardDynamicAssetArray {
    pub assets_per_key: HashMap<String, Vec<StandardDynamicAsset>>,
}

impl DynamicAssetCollection for AppendedStandardDynamicAssetArray {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, assets) in &self.assets_per_key {
            dynamic_assets.register_asset(key, Box::new(assets.clone()));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeriAutoRoutingRule {
    pub dynamic_key: &'static str,
    pub suffix: &'static str,
}

fn auto_routing_rules() -> &'static Mutex<Vec<SeriAutoRoutingRule>> {
    static RULES: OnceLock<Mutex<Vec<SeriAutoRoutingRule>>> = OnceLock::new();
    RULES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn register_seri_auto_routing_rule(
    dynamic_key: &'static str,
    suffix: &'static str,
) {
    let Ok(mut rules) = auto_routing_rules().lock() else {
        warn!("Failed locking auto routing rules registry");
        return;
    };
    let rule = SeriAutoRoutingRule {
        dynamic_key,
        suffix,
    };
    if !rules.iter().any(|existing| existing == &rule) {
        rules.push(rule);
    }
}

pub fn register_seri_dynamic_asset_key(dynamic_assets: &mut DynamicAssets, dynamic_key: &str) {
    let manifest_map = load_seri_manifest_map();
    let mut assets_per_key: HashMap<String, Vec<StandardDynamicAsset>> = HashMap::default();
    assets_per_key.insert(
        dynamic_key.to_string(),
        manifest_map.get(dynamic_key).cloned().unwrap_or_default(),
    );
    AppendedStandardDynamicAssetArray { assets_per_key }.register(dynamic_assets);
}

fn load_seri_manifest_map() -> HashMap<String, Vec<StandardDynamicAsset>> {
    let rules = auto_routing_rules()
        .lock()
        .map(|rules| rules.clone())
        .unwrap_or_default();
    let mut merged: HashMap<String, Vec<StandardDynamicAsset>> = auto_discover_assets_by_rules(&rules);
    for manifest_path in find_seri_manifest_paths() {
        let parsed = match load_single_seri_manifest(&manifest_path) {
            Ok(parsed) => parsed,
            Err(err) => {
                warn!("{err:#}");
                continue;
            }
        };
        for (key, mut assets) in parsed {
            if assets.is_empty() {
                continue;
            }
            if key == "auto" || key == "seri.auto" {
                for asset in assets.drain(..) {
                    let Some(inferred_key) = infer_dynamic_key_from_asset(&asset, &rules) else {
                        warn!("Could not infer dynamic key from auto asset in '{}': {:?}", manifest_path, asset);
                        continue;
                    };
                    push_unique_asset(&mut merged, inferred_key, asset);
                }
                continue;
            }
            for asset in assets.drain(..) {
                push_unique_asset(&mut merged, &key, asset);
            }
        }
    }
    for assets in merged.values_mut() {
        sort_assets_for_precedence(assets);
    }
    merged
}

fn auto_discover_assets_by_rules(
    rules: &[SeriAutoRoutingRule],
) -> HashMap<String, Vec<StandardDynamicAsset>> {
    let assets_root = Path::new("assets");
    if !assets_root.exists() {
        return HashMap::default();
    }
    let mut merged: HashMap<String, Vec<StandardDynamicAsset>> = HashMap::default();
    let mut discovered: Vec<(String, String)> = Vec::new();
    let files = crate::def_db::discover_assets_files_matching(assets_root, |_| true)
        .unwrap_or_default();
    for (source, _) in files {
        let Some(dynamic_key) = infer_dynamic_key_from_path(&source.rel_path, rules) else {
            continue;
        };
        discovered.push((dynamic_key.to_string(), source.rel_path));
    }
    discovered.sort_by(|(key_a, path_a), (key_b, path_b)| {
        if key_a != key_b {
            return key_a.cmp(key_b);
        }
        let pri_a = path_precedence_rank(path_a);
        let pri_b = path_precedence_rank(path_b);
        pri_a.cmp(&pri_b).then(path_a.cmp(path_b))
    });
    for (dynamic_key, path) in discovered {
        push_unique_asset(
            &mut merged,
            &dynamic_key,
            StandardDynamicAsset::File { path },
        );
    }
    merged
}

fn push_unique_asset(
    merged: &mut HashMap<String, Vec<StandardDynamicAsset>>,
    dynamic_key: &str,
    asset: StandardDynamicAsset,
) {
    let out = merged.entry(dynamic_key.to_string()).or_default();
    if !out.iter().any(|existing| existing == &asset) {
        out.push(asset);
    }
}

fn sort_assets_for_precedence(assets: &mut [StandardDynamicAsset]) {
    assets.sort_by(|a, b| {
        let (rank_a, path_a) = asset_precedence_key(a);
        let (rank_b, path_b) = asset_precedence_key(b);
        rank_a.cmp(&rank_b).then(path_a.cmp(path_b))
    });
}

fn asset_precedence_key(asset: &StandardDynamicAsset) -> (u8, &str) {
    match asset {
        StandardDynamicAsset::File { path } => (path_precedence_rank(path), path.as_str()),
        _ => (1, ""),
    }
}

fn path_precedence_rank(path: &str) -> u8 {
    if path.starts_with("mods/") || path.contains("/mods/") {
        0
    } else {
        1
    }
}

fn infer_dynamic_key_from_asset(
    asset: &StandardDynamicAsset,
    rules: &[SeriAutoRoutingRule],
) -> Option<&'static str> {
    let StandardDynamicAsset::File { path } = asset else {
        return None;
    };
    infer_dynamic_key_from_path(path, rules)
}

fn infer_dynamic_key_from_path(path: &str, rules: &[SeriAutoRoutingRule]) -> Option<&'static str> {
    let mut best: Option<(&SeriAutoRoutingRule, usize)> = None;
    for rule in rules {
        if !path.ends_with(rule.suffix) {
            continue;
        }
        let score = rule.suffix.len();
        if let Some((_, best_score)) = best {
            if score <= best_score {
                continue;
            }
        }
        best = Some((rule, score));
    }
    best.map(|(rule, _)| rule.dynamic_key)
}

fn load_single_seri_manifest(path: &str) -> Result<HashMap<String, Vec<StandardDynamicAsset>>> {
    let full_path = format!("assets/{path}");
    let content = std::fs::read_to_string(&full_path)
        .with_context(|| format!("Failed reading seri manifest at '{path}'"))?;
    ron::from_str::<HashMap<String, Vec<StandardDynamicAsset>>>(&content)
        .with_context(|| format!("Failed parsing seri manifest RON at '{path}'"))
}

fn find_seri_manifest_paths() -> Vec<String> {
    let assets_root = Path::new("assets");
    if !assets_root.exists() {
        return Vec::new();
    }
    let mut out: Vec<String> = crate::def_db::discover_assets_files_by_suffixes(&[".seri_manifest.ron"])
        .unwrap_or_default()
        .into_iter()
        .map(|(source, _)| source.rel_path)
        .collect();
    out.sort();
    out
}
