/// Central location for all logging target constants.
/// Naming convention for string values:
/// - External libraries: Keep original names (e.g., naga, bevy_egui)
/// - Initialization: `<name>_init`
/// - Building: `<name>_build`
/// - Loading: `<name>_load`
/// - Processing: `<name>_process`
/// - Runtime systems: `<name>_system`

// ============================================================================
// EXTERNAL LIBRARIES (Third-party crates - keep original names)
// ============================================================================
pub const NAGA: &str = "naga";
pub const WGPU_HAL: &str = "wgpu_hal";
pub const WGPU_CORE: &str = "wgpu_core";
pub const BEVY_ECS_TILEMAP: &str = "bevy_ecs_tilemap";
pub const BEVY_ECS_RELATIONSHIP: &str = "bevy_ecs_relationship";
pub const BEVY_EGUI: &str = "bevy_egui";
pub const BEVY_REPLICON: &str = "bevy_replicon";
pub const BEVY_RENDER: &str = "bevy_render";
pub const BEVY_APP: &str = "bevy_app";
pub const COSMIC_TEXT: &str = "cosmic_text";
pub const OFFSET_ALLOCATOR: &str = "offset_allocator";
pub const BEVY_ASSET_LOADER: &str = "bevy_asset_loader";
pub const CALLOOP_LOOP_LOGIC: &str = "calloop_loop_logic";

// ============================================================================
// ASSET & RESOURCE LOADING (format: <name>_load)
// ============================================================================
pub const ASSET_LOADING: &str = "asset_loading";
pub const TILEMAP_LOAD: &str = "tilemap_load";
pub const DIMENSION_LOADING: &str = "dimension_loading";
pub const DEF_VALIDATION: &str = "def_validation";

// ============================================================================
// INITIALIZATION SYSTEMS (format: <name>_init)
// ============================================================================
pub const RACE_INIT: &str = "race_init";
pub const SPRITE_INIT: &str = "sprite_init";
pub const SPRITE_ANIMATION_INIT: &str = "sprite_animation_init";
pub const TILE_INIT: &str = "tile_init";
pub const TERRGEN_INIT: &str = "terrgen_init";
pub const BIOME_INIT: &str = "biome_init";
pub const OPLIST_INIT: &str = "oplist_init";
pub const SGC_INIT: &str = "sgc_init";
pub const PORTAL_INIT: &str = "portal_init";
pub const TERRPROBE_INIT: &str = "terrprobe_init";
pub const BEING_TEMPLATE_INIT: &str = "being_template_init";
pub const COLOR_SAMPLER_INIT: &str = "color_sampler_init";
pub const CHILDRENSPRITE_INIT: &str = "childrensprite_init";
pub const TILE_SHADER_INIT: &str = "tile_shader_init";
pub const GAME_INIT: &str = "game_init";

// ============================================================================
// BUILD & CONFIGURATION SYSTEMS (format: <name>_build)
// ============================================================================
pub const SPRITE_BUILD: &str = "sprite_build";
pub const BEING_TEMPLATE_BUILD: &str = "being_template_build";
pub const BODY_BUILD: &str = "body_build";
pub const BODY_TEMPL_INIT: &str = "body_templ_init";
pub const GAME_COMMON_SYSTEM: &str = "game_common_system";

// ============================================================================
// RUNTIME SYSTEMS - SPRITE & ANIMATION (format: <name>_system)
// ============================================================================
pub const SPRITE_SYSTEM: &str = "sprite_system";
pub const SPRITE_SAMPLER_SYSTEM: &str = "sprite_sampler_system";
pub const SPRITE_ANIMATION_SYSTEM: &str = "sprite_animation_system";

// ============================================================================
// RUNTIME SYSTEMS - TERRAIN & STRUCTURE GENERATION (format: <name>_process)
// ============================================================================
pub const TERRGEN_PROCESS: &str = "terrgen_process";
pub const TERRGEN_SYSTEM: &str = "terrgen_system";
pub const STRUCTURE_SPAWN: &str = "structure_spawn";

// ============================================================================
// RUNTIME SYSTEMS - TILEMAP & CHUNKING (format: <name>_system)
// ============================================================================
pub const TILEMAP_SYSTEM: &str = "tilemap_system";
pub const CHUNK_DESPAWN: &str = "chunk_despawn";
pub const CHUNK_VISIBILITY: &str = "chunk_visibility";
pub const CHUNK_ACTIVATION: &str = "chunk_activation";
pub const GPOS_MAP: &str = "gpos_map";

// ============================================================================
// RUNTIME SYSTEMS - REGIONING & STRUCTURE (format: <name>_system)
// ============================================================================
pub const SGC_CHUNK_OFFER: &str = "sgc_chunk_offer";
pub const SGC_CHUNK_CLAIM: &str = "sgc_chunk_claim";
pub const REGION_SYSTEM: &str = "region_system";
pub const RIVER_SYSTEM: &str = "river_system";

// ============================================================================
// RUNTIME SYSTEMS - GAMEPLAY (format: <name>_system)
// ============================================================================
pub const BEING_SYSTEM: &str = "being_system";
pub const WANDER_SYSTEM: &str = "wander_system";
pub const BODY_HP_SYSTEM: &str = "body_hp_system";
pub const BEING_MELEE_DEBUG: &str = "being_melee_debug";
pub const MOVEMENT_SYSTEM: &str = "movement_system";
pub const FACTION_SYSTEM: &str = "faction_system";
pub const DUNGEONING_SYSTEM: &str = "dungeoning_system";
pub const ENTITY_ZERO_SYSTEM: &str = "entity_zero_system";
pub const BEING_CONTROL: &str = "being_control";
pub const ENTITY_MAP_SYSTEM: &str = "entity_map_system";
pub const ITEM_SYSTEM: &str = "item_system";
pub const WILDLIFE_SYSTEM: &str = "wildlife_system";


// ============================================================================
// UTILITIES & SEARCH
// ============================================================================
pub const POSITION_SEARCH: &str = "position_search";
pub const DEBUG: &str = "debug";
pub const CONTROL: &str = "control";
pub const INSPECTOR: &str = "inspector";

// ============================================================================
// DEBUG MARKERS (usually disabled)
// ============================================================================
pub const DEBUG_TILE: &str = "debug_tile";
