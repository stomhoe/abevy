use bevy::platform::collections::HashMap;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
#[relationship(relationship_target = AppliedModifiers)]
#[require(
    AssetScoped,
    SparedFromHotReloading,
    Replicated,
    ApplyMode::Add,
    Prefix::trunc("Modif"),
    ExcludedFromAutoRenamer,
)]
pub struct ModifierTarget(
    #[relationship]
    #[entities]
    pub Entity,
);

#[derive(Component, Debug, Default, Clone, Reflect)]
#[relationship_target(relationship = ModifierTarget)]
pub struct AppliedModifiers(Vec<Entity>);

// BORRÉ TODOS LOS Reflect PORQUE QUIERO QUE SE IMPLEMENTEN DEBUG WINDOWS PARA VER ESTOS COMPONENTES COMODAMENTE

pub type ModifierTags = TagSet;
/*categorías/tipo de sustancia/familia de sustancia a las q pertenece: race_modifier,
    (así se pueden identificar sustancias origen y hacer sistemas de antidotos q contrarresten sustancias específicas)
*/

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(CurrEffectiveValue)]
pub struct BaseValue(pub f32); //negate for opposite effect or mitigation

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ApplyMode::Add)]
/// final value after all antidote and OffsetValForSelf and CopyFracOfOthersIntoSelf processing
pub struct CurrEffectiveValue(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
///poison ID, efectiveness(multiplicador sobre propia Potency, resultado se substrae a la Potency del veneno)
pub struct Antidote(pub HashMap<Tag, f32>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
/// offset value for self if other tag is present on the same target as us. (could be used for sinergy between modifiers, e.g. a modifier with "leg" tag gives an offset to self's final value if we are both present on the same target)
pub struct OffsetValForSelf(pub HashMap<Tag, f32>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
/// copy a portion of value from another modifier into self if present on same target
pub struct CopyFracOfOthersIntoSelf(pub HashMap<Tag, f32>);
///f32 entre 0 y 1, se multiplica con el valor presente en la cat y lo devuelto se le suma a la efective potency nuestra

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum ModifierSynergy {
    Offset(f32),
    CopyFrac(f32),
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ModifierSynergies(pub HashMap<Tag, ModifierSynergy>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq)]
pub struct MinForDamage;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, )]
//para flat/scaled damage reduction o increase. combinar con OperationType para flat damage reduction o scaled
pub struct ConvertsDamageOnNonPenetration(pub HashMap<String, String>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub enum ApplyMode {
    #[default]
    Add,
    Min,
    Max,
    /// solo [0, ...] permitido. NO RECOMENDADO USAR, MUCHO MÁS DIFÍCIL DE BALANCEAR
    Mul,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct MitigatingOnly;
