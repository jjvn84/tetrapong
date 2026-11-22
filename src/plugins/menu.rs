//! Menu screens: the title screen, the player/difficulty options and the rules.

use crate::game_state::GameState;
use crate::plugins::paddle::{PaddleLocation, PlayerType};
use crate::plugins::ui::{
    MenuAction, spawn_button, spawn_choice_button, spawn_screen, spawn_text, spawn_title,
};
use crate::settings::{Difficulty, Settings};
use crate::theme;
use bevy::picking::Pickable;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu);
        app.add_systems(OnEnter(GameState::Options), spawn_options);
        app.add_systems(OnEnter(GameState::HowToPlay), spawn_how_to_play);
        // Escape is the keyboard shortcut for "go back": from the options, the
        // rules, a running round or the result screen it always returns to the menu.
        app.add_systems(Update, escape_to_menu);
    }
}

fn spawn_main_menu(mut commands: Commands, settings: Res<Settings>) {
    spawn_screen(&mut commands, GameState::MainMenu, |parent| {
        spawn_title(parent, "TETRAPONG", theme::BALL);
        spawn_text(
            parent,
            "Four paddles. One ball. No safe walls.",
            19.0,
            theme::TEXT_MUTED,
        );
        spawn_settings_summary(parent, &settings);

        parent.spawn((
            Node {
                height: Val::Px(8.0),
                ..default()
            },
            Pickable::IGNORE,
        ));

        spawn_button(parent, MenuAction::Play, "Play");
        spawn_button(parent, MenuAction::HowToPlay, "How to Play");
        spawn_button(parent, MenuAction::Options, "Options");
        spawn_button(parent, MenuAction::Quit, "Quit");

        spawn_footer(parent);
    });
}

fn spawn_settings_summary(parent: &mut ChildSpawnerCommands, settings: &Settings) {
    let humans = PaddleLocation::ALL
        .iter()
        .filter(|location| settings.player_type(**location) == PlayerType::Human)
        .count();

    let summary = if humans == 0 {
        format!("All computer paddles - {}", settings.difficulty.label())
    } else {
        format!(
            "{} human - {} computer - {}",
            humans,
            PaddleLocation::COUNT - humans,
            settings.difficulty.label()
        )
    };

    spawn_text(parent, summary, 16.0, theme::TEXT_MUTED);
}

fn spawn_footer(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(18.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|footer| {
            spawn_text(
                footer,
                "github.com/jjvn84/tetrapong  |  MIT License  |  (c) 2026 Juan Vasquez",
                14.0,
                theme::TEXT_MUTED,
            );
        });
}

fn spawn_options(mut commands: Commands) {
    spawn_screen(&mut commands, GameState::Options, |parent| {
        spawn_title(parent, "Options", theme::BALL);
        spawn_text(
            parent,
            "Choose who controls each paddle and how hard the computer plays.",
            18.0,
            theme::TEXT_MUTED,
        );

        spawn_text(parent, "PLAYERS", 16.0, theme::TEXT_MUTED);
        for location in PaddleLocation::ALL {
            spawn_player_row(parent, location);
        }

        spawn_text(parent, "DIFFICULTY", 16.0, theme::TEXT_MUTED);
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                for difficulty in Difficulty::ALL {
                    spawn_choice_button(
                        row,
                        MenuAction::SetDifficulty(difficulty),
                        difficulty.label(),
                    );
                }
            });
        parent.spawn((
            Node {
                height: Val::Px(4.0),
                ..default()
            },
            Pickable::IGNORE,
        ));
        spawn_button(parent, MenuAction::BackToMenu, "Back");
    });
}

fn spawn_player_row(parent: &mut ChildSpawnerCommands, location: PaddleLocation) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    // Wide enough for the longest name ("Bottom paddle") on one line
                    // at this font size; a fixed width keeps the buttons aligned too.
                    width: Val::Px(160.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|label| {
                spawn_text(
                    label,
                    format!("{} paddle", location.label()),
                    18.0,
                    theme::paddle_color(location),
                );
            });

            spawn_choice_button(
                row,
                MenuAction::SetPlayer {
                    location,
                    player_type: PlayerType::Human,
                },
                PlayerType::Human.label(),
            );
            spawn_choice_button(
                row,
                MenuAction::SetPlayer {
                    location,
                    player_type: PlayerType::Computer,
                },
                PlayerType::Computer.label(),
            );
        });
}

fn spawn_how_to_play(mut commands: Commands) {
    spawn_screen(&mut commands, GameState::HowToPlay, |parent| {
        spawn_title(parent, "How to Play", theme::BALL);

        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: Val::Px(48.0),
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|columns| {
                spawn_rules_column(columns);
                spawn_power_ups_column(columns);
            });

        spawn_button(parent, MenuAction::BackToMenu, "Back");
    });
}

fn spawn_rules_column(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Px(470.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|column| {
            spawn_text(column, "RULES", 18.0, theme::BALL);
            for line in [
                "Every paddle starts with 3 lives.",
                "When the ball hits your wall, you lose a life.",
                "After each point the ball waits 2 seconds, then serves in a random direction.",
                "At 0 lives the paddle is eliminated.",
                "The last paddle standing wins.",
                "If humans are playing, the round also ends when every human paddle is gone.",
            ] {
                spawn_text(column, format!("- {line}"), 15.0, theme::TEXT_MUTED);
            }

            spawn_text(column, "CONTROLS", 18.0, theme::BALL);
            for line in [
                "Bottom paddle: Left / Right arrow keys",
                "Left paddle: Q / A",
                "Right paddle: I / K",
                "Top paddle: F / G",
            ] {
                spawn_text(column, line, 15.0, theme::TEXT_MUTED);
            }
            spawn_text(
                column,
                "Keys only apply to paddles set to Human in Options.",
                15.0,
                theme::TEXT_MUTED,
            );
            spawn_text(
                column,
                "Esc returns to the menu. R restarts a finished round.",
                15.0,
                theme::TEXT_MUTED,
            );
        });
}

fn spawn_power_ups_column(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: Val::Px(430.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|column| {
            spawn_text(column, "POWER-UPS", 18.0, theme::BALL);
            spawn_text(
                column,
                "The last paddle to hit the ball collects the badge it touches. Power-ups do not bounce the ball.",
                15.0,
                theme::TEXT_MUTED,
            );

            for (color, line) in [
                (
                    theme::POWER_UP_WIDTH,
                    "Wider or narrower paddle (blue ring)",
                ),
                (
                    theme::POWER_UP_PADDLE_SPEED,
                    "Faster or slower paddle (purple ring)",
                ),
                (
                    theme::POWER_UP_BALL_SPEED,
                    "Faster or slower ball (orange ring)",
                ),
                (
                    theme::POWER_UP_EARN_LIFE,
                    "Gain one life (green ring)",
                ),
                (
                    theme::POWER_UP_LOSE_LIFE,
                    "Lose one life (red ring)",
                ),
            ] {
                spawn_text(column, format!("- {line}"), 15.0, color);
            }

            spawn_text(
                column,
                "Stack enough speed power-ups and the ball becomes hard to read.",
                15.0,
                theme::TEXT_MUTED,
            );
        });
}

fn escape_to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *state.get() != GameState::MainMenu && keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}
