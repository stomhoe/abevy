# Modding Guide (Manifest-Driven Dynamic Assets)

This project loads gameplay `*Seri` data through dynamic-asset manifests.

## Quick Start

1. Create a manifest file anywhere under `assets/` ending with `.seri_manifest.ron`.
2. Add one or more dynamic keys, each mapped to an array of dynamic assets.
3. Use `File(path: "...")` entries for `*Seri` files.
4. Place your mod's actual `.ron` files under `assets/` at the paths referenced by the manifest.

Example:

```ron
{
  "seri.being.race": [
    File(path: "mods/my_mod/ron/being/race/my_race.race.ron"),
  ],
  "seri.tilemap.tile": [
    File(path: "mods/my_mod/ron/tilemap/tiling/tile/my_tile.tile.ron"),
  ],
}
```

### Automatic Discovery

You do not need to list every file in a manifest.

At load time, the game scans `assets/` and auto-routes matching `*.presuffix.ron` files to dynamic keys using the rules registered by `define_entity_map_systems!` (`ron_dir` + `ron_suffix`).

If your file sits in the expected directory pattern and has the expected suffix, it is discovered automatically.

### Optional `auto` Manifest

You can still use `auto` (or `seri.auto`) and let the loader infer the target `seri.*` key from file suffix/path:

```ron
{
  "auto": [
    File(path: "mods/my_mod/ron/being/race/my_race.race.ron"),
    File(path: "mods/my_mod/ron/tilemap/tiling/tile/my_tile.tile.ron"),
  ],
}
```

Inference is not hardcoded in docs; it is derived from the rules registered by each `define_entity_map_systems!` declaration (`dynamic_key`, `ron_dir`, `ron_suffix`).  
That means if you rename a suffix (for example `sampler.ron` -> `bosampler.ron`) in the macro call, auto inference follows automatically.

Suggested layout:

- `assets/mods/<mod_name>/<mod_name>.seri_manifest.ron`
- `assets/mods/<mod_name>/ron/...`

## Available Dynamic Keys

- `seri.dimension`
- `seri.sprite.animation`
- `seri.sprite.weighted_sampler`
- `seri.sprite.config`
- `seri.tilemap.region.sgc`
- `seri.tilemap.terrprobe`
- `seri.tilemap.operation_list`
- `seri.tilemap.opfilter`
- `seri.tilemap.terrgen.noise`
- `seri.tilemap.tile_shader.repeat_tex`
- `seri.tilemap.tile_shader.wavy`
- `seri.tilemap.tile_shader.rocky`
- `seri.tilemap.tile`
- `seri.tilemap.tile.weighted_sampler`
- `seri.color_sampler`
- `seri.being.sex`
- `seri.being.bit`
- `seri.being.race`
- `seri.being.body.tree`
- `seri.being.body.sampler`
- `seri.being.body.part`

## Merge and Append Behavior

- The loader scans all `assets/**/*.seri_manifest.ron` files.
- Auto-discovered and manifest-provided `File(path: \"...\")` entries are ordered deterministically.
- Mod paths (`mods/...` or containing `/mods/`) are given higher priority and loaded first.
- Entries are deduplicated by exact dynamic asset entry.
- If two files define entities with the same in-game `id`, this ordering makes mod content win deterministically with current entity-map collision behavior.

## Base Content

Core game content is declared in:

- `assets/ron/base.seri_manifest.ron`

Mods should add separate manifest files instead of editing the base manifest.

## Notes

- You can use any `StandardDynamicAsset` variant supported by `bevy_asset_loader`, but `File(path: "...")` is the intended one for `*Seri` data.
- Keep paths relative to `assets/`.
- Broken or invalid manifests are skipped with warnings at load time.

## Explicit Def Patching

You can now patch defs explicitly with `*.defpatch.ron` files anywhere under `assets/`.

Patch files are loaded deterministically and applied after base+mod def merge.

Supported operations:

- `upsert`: create/replace whole def
- `delete`: remove a def by id
- `set_field`: set a nested field by path (dot + `[index]`)
- `remove_field`: remove nested field/key by path
- `merge`: recursive map merge, seq append
- `copy`: clone one def id into another

Example:

```ron
[
  (
    op: "set_field",
    type: "TileSeri",
    id: "ocean",
    path: "walk_speed",
    value: Some(0.2),
  ),
  (
    op: "upsert",
    type: "RaceSeri",
    id: "my_custom_race",
    value: (
      id: "my_custom_race",
      name: "My Race",
      body_tree: "human",
      sexes: {"male": (100, [])},
      fallback_sprites_to_sample: [],
    ),
  ),
]
```

## Global Registry API

Global registry/cross-ref helpers live in `common::def_db`:

- `global_registry_snapshot()`
- `resolve_def_ref(type_name, id)`
- `resolve_def_field(type_name, id, path)`
- `DefDatabase::<T>::resolve_typed_ref(type_name, id)`

## Startup Def Validation

Validation runs once during `AssetLoading::SpawnReplicatedEntities` (server/disconnected side), after all expected def types are loaded.

Register rules in code:

```rust
common::def_db::register_ref_rule(common::def_db::DefRefRule {
    from_type: "TileSeri".to_string(),
    from_path: "shader".to_string(),
    to_type: "ShaderRepeatTexSeri".to_string(),
    allow_missing: false,
});
```

Config resource:

- `DefValidationConfig { enabled, fail_fast }`
- default is `enabled = true`, `fail_fast = true` (panic on validation errors)
