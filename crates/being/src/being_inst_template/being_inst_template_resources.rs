use being_shared::BeingInstTemplate;
use bevy::{prelude::*};
pub use crate::being_inst_template::being_inst_template_seris::*;

common::define_entity_map_systems!(
    BeingInstTemplate,
    (),
    Bit,
    "bit",
    "BIT",
    BeingInstTemplate,
    common::common_components::StrId,
    BitSeri, "seri.being.inst_templ", "bit.ron",
);
