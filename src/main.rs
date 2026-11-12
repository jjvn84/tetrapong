use bevy::prelude::*;
mod plugins;
use crate::plugins::{
    ball::BallPlugin, hud::HudPlugin, paddle::PaddlePlugin, physics::PhysicsPlugin,
    walls::WallsPlugin,
};

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

const STARTING_LIVES: u32 = 3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(WallsPlugin)
        .add_plugins(BallPlugin)
        .add_plugins(PaddlePlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(HudPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
}
