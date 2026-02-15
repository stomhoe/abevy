#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Clone)]
pub enum MainMenuButton {QuickStart, Host, Join, Settings}

#[derive(Component, Clone)]
pub struct MainMenuIpLineEdit;
