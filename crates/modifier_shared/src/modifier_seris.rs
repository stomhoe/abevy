#[derive(serde::Deserialize, Debug, Clone)]
pub enum ModifierSynergySeri {
    Offset(f32),
    CopyFrac(f32),
}
