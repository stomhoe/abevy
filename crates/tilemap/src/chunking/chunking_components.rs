use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, }, prelude::*};

use super::chunking_resources::AaChunkRangeSettings;
use crate::regioning::regioning_components::ChunksActiveInRegion;
use ::tilemap_shared::*;
use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};



#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, Reflect, )]
#[relationship(relationship_target = ChunksActiveInRegion)]
pub struct Chunk {
    #[relationship]
    pub region_ent: Entity,
}


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct SaveTile {
    pub chunk_pos: ChunkPos,//NO HACE FALTA PORQ EL CHUNKPOS SE PUEDE CALCULAR A PARTIR DE GLOBAL POS
}



#[derive(Component, Debug, Reflect, Default,)]
pub struct TilesToSave(pub EntityHashSet);
impl TilesToSave { pub fn entities(&self) -> &EntityHashSet { &self.0 } }


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct TerrGenOpsLaunched;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct ReadyForTerrgen;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct TerrGenDisabled;

#[derive(Component, Debug, Reflect)]
pub struct ChunkDespawnTimer(pub Timer);

impl ChunkDespawnTimer {
    pub fn new() -> Self {
        Self(Timer::from_seconds(20.0, TimerMode::Once))
    }
}

#[derive(Component, Debug, Reflect)]
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

impl InspectorPrimitive for ActivatingChunks {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        self.render_grid(ui, );
        false
    }
    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        self.render_grid(ui, );
    }
}
