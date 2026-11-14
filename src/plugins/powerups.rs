use crate::plugins::{
    ball::Ball,
    paddle::GAP_BETWEEN_PADDLE_AND_FLOOR,
    paddle::Lives,
    paddle::Paddle,
    paddle::PaddleEliminated,
    paddle::PaddleLocation,
    physics::{Collider, ColliderType, Velocity},
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
    PaddleSpeedHalf,
    PaddleSpeedDouble,
    BallSpeedHalf,
    BallSpeedDouble,
    LoseOneLife,
    EarnOneLife,
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
    mut commands: Commands,
    mut messages: MessageReader<PowerUpHit>,
    mut paddles: Query<(Entity, &mut Paddle, &mut Transform, &mut Lives)>,
    mut eliminated: MessageWriter<PaddleEliminated>,
    ball: Single<(&mut Ball, &mut Velocity)>,
    window: Query<&Window>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let min_window_dimension = window.width().min(window.height());
    let (mut ball, mut ball_velocity) = ball.into_inner();

    for message in messages.read() {
        match message.kind {
            PowerUpKind::BallSpeedDouble | PowerUpKind::BallSpeedHalf => {
                let factor = match message.kind {
                    PowerUpKind::BallSpeedDouble => 2.0,
                    PowerUpKind::BallSpeedHalf => 0.5,
                    _ => unreachable!(),
                };

                ball.multiply_speed(factor);

                // Keep the current direction but apply the clamped speed now.
                // During the serve pause the velocity is zero, so there is
                // nothing to rescale; the next serve already uses the new speed.
                if ball_velocity.0 != Vec2::ZERO {
                    ball_velocity.0 =
                        ball_velocity.0.normalize() * ball.speed(min_window_dimension);
                }
            }
            kind @ (PowerUpKind::PaddleDouble
            | PowerUpKind::PaddleHalf
            | PowerUpKind::PaddleSpeedDouble
            | PowerUpKind::PaddleSpeedHalf
            | PowerUpKind::LoseOneLife
            | PowerUpKind::EarnOneLife) => {
                // Messages are read-only, so look up the real paddle entity
                // instead of trying to mutate a copy carried by the message.
                let Some((paddle_entity, mut paddle, mut paddle_transform, mut lives)) = paddles
                    .iter_mut()
                    .find(|(_, paddle, _, _)| paddle.location == message.paddle)
                else {
                    // The paddle may have been eliminated since the power-up spawned.
                    continue;
                };

                match kind {
                    PowerUpKind::PaddleDouble => {
                        paddle.width_multiplier = (2.0 * paddle.width_multiplier).min(4.0);
                    }
                    PowerUpKind::PaddleHalf => {
                        // Halve the width, but never go below a quarter of the original.
                        paddle.width_multiplier = (0.5 * paddle.width_multiplier).max(0.25);
                    }
                    PowerUpKind::PaddleSpeedDouble => {
                        paddle.speed_multiplier = (2.0 * paddle.speed_multiplier).min(4.0);
                    }
                    PowerUpKind::PaddleSpeedHalf => {
                        paddle.speed_multiplier = (0.5 * paddle.speed_multiplier).max(0.25);
                    }
                    PowerUpKind::LoseOneLife => {
                        lives.0 = lives.0.saturating_sub(1);

                        if lives.0 == 0 {
                            commands.entity(paddle_entity).despawn();
                            eliminated.write(PaddleEliminated(paddle.location));
                        }
                    }
                    PowerUpKind::EarnOneLife => {
                        lives.0 = lives.0.saturating_add(1);
                    }
                    _ => unreachable!(),
                }

                // Apply the new size immediately; otherwise the sprite and its
                // collision AABB keep the old size until the next window resize.
                paddle_transform.scale = paddle.size(min_window_dimension);
            }
        }
    }
}
