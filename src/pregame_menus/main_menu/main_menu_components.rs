#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component)]
pub enum MainMenuButton {QuickStart, Host, Join, Settings}


#[derive(Component)]
pub struct MainMenuIpLineEdit;