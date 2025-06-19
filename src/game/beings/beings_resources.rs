use bevy::{platform::collections::HashMap, prelude::*};

use crate::game::{beings::beings_components::*};


//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource)]
pub struct ClassDatabase (
    HashMap<ClassNid, Entity>
);

#[derive(Resource)]
pub struct RaceDatabase (
    HashMap<RaceNid, Entity>
);


