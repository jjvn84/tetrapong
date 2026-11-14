use crate::plugins::ball::Ball;
use crate::plugins::paddle::{LoseLife, Paddle};
use crate::plugins::powerups::{PowerUp, PowerUpHit};
use crate::plugins::walls::WallLocation;
use bevy::prelude::*;
use core::f32::consts::PI;

const MAX_BOUNCE_ANGLE: f32 = PI / 3.0; // 60 degrees

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (apply_velocity, check_for_collisions)
                // `chain`ing systems together runs them in order
                .chain(),
        );
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default)]
pub struct Collider(pub ColliderType);

pub enum ColliderType {
    Wall,
    Paddle,
    PowerUp,
}

impl Default for ColliderType {
    fn default() -> Self {
        ColliderType::Wall
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn check_for_collisions(
    mut commands: Commands,
    ball_query: Single<(&mut Ball, &mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<
        (
            Entity,
            &Collider,
            &Transform,
            Option<&WallLocation>,
            Option<&Paddle>,
            Option<&PowerUp>,
        ),
        With<Collider>,
    >,
    mut lose_life: MessageWriter<LoseLife>,
    mut powerup_hit: MessageWriter<PowerUpHit>,
) {
    let (mut ball, mut ball_velocity, ball_transform) = ball_query.into_inner();

    for (collider_entity, collider, collider_transform, wall_location, paddle, power_up) in
        &collider_query
    {
        let collision = ball_collision(
            BoundingCircle::new(
                ball_transform.translation.truncate(),
                ball_transform.scale.x / 2.,
            ),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            match collider.0 {
                ColliderType::Wall => {
                    if let Some(wall_location) = wall_location {
                        lose_life.write(LoseLife(wall_location.owner()));
                    }
                    // Reflect the ball's velocity when it collides
                    let mut reflect_x = false;
                    let mut reflect_y = false;

                    // Reflect only if the velocity is in the opposite direction of the collision
                    // This prevents the ball from getting stuck inside the bar
                    match collision {
                        Collision::Left => reflect_x = ball_velocity.x > 0.0,
                        Collision::Right => reflect_x = ball_velocity.x < 0.0,
                        Collision::Top => reflect_y = ball_velocity.y < 0.0,
                        Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                    }

                    // Reflect velocity on the x-axis if we hit something on the x-axis
                    if reflect_x {
                        ball_velocity.x = -ball_velocity.x;
                    }

                    // Reflect velocity on the y-axis if we hit something on the y-axis
                    if reflect_y {
                        ball_velocity.y = -ball_velocity.y;
                    }
                }
                ColliderType::Paddle => {
                    // `Collision` names the face of the paddle that was hit.
                    // `normal` points away from that face; `tangent` runs along it.
                    let (normal, tangent) = match collision {
                        Collision::Top => (Vec2::Y, Vec2::X),
                        Collision::Bottom => (Vec2::NEG_Y, Vec2::X),
                        Collision::Left => (Vec2::NEG_X, Vec2::Y),
                        Collision::Right => (Vec2::X, Vec2::Y),
                    };

                    // Only bounce when the ball is moving into the paddle.
                    if ball_velocity.0.dot(normal) < 0.0 {
                        if let Some(paddle) = paddle {
                            ball.last_hit_by = Some(paddle.location);
                        }

                        // Half the paddle's length along the tangent axis.
                        let half_extent = 0.5
                            * (collider_transform.scale.x * tangent.x.abs()
                                + collider_transform.scale.y * tangent.y.abs());

                        // -1 = one edge, 0 = centre, +1 = the other edge.
                        let offset = ((ball_transform.translation.truncate()
                            - collider_transform.translation.truncate())
                        .dot(tangent)
                            / half_extent)
                            .clamp(-1.0, 1.0);

                        // Centre hits leave perpendicular to the paddle;
                        // edge hits rotate up to MAX_BOUNCE_ANGLE.
                        let angle = offset * MAX_BOUNCE_ANGLE;
                        let direction = normal * angle.cos() + tangent * angle.sin();

                        // Preserve the ball's current speed.
                        ball_velocity.0 = direction * ball_velocity.0.length();
                    }
                }
                ColliderType::PowerUp => {
                    if let (Some(last_hit_by), Some(power_up)) = (ball.last_hit_by, power_up) {
                        powerup_hit.write(PowerUpHit {
                            kind: power_up.kind,
                            paddle: last_hit_by,
                        });
                        commands.entity(collider_entity).despawn();
                    }
                }
            }
        }
    }
}
