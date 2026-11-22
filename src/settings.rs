//! Player-configurable options shared by the menu and the gameplay plugins.
//!
//! `Settings` is a resource so the Options screen can edit it and the round setup
//! (paddle ownership and AI difficulty) can read it without extra plumbing.

use crate::plugins::paddle::{PaddleLocation, PlayerType};
use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// One entry per `PaddleLocation`, indexed by `PaddleLocation::index`.
    player_types: [PlayerType; 4],
    pub difficulty: Difficulty,
}

impl Default for Settings {
    fn default() -> Self {
        let mut player_types = [PlayerType::Computer; PaddleLocation::COUNT];
        player_types[PaddleLocation::Bottom.index()] = PlayerType::Human;

        Self {
            player_types,
            difficulty: Difficulty::default(),
        }
    }
}

impl Settings {
    pub fn player_type(&self, location: PaddleLocation) -> PlayerType {
        self.player_types[location.index()]
    }

    pub fn set_player_type(&mut self, location: PaddleLocation, player_type: PlayerType) {
        self.player_types[location.index()] = player_type;
    }

    /// Whether the round includes at least one human-controlled paddle.
    ///
    /// An all-computer round is allowed: in that case the "every human was
    /// eliminated" rule is skipped and the round runs until one paddle is left.
    pub fn has_human_players(&self) -> bool {
        self.player_types.contains(&PlayerType::Human)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Easy, Self::Medium, Self::Hard];

    /// The value `fail_probability` must reach for the computer paddle to skip a
    /// frame. `fail_probability` is drawn from `0.0..10.0`, so higher thresholds
    /// mean the paddle misses less often, i.e. a harder opponent.
    pub const fn fail_threshold(self) -> f32 {
        match self {
            Self::Easy => 5.0,
            Self::Medium => 6.0,
            Self::Hard => 7.0,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_one_human_and_medium() {
        let settings = Settings::default();

        assert_eq!(
            settings.player_type(PaddleLocation::Bottom),
            PlayerType::Human
        );
        for location in [
            PaddleLocation::Left,
            PaddleLocation::Right,
            PaddleLocation::Top,
        ] {
            assert_eq!(settings.player_type(location), PlayerType::Computer);
        }
        assert_eq!(settings.difficulty, Difficulty::Medium);
        assert!(settings.has_human_players());
    }

    #[test]
    fn player_types_are_stored_per_side() {
        let mut settings = Settings::default();

        settings.set_player_type(PaddleLocation::Left, PlayerType::Human);
        settings.set_player_type(PaddleLocation::Bottom, PlayerType::Computer);

        assert_eq!(
            settings.player_type(PaddleLocation::Left),
            PlayerType::Human
        );
        assert_eq!(
            settings.player_type(PaddleLocation::Bottom),
            PlayerType::Computer
        );
        assert_eq!(
            settings.player_type(PaddleLocation::Right),
            PlayerType::Computer
        );
    }

    #[test]
    fn an_all_computer_round_has_no_humans() {
        let mut settings = Settings::default();
        for location in PaddleLocation::ALL {
            settings.set_player_type(location, PlayerType::Computer);
        }

        assert!(!settings.has_human_players());
    }

    #[test]
    fn difficulty_thresholds_match_the_requested_modes() {
        assert_eq!(Difficulty::Easy.fail_threshold(), 5.0);
        assert_eq!(Difficulty::Medium.fail_threshold(), 6.0);
        assert_eq!(Difficulty::Hard.fail_threshold(), 7.0);
    }
}
