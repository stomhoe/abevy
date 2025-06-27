use bevy::prelude::*;
//use bevy_renet::renet::ClientId;

#[derive(Component, Debug,)]
#[require(Player)]
pub struct SelfPlayer {}


//NO ES PARA ADJUNTARSELO A ENTITIES COMÚNES (OBJETOS O BEINGS)
// ES PARA ADJUNTARSELO A ENTITIES QUE REPRESENTAN JUGADORES
#[derive(Debug, Component)]
pub struct Player {
    //pub id: ClientId,
    pub display_name: String,
}

impl Default for Player {
    fn default() -> Self {
        Self { 
            //id: ClientId::from(rand::random::<u64>()),
            display_name: format!("Player-{}", nano_id::base64::<6>()),
        }
    }
}


#[derive(Component, Default)] 
#[require(Transform)]
pub struct CameraTarget;
