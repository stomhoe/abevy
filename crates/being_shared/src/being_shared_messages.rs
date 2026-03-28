#[allow(unused_imports)] use bevy::prelude::*;



#[derive(Message, Debug, Clone, )]
pub struct UnfreezeBeing(pub Entity);

#[derive(Message, Debug, Clone, )]
pub struct FaithfulSimBeing(pub Entity);
