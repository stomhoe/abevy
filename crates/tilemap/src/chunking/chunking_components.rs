
use bevy::{ecs::{entity::EntityHashSet, system::SystemParam}, prelude::*};

use super::chunking_resources::AaChunkRangeSettings;
use crate::regioning::regioning_components::ChunksActiveInRegion;
use ::tilemap_shared::*;
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone)]
#[relationship(relationship_target = ChunksActiveInRegion)]
pub struct Chunk {
    #[relationship]
    pub region_ent: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MacroChunk;

#[derive(Component, Debug, Clone, Copy)]
pub struct MacroChunkRef(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
pub struct TerrgenDiscoveredMacroChunk;

#[derive(Component, Debug, Copy, Clone, )]
pub struct SaveTile {
    pub chunk_pos: ChunkPos,//NO HACE FALTA PORQ EL CHUNKPOS SE PUEDE CALCULAR A PARTIR DE GLOBAL POS
}

/*
           .replicate::<TilemapOf>()

*/

#[derive(Component, Debug, Default, Clone)]
pub struct TilesToSave(pub EntityHashSet);
impl TilesToSave { pub fn entities(&self) -> &EntityHashSet { &self.0 } }

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TerrGenState {
    #[default]
    Pending,
    Ready,
    OpsLaunched,
    Finished,
    Disabled,
}

#[derive(SystemParam)]
pub struct ChunkParamSet<'w, 's> {
    terrgen_states: Query<'w, 's, &'static TerrGenState, With<Chunk>>,
}

impl ChunkParamSet<'_, '_> {
    pub fn is_chunk_spawn_ready(&self, chunk_ent: Entity) -> bool {
        let Ok(terrgen_state) = self.terrgen_states.get(chunk_ent) else {
            return false;
        };
        *terrgen_state == TerrGenState::Finished
    }
}

impl TerrGenState {
    pub fn is_ready(&self) -> bool {
        *self == TerrGenState::Finished
    }
}



#[derive(Component, Debug, Clone)]
pub struct ActivatingChunks {
    pub reactivation_timer: Timer,
    pub entities: Vec<Entity>,
}

impl ActivatingChunks {
    pub fn new(chunkrange: &AaChunkRangeSettings) -> Self {
        Self {
            entities: Vec::with_capacity((chunkrange.approximate_number_of_chunks(1.2)) as usize),
            reactivation_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
    pub fn render_grid(&self, ui: &mut egui::Ui, ) {
        use bevy_inspector_egui::egui;

        let num_chunks = self.entities.len();
        ui.label(format!("Activating Chunks: {}", num_chunks));
        let grid_size = (num_chunks as f32).sqrt().ceil() as usize;
        let chunk_size = 50.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(
                chunk_size * grid_size as f32,
                chunk_size * grid_size as f32,
            ),
            egui::Sense::hover(),
        );

        for (i, entity) in self.entities.iter().enumerate() {
            let row = i / grid_size;
            let col = i % grid_size;
            let rect = egui::Rect::from_min_size(
            response.rect.min + egui::vec2(col as f32 * chunk_size, row as f32 * chunk_size),
            egui::vec2(chunk_size - 2.0, chunk_size - 2.0),
            );
            painter.rect_filled(rect, 0.0, egui::Color32::LIGHT_GREEN);
            painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            entity.index().to_string(),
            egui::FontId::proportional(14.0),
            egui::Color32::BLACK,
            );
        }
    }
}
