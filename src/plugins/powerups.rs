use crate::plugins::{
    ball::Ball,
    paddle::GAP_BETWEEN_PADDLE_AND_FLOOR,
    paddle::Lives,
    paddle::Paddle,
    paddle::PaddleEliminated,
    paddle::PaddleLocation,
    physics::{Collider, ColliderType, Velocity},
};
use crate::theme;
use bevy::prelude::*;
use rand::RngExt;
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use strum::VariantArray;

const POWERUP_SIZE: f32 = 4.5; // As a percentage of the window's minimum dimension

const MAXIMUM_NUMBER_OF_ACTIVE_POWERUPS: usize = 4;

const TIME_BETWEEN_POWERUPS: f32 = 2.0;

pub struct PowerUpsPlugin;

impl Plugin for PowerUpsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(
            TIME_BETWEEN_POWERUPS,
            TimerMode::Repeating,
        )));
        app.add_systems(Startup, setup_power_up_assets);
        app.add_systems(Update, (spawn_power_up, handle_powerup_hit));
        app.add_message::<PowerUpHit>();
    }
}

#[derive(Component)]
#[require(Collider(ColliderType::PowerUp))]
pub struct PowerUp {
    pub kind: PowerUpKind,
}

#[derive(VariantArray, Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl PowerUpKind {
    /// Colour of the ring around the power-up badge, grouped by effect.
    fn color(self) -> Color {
        match self {
            Self::PaddleHalf | Self::PaddleDouble => theme::POWERUP_WIDTH,
            Self::PaddleSpeedHalf | Self::PaddleSpeedDouble => theme::POWERUP_PADDLE_SPEED,
            Self::BallSpeedHalf | Self::BallSpeedDouble => theme::POWERUP_BALL_SPEED,
            Self::EarnOneLife => theme::POWERUP_EARN_LIFE,
            Self::LoseOneLife => theme::POWERUP_LOSE_LIFE,
        }
    }
}

#[derive(Message)]
pub struct PowerUpHit {
    pub kind: PowerUpKind,
    pub paddle: PaddleLocation,
}

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct PowerUpAssets {
    badge_mesh: Handle<Mesh>,
    ring_mesh: Handle<Mesh>,
    badge_material: Handle<ColorMaterial>,
    glyph_material: Handle<ColorMaterial>,
    kinds: HashMap<PowerUpKind, PowerUpKindAssets>,
}

struct PowerUpKindAssets {
    glyph_mesh: Handle<Mesh>,
    ring_material: Handle<ColorMaterial>,
}

fn setup_power_up_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let kinds: HashMap<PowerUpKind, PowerUpKindAssets> = PowerUpKind::VARIANTS
        .iter()
        .map(|kind| {
            (
                *kind,
                PowerUpKindAssets {
                    glyph_mesh: glyph_mesh(*kind, &mut meshes),
                    ring_material: materials.add(ColorMaterial::from_color(kind.color())),
                },
            )
        })
        .collect();

    commands.insert_resource(PowerUpAssets {
        badge_mesh: meshes.add(Circle::new(0.5)),
        ring_mesh: meshes.add(Annulus::new(0.40, 0.5)),
        badge_material: materials.add(ColorMaterial::from_color(theme::POWERUP_BADGE)),
        glyph_material: materials.add(ColorMaterial::from_color(theme::POWERUP_GLYPH)),
        kinds,
    });
}

/// Each kind gets a distinct glyph so it can be recognised by shape as well as
/// by the ring colour.
fn glyph_mesh(kind: PowerUpKind, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    match kind {
        PowerUpKind::PaddleDouble => meshes.add(Rectangle::new(0.62, 0.14)),
        PowerUpKind::PaddleHalf => meshes.add(Rectangle::new(0.32, 0.14)),
        PowerUpKind::PaddleSpeedDouble => meshes.add(Triangle2d::new(
            Vec2::new(0.34, 0.0),
            Vec2::new(-0.26, 0.30),
            Vec2::new(-0.26, -0.30),
        )),
        PowerUpKind::PaddleSpeedHalf => meshes.add(Triangle2d::new(
            Vec2::new(0.22, 0.0),
            Vec2::new(-0.17, 0.20),
            Vec2::new(-0.17, -0.20),
        )),
        PowerUpKind::BallSpeedDouble => meshes.add(Circle::new(0.22)),
        PowerUpKind::BallSpeedHalf => meshes.add(Annulus::new(0.11, 0.22)),
        PowerUpKind::EarnOneLife => meshes.add(Rhombus::new(0.40, 0.55)),
        PowerUpKind::LoseOneLife => meshes.add(Triangle2d::new(
            Vec2::new(0.0, -0.32),
            Vec2::new(0.33, 0.22),
            Vec2::new(-0.33, 0.22),
        )),
    }
}

fn spawn_power_up(
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    mut commands: Commands,
    current_power_ups: Query<&PowerUp>,
    assets: Res<PowerUpAssets>,
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

        let size = POWERUP_SIZE * min_window_dimension / 100.0;

        spawn_power_up_at(
            &mut commands,
            &assets,
            powerup_kind,
            Vec3::new(powerup_x, powerup_y, 0.9),
            size,
        );
    }
}

/// Spawns a collectible badge at `position` scaled to `size`.
///
/// The root carries the collider so the collision box stays independent of the
/// decorative children.
fn spawn_power_up_at(
    commands: &mut Commands,
    assets: &PowerUpAssets,
    kind: PowerUpKind,
    position: Vec3,
    size: f32,
) {
    let kind_assets = assets.kinds.get(&kind).unwrap();

    commands
        .spawn((
            Transform {
                translation: position,
                scale: Vec3::new(size, size, 1.0),
                ..default()
            },
            Visibility::default(),
            Collider(ColliderType::PowerUp),
            PowerUp { kind },
        ))
        .with_children(|parent| {
            // Dark badge to separate the power-up from the arena.
            parent.spawn((
                Mesh2d(assets.badge_mesh.clone()),
                MeshMaterial2d(assets.badge_material.clone()),
            ));
            // Coloured ring identifying the category.
            parent.spawn((
                Mesh2d(assets.ring_mesh.clone()),
                MeshMaterial2d(kind_assets.ring_material.clone()),
                Transform::from_xyz(0.0, 0.0, 0.01),
            ));
            // White glyph identifying the specific effect.
            parent.spawn((
                Mesh2d(kind_assets.glyph_mesh.clone()),
                MeshMaterial2d(assets.glyph_material.clone()),
                Transform::from_xyz(0.0, 0.0, 0.02),
            ));
        });
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
