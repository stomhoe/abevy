use bevy_replicon::prelude::Channel;

pub const AO_MOVEMENT_INPUT_CHANNEL: Channel = Channel::Unordered;
pub const AO_MOVEMENT_CORRECTION_CHANNEL: Channel = Channel::Unordered;
pub const AO_MOVEMENT_STEP_CHANNEL: Channel = Channel::Unreliable;
pub const AO_MOVEMENT_HEADING_CHANNEL: Channel = Channel::Ordered;
