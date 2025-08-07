
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

//USAR Name

//USAR CHILDOF PARA Q TENGAN UNA FUENTE Q AL SER BORRADA BORRA LOS EFECTOS. P. ej: LEG

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[relationship(relationship_target = AppliedModifiers)]
pub struct ModifierTarget(#[relationship]#[entities]pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[relationship_target(relationship = ModifierTarget)]
pub struct AppliedModifiers(Vec<Entity>);
impl AppliedModifiers {pub fn entities(&self) -> &Vec<Entity> {&self.0}}



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
pub struct BaseValue(pub f32);//negate for opposite effect or negation


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct EffectiveValue(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
///poison ID, efectiveness(multiplicador sobre propia Potency, resultado se substrae a la Potency del veneno) 
pub struct Antidote(pub HashMap<String, f32>);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]//PARA PIERNAS 
/// offset value for self if other category is present on the same target as us
pub struct OffsetValForSelf(pub HashMap<String, f32>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]//PARA PIERNAS 
/// copy a portion of value from another modifier into self if present on same target
pub struct CopyValPortionForSelf(pub HashMap<String, f32>);///f32 entre 0 y 1, se multiplica con el valor presente en la cat y lo devuelto se le suma a la efective potency nuestra


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub enum OperationType {
    #[default] Offsetting, 
    /// solo [0, ...] permitido NO RECOMENDADO USAR, MUCHO MÁS DIFÍCIL DE BALANCEAR 
    Scaling, 
    Min, Max
}




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct MitigatingOnly;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories)]
pub struct MovementModifier;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier)]
pub struct Speed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier)]
pub struct InvertMovement;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, )]
pub struct HandlingCapability;