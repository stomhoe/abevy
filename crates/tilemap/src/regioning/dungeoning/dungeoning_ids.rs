use std::sync::OnceLock;

use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use common::{common_components::HashId, log_targets::SGC_INIT};

pub const DRUNKWALK: HashId = HashId::hash("drunkwalk");
pub const CHAMBERS_CORRIDORS: HashId = HashId::hash("chamberscorridors");
pub const MAZE: HashId = HashId::hash("maze");
pub const SPIRAL: HashId = HashId::hash("spiral");
pub const ARCHI: HashId = HashId::hash("archi");

#[derive(Debug, Clone, Copy)]
pub struct StructureGeneratorDescriptor {
	pub structure_hash_id: HashId,
}

inventory::collect!(StructureGeneratorDescriptor);

pub fn admitted_structure_ids_for_claiming() -> &'static [HashId] {
	static ADMITTED_STRUCTURE_IDS: OnceLock<Vec<HashId>> = OnceLock::new();
	ADMITTED_STRUCTURE_IDS
		.get_or_init(|| {
			let mut seen: HashSet<HashId> = HashSet::default();
			let mut admitted = Vec::new();
			for descriptor in inventory::iter::<StructureGeneratorDescriptor> {
				if seen.insert(descriptor.structure_hash_id) {
					admitted.push(descriptor.structure_hash_id);
				}
			}
			admitted.sort_unstable_by_key(|hash_id| hash_id.as_u64());
			debug!(target: SGC_INIT, "Discovered {} admitted structure generator ids: {:?}", admitted.len(), admitted);
			admitted
		})
		.as_slice()
}
