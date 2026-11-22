//! High-level game flow.
//!
//! `GameState` lives in its own module (next to `theme.rs`) because every plugin
//! needs to refer to it: the menu screens are built in `OnEnter` for their state,
//! the gameplay plugins register their round entities in `OnEnter(GameState::Game)`,
//! and `game.rs` gates the systems on it.

use bevy::prelude::*;

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum GameState {
    /// The title screen. This is the state the game boots into.
    #[default]
    MainMenu,
    /// Choose who controls each paddle and how hard the computer plays.
    Options,
    /// Read the rules, controls and power-up explanations.
    HowToPlay,
    /// A round is running: the ball moves, paddles respond, power-ups spawn.
    Game,
    /// The round is decided; gameplay is frozen and the result screen is shown.
    GameOver,
}
