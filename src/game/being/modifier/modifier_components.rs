
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

//USAR Name

//USAR CHILDOF PARA Q TENGAN UNA FUENTE Q AL SER BORRADA BORRA LOS EFECTOS

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[relationship(relationship_target = AppliedModifiers)]
pub struct ModifierTarget {
    #[relationship] #[entities]
    pub target: Entity,
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[relationship_target(relationship = ModifierTarget)]
pub struct AppliedModifiers(Vec<Entity>);
impl AppliedModifiers {
    pub fn entities(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories)]
pub struct MovementModifier;




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(CurrTimeBasedPotency, )]
pub struct BasePotency(pub f32);//negate for opposite effect or negation


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct CurrTimeBasedPotency(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct EffectivePotency(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
///poison ID, efectiveness(multiplicador sobre propia Potency, resultado se substrae a la Potency del veneno) 
pub struct Antidote(pub HashMap<String, f32>);
impl Antidote {

}


/*TO-DO ¡IMPORTANTE! NO OLVIDARSE DE AGREGAR: 
superstate_plugin::<Modifier, (Walking, Flying)>,
 EN EL Plugin DEL MÓDULO */
#[derive(Component, Default)]
pub struct ModifierCategories(
    /*categorías/tipo de sustancia/familia de sustancia a las q pertenece: fentanyl, race_modifier, narcan 
     (así se pueden identificar sustancias origen y hacer sistemas de antidotos q contrarresten sustancias específicas)
    */
    pub Vec<String>
);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct MultiplyingModifier;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct MitigatingOnly;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier)]
pub struct Speed;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier)]
pub struct HandlingCapability;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier)]
pub struct InvertMovement;