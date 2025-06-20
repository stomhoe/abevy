use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::beings::classes::classes_components::*;

//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource, Debug, Default)]
pub struct ClassesDatabase (
    pub HashMap<ClassNid, Entity>
);



