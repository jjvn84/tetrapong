use crate::STARTING_LIVES;
use crate::plugins::paddle::{Lives, Paddle, PaddleEliminated, PaddleLocation};
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_lives_displays);
        app.add_systems(Update, handle_paddle_eliminated);
    }
}

#[derive(Component)]
struct LivesDisplay(PaddleLocation);

fn setup(mut commands: Commands) {
    commands.spawn((
        Text::new(format!("Left: {STARTING_LIVES}")),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.2, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        LivesDisplay(PaddleLocation::Left),
    ));
    commands.spawn((
        Text::new(format!("Right: {STARTING_LIVES}")),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.2, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        LivesDisplay(PaddleLocation::Right),
    ));
    commands.spawn((
        Text::new(format!("Bottom: {STARTING_LIVES}")),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.2, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        LivesDisplay(PaddleLocation::Bottom),
    ));
    commands.spawn((
        Text::new(format!("Top: {STARTING_LIVES}")),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.2, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        LivesDisplay(PaddleLocation::Top),
    ));
}

fn update_lives_displays(
    paddles_query: Query<(&Paddle, &Lives), Changed<Lives>>,
    mut displays_query: Query<(&LivesDisplay, &mut Text)>,
) {
    for (paddle, lives) in &paddles_query {
        for (display, mut text) in &mut displays_query {
            if display.0 == paddle.location {
                **text = format!("{:?}: {}", paddle.location, lives.0)
            }
        }
    }
}

fn handle_paddle_eliminated(
    mut messages: MessageReader<PaddleEliminated>,
    mut displays_query: Query<(&LivesDisplay, &mut Text)>,
) {
    for PaddleEliminated(location) in messages.read() {
        for (display, mut text) in &mut displays_query {
            if display.0 == *location {
                **text = format!("{location:?}: 0")
            }
        }
    }
}
