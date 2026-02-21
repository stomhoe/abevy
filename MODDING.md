# Modding Guide (Current Architecture)

This project is data-driven. Most content is loaded from `assets/**/*.ron` at startup, then turned into ECS entities/resources.

There are **two modding layers**:

1. **Seri manifests + `*Seri` assets** (`*.seri_manifest.ron` + files like `*.tile.ron`, `*.race.ron`, `*.tpt.ron`, etc.)
2. **Def DB patching** (`*.defpatch.ron`) for explicit patch operations on loaded defs

## 1) Startup Flow (High Level)

1. The game scans `assets/` for all `.seri_manifest.ron` files.
2. It also auto-discovers files by registered suffix rules (from `define_entity_map_systems!`).
3. Dynamic asset keys like `seri.tilemap.tile` are populated with `File(path: "...")` entries.
4. Systems call generated loaders like `load_tile_seri_defs()`, `load_race_seri_defs()`, etc.
5. Those `*Seri` structs are validated/transformed into runtime ECS components.

Separately, systems using `DefDatabase<T>`:

1. Scan matching suffixes under `assets/`.
2. Merge base/mod defs by precedence.
3. Apply all `*.defpatch.ron` operations.
4. Optionally run cross-reference validation rules.

## 2) Seri Manifest Modding

Create a manifest anywhere under `assets/` ending with `.seri_manifest.ron`.

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

You can also use `"auto"` / `"seri.auto"` and let the engine infer the key from suffix:

```ron
{
  "auto": [
    File(path: "mods/my_mod/ron/being/race/my_race.race.ron"),
    File(path: "mods/my_mod/ron/tilemap/tiling/tile/my_tile.tile.ron"),
  ],
}
```

Recommended layout:

- `assets/mods/<mod_name>/<mod_name>.seri_manifest.ron`
- `assets/mods/<mod_name>/ron/...`

## 3) Dynamic Keys In Use

Currently registered keys include:

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

## 4) Override / Precedence Rules

### For manifest + auto-discovered dynamic assets

- Assets are deduplicated by exact dynamic entry.
- Paths under `mods/` (or containing `/mods/`) are ranked before non-mod paths in the dynamic list.
- Final behavior on same in-game `id` depends on the consuming init system/map insertion semantics for that type.

### For `DefDatabase<T>`

- Base files load first, mod files load after (`mods/...` wins on same `id`).
- Then `.defpatch.ron` operations are applied.
- Patch files are discovered globally under `assets/` and processed in deterministic path order.

## 5) Def Patching (`*.defpatch.ron`)

Patch files can live anywhere under `assets/` and support:

- `upsert`
- `delete`
- `set_field`
- `remove_field`
- `merge`
- `copy`

Example:

```ron
[
  (
    op: "set_field",
    type: "TileSeri",
    id: "ocean",
    path: "walk_speed",
    value: 0.2,
  ),
  (
    op: "upsert",
    type: "RaceSeri",
    id: "my_custom_race",
    value: (
      id: "my_custom_race",
      name: "My Race",
      body_tree_or_sampler: "human",
      sexes: {"male": (100, [])},
      fallback_sprites_to_sample: [],
    ),
  ),
]
```

Path syntax supports dot + index style, for example:

- `stats.hp`
- `drops[0].id`

`merge` behavior:

- map + map: recursive merge
- seq + seq: append
- otherwise: replace target value

## 6) Operation Lists: `.ron` and `.tg`

Terrain operation lists can come from:

- regular `*.oplist.ron` seri files
- TG scripts under `assets/ron/tilemap/terrgen/oplist_scripts/**/*.oplist.tg`

At init, both sources are loaded and combined.

## 7) Validation / Introspection APIs

Useful runtime APIs in `common::def_db`:

- `global_registry_snapshot()`
- `resolve_def_ref(type_name, id)`
- `resolve_def_field(type_name, id, path)`
- `DefDatabase::<T>::resolve_typed_ref(type_name, id)`

Cross-def validation rules can be registered with:

- `register_ref_rule(DefRefRule { ... })`

Validation behavior is controlled by `DefValidationConfig { enabled, fail_fast }`.

## 8) Practical Mod Workflow

1. Add your mod files under `assets/mods/<mod_name>/...`.
2. Add a `<mod_name>.seri_manifest.ron` (or rely on auto discovery if paths/suffixes already match).
3. Keep unique `id`s unless intentionally overriding existing defs.
4. Use `.defpatch.ron` when you want surgical edits instead of full file replacement.
5. Start game, watch logs for parse/validation warnings.

## 9) Current Limitations / Notes

- There is no dedicated dependency graph between mods yet; ordering is path-based and deterministic.
- Some systems are strict and skip invalid entries rather than repairing them.
- If two mods override the same `id`, final winner is determined by the loader/patch order for that pipeline.

## 10) Base Content Reference

Core shipped manifest is at:

- `assets/ron/base.seri_manifest.ron`

Prefer adding new manifests/files under `assets/mods/` instead of editing base files.
