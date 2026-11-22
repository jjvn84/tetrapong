use crate::game_state::GameState;
use crate::plugins::ball::{BallPlugin, BallSet};
use crate::plugins::hud::HudPlugin;
use crate::plugins::paddle::{Lives, Paddle, PaddleLocation, PaddlePlugin, PaddleSet, PlayerType};
use crate::plugins::physics::{PhysicsPlugin, PhysicsSet};
use crate::plugins::powerups::{PowerUpsPlugin, PowerUpsSet};
use crate::plugins::walls::WallsPlugin;
use crate::theme;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        app.insert_resource(ClearColor(theme::BACKGROUND));

        // The state must be initialised before the plugins below register their
        // `OnEnter` systems, and after `DefaultPlugins` (which provides `StatesPlugin`).
        app.init_state::<GameState>();
        app.init_resource::<GameResult>();

        app.add_plugins(WallsPlugin);
        app.add_plugins(BallPlugin);
        app.add_plugins(PaddlePlugin);
        app.add_plugins(PhysicsPlugin);
        app.add_plugins(HudPlugin);
        app.add_plugins(PowerUpsPlugin);

        app.add_systems(Startup, setup);

        // Every gameplay system belongs to a set defined by its own plugin; the
        // state gate is applied once, here, instead of in each plugin.
        app.configure_sets(
            Update,
            (PaddleSet, BallSet, PhysicsSet, PowerUpsSet).run_if(in_state(GameState::Game)),
        );

        app.add_systems(
            Update,
            check_game_over
                .after(PaddleSet)
                .after(PowerUpsSet)
                .run_if(in_state(GameState::Game)),
        );
        app.add_systems(OnEnter(GameState::Game), reset_game_result);
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen);
        app.add_systems(Update, restart_on_key.run_if(in_state(GameState::GameOver)));
    }
}

/// Outcome of the round, read by the game-over screen.
#[derive(Resource, Default)]
pub struct GameResult(pub GameResultVariant);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResultVariant {
    #[default]
    InProgress,
    /// One paddle outlived every other one.
    Winner(PaddleLocation),
    /// Every human-controlled paddle was eliminated.
    HumansEliminated,
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
}

/// Ends the round once every human paddle is gone, or once only one paddle is left.
///
/// The last paddle standing always wins, whatever controls it. The human-loss rule
/// only applies while more than one paddle survives: it ends the round as soon as no
/// `PlayerType::Human` paddle is left, even if several computer paddles are alive.
fn check_game_over(
    paddles: Query<(&Paddle, &Lives, &PlayerType)>,
    mut next: ResMut<NextState<GameState>>,
    mut result: ResMut<GameResult>,
) {
    let alive: Vec<_> = paddles
        .iter()
        .filter(|(_, lives, _)| lives.0 > 0)
        .map(|(paddle, _, player_type)| (paddle.location, *player_type))
        .collect();

    if let Some(outcome) = decide_outcome(&alive) {
        result.0 = outcome;
        next.set(GameState::GameOver);
    }
}

/// Decides whether the round is over, given the paddles that still have lives.
///
/// `None` means the round keeps going (including the empty case, which happens
/// before the first paddle is spawned).
fn decide_outcome(alive: &[(PaddleLocation, PlayerType)]) -> Option<GameResultVariant> {
    // The last paddle standing wins, even if it is computer-controlled.
    if let [(location, _)] = alive {
        return Some(GameResultVariant::Winner(*location));
    }

    if alive.is_empty() {
        return None;
    }

    // With more than one survivor, losing every human paddle ends the round.
    if !alive
        .iter()
        .any(|(_, player_type)| *player_type == PlayerType::Human)
    {
        return Some(GameResultVariant::HumansEliminated);
    }

    None
}

fn spawn_game_over_screen(mut commands: Commands, result: Res<GameResult>) {
    let (title, title_color) = match &result.0 {
        GameResultVariant::Winner(location) => (
            format!("{location:?} paddle wins!"),
            theme::paddle_color(*location),
        ),
        GameResultVariant::HumansEliminated => ("Game Over".to_string(), theme::BALL),
        GameResultVariant::InProgress => return,
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(theme::OVERLAY),
            DespawnOnExit(GameState::GameOver),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::Px(56.0),
                    ..default()
                },
                TextColor(title_color),
            ));
            if result.0 == GameResultVariant::HumansEliminated {
                parent.spawn((
                    Text::new("No human player remains"),
                    TextFont {
                        font_size: FontSize::Px(36.0),
                        ..default()
                    },
                    TextColor(theme::TEXT_MUTED),
                ));
            }
            parent.spawn((
                Text::new("Press R to play again"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(theme::TEXT_MUTED),
            ));
        });
}

fn restart_on_key(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next.set(GameState::Game);
    }
}

fn reset_game_result(mut result: ResMut<GameResult>) {
    *result = GameResult::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::paddle::{PaddleLocation::*, PlayerType::*};

    #[test]
    fn full_arena_keeps_the_round_running() {
        let alive = [
            (Left, Computer),
            (Right, Computer),
            (Bottom, Human),
            (Top, Computer),
        ];

        assert_eq!(decide_outcome(&alive), None);
    }

    #[test]
    fn last_human_paddle_wins() {
        assert_eq!(
            decide_outcome(&[(Bottom, Human)]),
            Some(GameResultVariant::Winner(Bottom))
        );
    }

    #[test]
    fn eliminating_every_human_ends_the_round() {
        let alive = [(Left, Computer), (Right, Computer), (Top, Computer)];

        assert_eq!(
            decide_outcome(&alive),
            Some(GameResultVariant::HumansEliminated)
        );
    }

    #[test]
    fn remaining_humans_keep_the_round_running() {
        let alive = [(Bottom, Human), (Top, Human), (Left, Computer)];

        // With local multiplayer, one human going down is not the end of the round.
        assert_eq!(decide_outcome(&alive), None);

        let alive = [(Top, Human), (Left, Computer)];

        assert_eq!(decide_outcome(&alive), None);
    }

    #[test]
    fn last_paddle_standing_wins_even_when_it_is_a_computer() {
        assert_eq!(
            decide_outcome(&[(Left, Computer)]),
            Some(GameResultVariant::Winner(Left))
        );
    }

    #[test]
    fn empty_arena_is_not_decided() {
        assert_eq!(decide_outcome(&[]), None);
    }
}
