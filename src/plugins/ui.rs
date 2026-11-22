//! Shared UI building blocks for the menu screens and the game-over overlay.
//!
//! The menus are intentionally built from a handful of helpers so all screens keep
//! the same button sizes, colours and hover behaviour.

use crate::game_state::GameState;
use crate::plugins::paddle::{PaddleLocation, PlayerType};
use crate::settings::{Difficulty, Settings};
use crate::theme;
use bevy::picking::Pickable;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_menu_action);
        app.add_systems(Update, refresh_button_visuals);
    }
}

/// What a menu button does when activated.
///
/// The component lives on the button entity itself, so the single `Activate`
/// observer below can drive every screen (main menu, options and game over).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Play,
    Options,
    HowToPlay,
    BackToMenu,
    Quit,
    SetPlayer {
        location: PaddleLocation,
        player_type: PlayerType,
    },
    SetDifficulty(Difficulty),
}

impl MenuAction {
    /// Toggle buttons (player type and difficulty) show their current value as selected.
    fn is_selected(self, settings: &Settings) -> bool {
        match self {
            Self::SetPlayer {
                location,
                player_type,
            } => settings.player_type(location) == player_type,
            Self::SetDifficulty(difficulty) => settings.difficulty == difficulty,
            Self::Play | Self::Options | Self::HowToPlay | Self::BackToMenu | Self::Quit => false,
        }
    }
}

#[derive(Component)]
pub struct MenuButton;

const BUTTON_WIDTH: f32 = 280.0;
const BUTTON_HEIGHT: f32 = 52.0;
const BUTTON_FONT_SIZE: f32 = 22.0;

const CHOICE_WIDTH: f32 = 144.0;
const CHOICE_HEIGHT: f32 = 44.0;
const CHOICE_FONT_SIZE: f32 = 19.0;

/// Spawns a full-screen UI screen that is automatically removed when `state` is left.
pub fn spawn_screen(
    commands: &mut Commands,
    state: GameState,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) {
    spawn_fullscreen(commands, state, theme::BACKGROUND, build);
}

/// Like [`spawn_screen`], but with a custom background (the game-over overlay uses one).
pub fn spawn_fullscreen(
    commands: &mut Commands,
    state: GameState,
    background: Color,
    build: impl FnOnce(&mut ChildSpawnerCommands),
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
            BackgroundColor(background),
            // Let clicks reach the buttons instead of this full-screen layout node.
            Pickable::IGNORE,
            DespawnOnExit(state),
        ))
        .with_children(build);
}

pub fn spawn_title(parent: &mut ChildSpawnerCommands, text: impl Into<String>, color: Color) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(64.0),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
}

pub fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    font_size: f32,
    color: Color,
) {
    parent.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(color),
        Pickable::IGNORE,
    ));
}

/// A full-width menu button used for navigation actions.
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    action: MenuAction,
    label: impl Into<String>,
) {
    spawn_button_with_size(
        parent,
        action,
        label,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
        BUTTON_FONT_SIZE,
    );
}

/// A smaller button used for the options' exclusive choices.
pub fn spawn_choice_button(
    parent: &mut ChildSpawnerCommands,
    action: MenuAction,
    label: impl Into<String>,
) {
    spawn_button_with_size(
        parent,
        action,
        label,
        CHOICE_WIDTH,
        CHOICE_HEIGHT,
        CHOICE_FONT_SIZE,
    );
}

fn spawn_button_with_size(
    parent: &mut ChildSpawnerCommands,
    action: MenuAction,
    label: impl Into<String>,
    width: f32,
    height: f32,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            MenuButton,
            action,
            Hovered::default(),
            Pickable::default(),
            Node {
                width: Val::Px(width),
                height: Val::Px(height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(theme::MENU_BUTTON),
            BorderColor::all(theme::MENU_BUTTON_BORDER),
        ))
        .with_children(|parent| {
            // The label must not block picking: otherwise the text node would sit
            // on top of the button and swallow clicks.
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(font_size),
                    ..default()
                },
                TextColor(theme::MENU_TEXT),
                Pickable::IGNORE,
            ));
        });
}

type MenuButtonQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static MenuAction,
        &'static Hovered,
        Has<Pressed>,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    With<MenuButton>,
>;

fn refresh_button_visuals(settings: Res<Settings>, mut buttons: MenuButtonQuery) {
    for (action, hovered, pressed, mut background, mut border) in &mut buttons {
        let selected = action.is_selected(&settings);

        let background_color = if pressed {
            theme::MENU_BUTTON_PRESSED
        } else if selected && hovered.get() {
            theme::MENU_BUTTON_SELECTED_HOVER
        } else if selected {
            theme::MENU_BUTTON_SELECTED
        } else if hovered.get() {
            theme::MENU_BUTTON_HOVER
        } else {
            theme::MENU_BUTTON
        };

        if background.0 != background_color {
            background.0 = background_color;
        }

        let border_color = if selected {
            theme::MENU_BUTTON_BORDER_SELECTED
        } else {
            theme::MENU_BUTTON_BORDER
        };
        if border.top != border_color {
            border.set_all(border_color);
        }
    }
}

fn handle_menu_action(
    activation: On<Activate>,
    actions: Query<&MenuAction>,
    mut settings: ResMut<Settings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(&action) = actions.get(activation.entity) else {
        return;
    };

    match action {
        MenuAction::Play => next_state.set(GameState::Game),
        MenuAction::Options => next_state.set(GameState::Options),
        MenuAction::HowToPlay => next_state.set(GameState::HowToPlay),
        MenuAction::BackToMenu => next_state.set(GameState::MainMenu),
        MenuAction::Quit => {
            exit.write(AppExit::Success);
        }
        MenuAction::SetPlayer {
            location,
            player_type,
        } => settings.set_player_type(location, player_type),
        MenuAction::SetDifficulty(difficulty) => settings.difficulty = difficulty,
    }
}
