use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use ::being_shared::{
    BeingNavDebugField,
    BeingNavDebugKind,
    BeingNavDebugLine,
    DebuggingBeingNav,
};

#[derive(SystemParam)]
pub struct BeingNavDebugLog<'w, 's> {
    tracked: Res<'w, DebuggingBeingNav>,
    time: Res<'w, Time>,
    writer: MessageWriter<'w, BeingNavDebugLine>,
    messages: Local<'s, Vec<BeingNavDebugLine>>,
}

#[allow(unused_parens, )]
impl<'w, 's> BeingNavDebugLog<'w, 's> {
    pub fn is_tracked(&self, being_ent: Entity) -> bool {
        self.tracked.is_tracked(being_ent)
    }

    pub fn push(
        &mut self,
        being_ent: Entity,
        system: &'static str,
        kind: BeingNavDebugKind,
        summary: impl Into<String>,
        fields: Vec<BeingNavDebugField>,
    ) {
        if !self.tracked.is_tracked(being_ent) {
            return;
        }
        self.messages.push(BeingNavDebugLine {
            being_ent,
            timestamp_secs: self.time.elapsed_secs_f64(),
            system: system.to_string(),
            kind,
            summary: summary.into(),
            fields,
        });
    }

    pub fn flush(&mut self) {
        self.writer.write_batch(self.messages.drain(..));
    }
}
