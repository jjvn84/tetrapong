//! High-level game flow.
//!
//! `GameState` lives in its own module (next to `theme.rs`) because every plugin
//! needs to refer to it: the gameplay plugins register their round entities in
//! `OnEnter(GameState::Game)`, and `game.rs` gates the systems on it.

use bevy::prelude::*;

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum GameState {
    /// A round is running: the ball moves, paddles respond, power-ups spawn.
    #[default]
    Game,
    /// The round is decided; gameplay is frozen and the result screen is shown.
    GameOver,
}
