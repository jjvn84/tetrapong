use bevy::prelude::*;
mod plugins;
mod theme;
use crate::plugins::{
    ball::BallPlugin, hud::HudPlugin, paddle::PaddlePlugin, physics::PhysicsPlugin,
    powerups::PowerUpsPlugin, walls::WallsPlugin,
};

const STARTING_LIVES: u32 = 3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(theme::BACKGROUND))
        .add_plugins(WallsPlugin)
        .add_plugins(BallPlugin)
        .add_plugins(PaddlePlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(HudPlugin)
        .add_plugins(PowerUpsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
}
