
use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, tiles::TileStorage};
use camera::camera_components::CameraTarget;
use tilemap_shared::DimensionRef;
use tilemap_shared::ChunkPos;

use super::chunking_components::*;
use super::chunking_resources::*;


#[allow(unused_parens)]
pub fn update_chunk_visib(
    mut reader: MessageReader<RecheckChunksVisibility>,
    camera_query: Query<(&GlobalTransform, &DimensionRef), With<CameraTarget>>,
    mut chunks_query: Query<(&mut Visibility, &ChunkPos, &DimensionRef, &Children), With<Chunk>>,
    chunkrange_settings: Res<AaChunkRangeSettings>,
    mut event_writer: MessageWriter<DrawTilemap>,
    mut to_draw: Local<Vec<DrawTilemap>>,
) {
    if reader.is_empty() {
        return;
    }
    to_draw.reserve(reader.read().size_hint().0);
    reader.clear();
    if camera_query.is_empty() {
        return;
    }
    let Ok((camera_transform, camera_dimension)) = camera_query.single() else {
        error!("Failed to get singular camera target");
        return;
    };

    let camera_chunk_pos = ChunkPos::from(camera_transform.translation().xy());

    chunks_query.iter_mut().for_each(|(mut visibility, &chunk_pos, &chunk_dimension, children)| {

        let different_dimension = camera_dimension != &chunk_dimension;
        let out_of_visible = chunkrange_settings.out_of_visible_range(camera_transform, chunk_pos);
        let out_of_discovery = chunkrange_settings.out_of_discovery_range(camera_chunk_pos, chunk_pos);

        if different_dimension || (out_of_visible && out_of_discovery) {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        } else  {
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
                to_draw.extend(children.iter().map(|child| DrawTilemap(child)));
                //todo chequear q el tilemap tenga todos sus compoentes, sino mandar a una queue
            }
        }
    });
    event_writer.write_batch(to_draw.drain(..));
}

#[derive(Message, Debug, Clone, )]
pub struct RecheckChunksVisibility;

#[allow(unused_parens)]
pub fn detect_camera_change_pos_visib(
    camera_target: Query<&CameraTarget, Or<(Changed<GlobalTransform>, Added<CameraTarget>, Changed<DimensionRef>)>>,
    mut recheck_writer: MessageWriter<RecheckChunksVisibility>,
) {
    if camera_target.is_empty() {
        return;
    }
    let Ok(_) = camera_target.single() else {
        error!("More than one active camera target");
        return;
    };
    recheck_writer.write(RecheckChunksVisibility);
    trace!(target: "chunk_visibility", "Camera position or dimension changed, rechecking chunk visibility.");
}

pub fn periodically_recheck_chunk_visibility(
    mut recheck_writer: MessageWriter<RecheckChunksVisibility>,
) {
    recheck_writer.write(RecheckChunksVisibility);
    trace!(target: "chunk_visibility", "Rechecking chunk visibility due to timer.");
}
