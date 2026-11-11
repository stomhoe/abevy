use bevy::{asset::AsAssetId, prelude::*, render::{RenderApp, extract_component::ExtractComponent}};

/// A light occluder component. Should be used alongside a Mesh2d
#[derive(Component, Clone, Debug, Reflect, ExtractComponent, FromTemplate)]
#[extract_app(RenderApp)]
pub struct LightOccluder2d {
    /// Any texture with a transparent background. The occluder will take it's shape.
    pub occluder_mask: Handle<Image>,
    /// World-space height used to project directional shadows.
    pub height: f32,
}

impl Default for LightOccluder2d {
    fn default() -> Self {
        Self { occluder_mask: Handle::default(), height: 32.0 }
    }
}

impl LightOccluder2d {
    /// Creates a new [`LightOccluder2d`] with an occlusion mask
    pub fn new(occluder_mask: Handle<Image>) -> Self {
        Self { occluder_mask, ..Default::default() }
    }

    /// Creates a light occluder with an explicit world-space height.
    pub fn with_height(occluder_mask: Handle<Image>, height: f32) -> Self {
        Self { occluder_mask, height }
    }
}

impl From<LightOccluder2d> for AssetId<Image> {
    fn from(material: LightOccluder2d) -> Self {
        material.occluder_mask.id()
    }
}

impl From<&LightOccluder2d> for AssetId<Image> {
    fn from(material: &LightOccluder2d) -> Self {
        material.occluder_mask.id()
    }
}

impl AsAssetId for LightOccluder2d {
    type Asset = Image;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.occluder_mask.id()
    }
}
