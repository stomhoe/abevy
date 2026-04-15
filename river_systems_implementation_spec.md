# River Systems Implementation Spec

## What To Implement

### Goal
Implement deterministic river generation in the existing river systems so that:

1. Reloading the same region always produces the exact same river claims and river tiles.
2. Neighboring regions generate seamless rivers across borders, regardless of load order.
3. River shape looks natural: one dominant trunk, sparse tributaries, smooth direction changes, and no spider-web pattern.
4. Small islands do not get rivers.
5. Rivers are configured through SGC args, with phase 1 focusing on the essential subset.
6. River claiming is handled in a dedicated pre-claim path before the normal structure claim pipeline.
7. Use the dedicated ocean probe to tell where ocean and non-land tiles are.

### Existing Systems To Reuse

Do not redesign the pipeline. Build on the existing regioning and terrprobe infrastructure:

- Claim entry point: [crates/tilemap/src/regioning/natural/river_systems.rs](crates/tilemap/src/regioning/natural/river_systems.rs)
- Claim/build pipeline: [crates/tilemap/src/regioning/regioning.rs](crates/tilemap/src/regioning/regioning.rs)
- Messages: [crates/tilemap/src/regioning/regioning_messages.rs](crates/tilemap/src/regioning/regioning_messages.rs)
- Claim state: [crates/tilemap/src/regioning/regioning_components.rs](crates/tilemap/src/regioning/regioning_components.rs)
- River debug state: [crates/tilemap/src/regioning/natural/river_components.rs](crates/tilemap/src/regioning/natural/river_components.rs)
- Terrain probe messages: [crates/tilemap/src/terrain/terrprobe/terrprobe_messages.rs](crates/tilemap/src/terrain/terrprobe/terrprobe_messages.rs)
- Region probe pattern: [crates/tilemap/src/terrain/terrprobe/terrprobe_pattern_region.rs](crates/tilemap/src/terrain/terrprobe/terrprobe_pattern_region.rs)
- River probe template: [assets/ron/tilemap/terrgen/probe/river_land_all.tpt.ron](assets/ron/tilemap/terrgen/probe/river_land_all.tpt.ron)
- Ocean probe template: [assets/ron/tilemap/terrgen/probe/river_ocean.tpt.ron](assets/ron/tilemap/terrgen/probe/river_ocean.tpt.ron)
- River SGC config: [assets/ron/tilemap/region/structures/river.sgc](assets/ron/tilemap/region/structures/river.sgc)

### Required Behavioral Rules

1. Exact border continuity is required.
2. Each region claims only its own in-region chunks.
3. Rivers should always attempt generation in eligible regions.
4. Tributaries must stay sparse and must merge into the main trunk only.
5. Tributaries must remain narrower than the trunk.
6. Use smooth tapering so the river does not look blocky or polygonal.
7. Reroute is allowed only if it is cheap and deterministic: at most 4 detour steps and 1 retry.
8. If reroute budget is exceeded, skip the river.
9. If a strict mouth cannot be found, use the nearest low-inlandness fallback terminal.
10. Use the ocean probe to identify ocean / non-land tiles.
11. Use river_region_probe for inlandness sampling and river_ocean_probe for ocean detection in phase 1.
12. Terrgen suppression is controlled only by disable_terrgen args.

### Determinism Contract

Determinism must come from stable inputs only:

- Global generation seed
- Dimension hash
- Region position or chunk position
- Fixed salt values for each stage

Do not use runtime order, current frame count, entity creation order, or unseeded randomness in any generation decision.

All hash-based decisions must be stable across reloads. Any iteration over maps or sets that affects generation must be sorted first.

### Phase 1: Claim-Stage River Logic

The claim system in river_systems.rs must:

1. Read OfferChunk messages.
2. Read SampledValuesCollected messages for completed probe results.
3. Resolve the river probe template via TerrProbeTemplEntityMap.
4. Start a deterministic terrprobe request for each river offer.
5. Store pending river offer state in locals keyed by requester.
6. Reconstruct the inlandness field from the sampled matrix.
7. Use the ocean probe results when building the land/water picture.
8. Reject small islands using river_min_island_area_chunks.
9. Select deterministic source candidates from high inlandness areas.
10. Select deterministic mouth candidates near ocean or near-ocean tiles, using the ocean probe as the boundary signal.
11. Generate one trunk plus sparse tributaries.
12. Apply the cheap reroute rule if blocked.
13. Convert the final river path into claimed chunk positions.
14. Emit ChunksClaim for in-region chunks only.
15. Mark unsupported/non-river offers as skipped so ClaimList does not have holes.

### Phase 2: Build-Stage River Logic

The build system in river_systems.rs must:

1. Read SgcPrepareTilesOrder messages.
2. Filter to river structure orders only.
3. Rebuild the same deterministic river geometry from the same rules used in phase 1.
4. Resolve river_tile_id to a TileRef.
5. Emit tile placements chunk by chunk in stable order.
6. Build TerrGenDisabledGposForChunks only when disable_terrgen rules say so.
7. Emit StructureBuildCompliance with the correct chunk payloads.
8. Update RiverDebugData with success/failure and tile/source/mouth tracking.
9. Use the ocean probe to define the ocean boundary when selecting mouths and terminals.

### Config Scope To Implement Now

Implement these now:

1. river_tile_id
2. river_min_island_area_chunks
3. river_source_min_inlandness
4. river_source_hash_stride
5. river_source_mouth_min_distance
6. river_source_min_separation_steps
7. river_max_sources
8. river_mouth_max_inlandness
9. river_worm_length
10. river_trace_neighbor_radius
11. river_directional_inertia
12. river_downhill_weight
13. river_uphill_penalty
14. river_main_half_width_start
15. river_main_half_width_end

Do not require extra SGC probe overrides in phase 1. Sampling must use the river_region_probe template only.

### Aesthetic Requirements

The river should look like a natural drainage system:

1. One dominant main river.
2. Few tributaries.
3. Tributaries merge into the main trunk, not into each other.
4. Width increases downstream on the trunk.
5. Tributaries are narrower and taper toward the merge point.
6. Direction changes should be smooth and biased by inertia, downhill flow, and local terrain.
7. The final macro shape should not resemble a dense web or random worm cloud.

### Failure Handling

Fail cleanly when any of the following happen:

1. Missing global settings.
2. Missing river probe template.
3. Missing river tile template.
4. No valid river source candidate.
5. No valid mouth candidate and no acceptable fallback.
6. Island is too small.
7. River path is blocked and reroute budget is exhausted.
8. River geometry produces no in-region chunks.

In failure cases, do not leave behind partially valid claim state that can confuse the normal region claim pipeline.

### Logging And Debugging

Use the river_system log target for bounded, high-signal logs only:

1. Probe start and completion.
2. Source and mouth selection counts.
3. Claim emission counts.
4. Build emission counts.
5. Failure reasons.

Update RiverDebugData with:

1. Active probes.
2. Claimed chunks.
3. Failed chunks.
4. Generated river tiles.
5. Source points.
6. Mouth points.
7. Sampled values.
8. Success and failure counts.

### Verification Checklist

The implementation should pass these checks:

1. Same region reload produces identical rivers.
2. Neighboring regions produce seamless border rivers.
3. Small islands produce no rivers.
4. River shape stays trunk-dominant and sparse.
5. Claim pipeline does not produce holes.
6. Build pipeline finishes without timeouts.
7. Terrgen suppression only happens when configured.

## How To Implement It


### 2. Build The Claim System In Deterministic Stages

1. Read OfferChunk messages once.
2. Reject non-river offers early, but remember to mark their claim index as skipped.
3. For river offers, resolve the region dimension and river probe template.
4. Create a deterministic requester entity per offer if needed.
5. Send a TerrProbeJob using the river_region_probe template.
6. Store the pending offer state in a local buffer keyed by requester.
7. When SampledValuesCollected arrives, match it to the pending requester.
8. Convert the sampled matrix into a stable inlandness lookup.
9. Compute connected landmass size and reject small islands.
10. Pick source points from high inlandness cells using stable ranking.
11. Pick mouth points from low inlandness / ocean-adjacent cells, confirmed by the ocean probe.
12. Build a main trunk first.
13. Add only sparse tributaries, and only allow them to merge into the trunk.
14. If a path is blocked, try a very small deterministic reroute.
15. If reroute fails, skip the whole river.
16. Turn the final path into chunk positions inside the current region only.
17. Sort the final chunk list deterministically.
18. Emit ChunksClaim.

### 3. Build The River Geometry With Stable Ordering

1. Use the same deterministic inputs in the build system that were used for claiming.
2. Recompute the same trunk and tributaries.
3. Never depend on claim ordering or message arrival order.
4. Use stable tie-break rules when multiple candidate tiles score the same.
5. Keep river widths smooth and narrower for tributaries.
6. Generate the final global tile list in a sorted, repeatable order.

### 4. Keep Claim And Build Symmetric

The claim system and build system must agree on the same:

1. Source selection rules.
2. Mouth selection rules.
3. Direction change rules.
4. Width profile rules.
5. Merge rules.
6. Reroute fallback rules.

If these diverge, reload determinism will break.

### 5. Use Stable Tie-Breakers Everywhere

When two candidates are otherwise equal, break ties by a deterministic order such as:

1. Higher inlandness first.
2. Lower distance to mouth or source target.
3. Lower y.
4. Lower x.

Do not rely on hash map iteration order or insertion order.

### 6. Respect The Cheap Reroute Limit

If a river step is blocked:

1. Try a tiny alternate route.
2. Stop after at most 4 detour steps and 1 retry.
3. If that fails, skip the whole river.

Do not recursively reroute until the output becomes messy.

### 7. Emit Correct Build Compliance

1. Resolve river_tile_id to a TileRef.
2. For each chunk, collect river tiles in stable order.
3. Build TerrGenBlockedGposMask only when disable_terrgen says to do so.
4. Put chunk tile data and suppression masks into StructureBuildCompliance.

### 8. Keep Debug State Honest

Update RiverDebugData at the important points:

1. When a probe starts.
2. When a probe completes.
3. When a source is selected.
4. When a mouth is selected.
5. When chunks are claimed.
6. When generation fails.

If you do not have enough debug state, add only the minimum extra fields needed.



 Do not let river logic accidentally depend on unrelated structure generation state.

