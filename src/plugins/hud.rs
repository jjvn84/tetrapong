use crate::STARTING_LIVES;
use crate::game_state::GameState;
use crate::plugins::paddle::{Lives, Paddle, PaddleEliminated, PaddleLocation};
use crate::theme;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // The HUD belongs to the round: hide it on the menus and the result screen,
        // and rebuild it with fresh lives whenever a new round starts.
        app.add_systems(OnEnter(GameState::Game), setup);
        app.add_systems(Update, (update_lives_displays, handle_paddle_eliminated));
    }
}

#[derive(Component)]
struct LivesText(PaddleLocation);

fn setup(mut commands: Commands) {
    spawn_lives_display(
        &mut commands,
        PaddleLocation::Left,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
    );
    spawn_lives_display(
        &mut commands,
        PaddleLocation::Right,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            right: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
    );
    spawn_lives_display(
        &mut commands,
        PaddleLocation::Bottom,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
    );
    spawn_lives_display(
        &mut commands,
        PaddleLocation::Top,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            top: Val::Px(16.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
    );
}

fn spawn_lives_display(commands: &mut Commands, location: PaddleLocation, mut node: Node) {
    node.border_radius = BorderRadius::all(Val::Px(10.0));
    commands
        .spawn((
            node,
            BackgroundColor(theme::HUD_PANEL),
            // Removed with the rest of the round when the player leaves Game.
            DespawnOnExit(GameState::Game),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("{}: {STARTING_LIVES}", location.label())),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(theme::paddle_color(location)),
                LivesText(location),
            ));
        });
}

fn update_lives_displays(
    paddles_query: Query<(&Paddle, &Lives), Changed<Lives>>,
    mut displays_query: Query<(&LivesText, &mut Text)>,
) {
    for (paddle, lives) in &paddles_query {
        for (display, mut text) in &mut displays_query {
            if display.0 == paddle.location {
                **text = format!("{}: {}", paddle.location.label(), lives.0)
            }
        }
    }
}

fn handle_paddle_eliminated(
    mut messages: MessageReader<PaddleEliminated>,
    mut displays_query: Query<(&LivesText, &mut Text)>,
) {
    for PaddleEliminated(location) in messages.read() {
        for (display, mut text) in &mut displays_query {
            if display.0 == *location {
                **text = format!("{}: 0", location.label())
            }
        }
    }
}
