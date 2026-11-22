use bevy::prelude::*;
mod game_state;
mod plugins;
mod settings;
mod theme;
use crate::plugins::game::GamePlugin;

const STARTING_LIVES: u32 = 3;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
