use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlipHorizontallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlipVerticallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct FlipDiagonallyBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct ChangeFacingDirectionBasedOnHash;

#[derive(Component, Deserialize, Serialize, Default, Debug, Clone)]
pub struct RotateTransform;
