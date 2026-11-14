use crate::plugins::{
    paddle::GAP_BETWEEN_PADDLE_AND_FLOOR,
    paddle::Paddle,
    paddle::PaddleLocation,
    physics::{Collider, ColliderType},
};
use bevy::prelude::*;
use rand::RngExt;
use rand::seq::IndexedRandom;
use strum::VariantArray;

const POWERUP_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const POWERUP_SIZE: f32 = 3.0; // As a percentage of the window's minimum dimension

const MAXIMUM_NUMBER_OF_ACTIVE_POWERUPS: usize = 4;

pub struct PowerUpsPlugin;

impl Plugin for PowerUpsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Update, (spawn_power_up, handle_powerup_hit));
        app.add_message::<PowerUpHit>();
    }
}

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Component)]
#[require(Collider(ColliderType::PowerUp))]

pub struct PowerUp {
    pub kind: PowerUpKind,
}

#[derive(VariantArray, Copy, Clone)]
pub enum PowerUpKind {
    PaddleHalf,
    PaddleDouble,
}

#[derive(Message)]
pub struct PowerUpHit {
    pub kind: PowerUpKind,
    pub paddle: PaddleLocation,
}

fn spawn_power_up(
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut commands: Commands,
    current_power_ups: Query<&PowerUp>,
    window: Query<&mut Window>,
) {
    if current_power_ups.iter().len() == MAXIMUM_NUMBER_OF_ACTIVE_POWERUPS {
        return;
    }
    // Tick the timer with the delta time from the `Time` resource
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        let min_window_dimension = window
            .single()
            .unwrap()
            .resolution
            .width()
            .min(window.single().unwrap().resolution.height());

        let powerup_x = rand::rng().random_range(
            -min_window_dimension / 2.0
                + 2.0 * GAP_BETWEEN_PADDLE_AND_FLOOR * min_window_dimension / 100.0
                ..min_window_dimension / 2.0
                    - 2.0 * GAP_BETWEEN_PADDLE_AND_FLOOR * min_window_dimension / 100.0,
        );
        let powerup_y = rand::rng().random_range(
            -min_window_dimension / 2.0
                + 2.0 * GAP_BETWEEN_PADDLE_AND_FLOOR * min_window_dimension / 100.0
                ..min_window_dimension / 2.0
                    - 2.0 * GAP_BETWEEN_PADDLE_AND_FLOOR * min_window_dimension / 100.0,
        );

        let mut rng = rand::rng();
        let powerup_kind = *PowerUpKind::VARIANTS.choose(&mut rng).unwrap();

        commands.spawn((
            Sprite::from_color(POWERUP_COLOR, Vec2::ONE),
            Transform {
                translation: Vec3::new(powerup_x, powerup_y, 1.0),
                scale: Vec3::new(
                    POWERUP_SIZE * min_window_dimension / 100.0,
                    POWERUP_SIZE * min_window_dimension / 100.0,
                    1.0,
                ),
                ..default()
            },
            Collider(ColliderType::PowerUp),
            PowerUp { kind: powerup_kind },
        ));
    }
}

fn handle_powerup_hit(
    mut messages: MessageReader<PowerUpHit>,
    mut paddles: Query<(&mut Paddle, &mut Transform)>,
    window: Query<&Window>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let min_window_dimension = window.resolution.width().min(window.resolution.height());

    for message in messages.read() {
        // Messages are read-only, so look up the real paddle entity instead of
        // trying to mutate the copy carried by the message.
        let Some((mut paddle, mut paddle_transform)) = paddles
            .iter_mut()
            .find(|(paddle, _)| paddle.location == message.paddle)
        else {
            // The paddle may have been eliminated since the power-up spawned.
            continue;
        };

        match message.kind {
            PowerUpKind::PaddleDouble => {
                paddle.width_multiplier = (2.0 * paddle.width_multiplier).min(4.0);
            }
            PowerUpKind::PaddleHalf => {
                // Halve the width, but never go below a quarter of the original.
                paddle.width_multiplier = (0.5 * paddle.width_multiplier).max(0.25);
            }
        }

        // Apply the new width immediately; otherwise the sprite and its
        // collision AABB keep the old size until the next window resize.
        paddle_transform.scale = paddle.size(min_window_dimension);
    }
}
