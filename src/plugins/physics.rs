use crate::plugins::ball::Ball;
use crate::plugins::paddle::{LoseLife, Paddle};
use crate::plugins::powerups::{PowerUp, PowerUpHit};
use crate::plugins::walls::WallLocation;
use bevy::prelude::*;
use core::f32::consts::PI;

const MAX_BOUNCE_ANGLE: f32 = PI / 3.0; // 60 degrees

// The ball moves in substeps so a fast ball can't skip the thin arena walls
// between two collision checks (tunneling). Each substep advances it by at
// most a fraction of its radius, and a long frame (window drag, breakpoint,
// alt-tab) is capped so it can't teleport across the arena.
const MAX_STEP_RADIUS_FRACTION: f32 = 0.5;
const MAX_FRAME_DELTA: f32 = 0.1;
const MAX_SUBSTEPS: u32 = 128;

#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
pub struct PhysicsSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_ball.in_set(PhysicsSet));
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

/// Moves the ball for this frame and resolves collisions after every substep.
///
/// Checking collisions only once, after the whole frame's movement, lets a
/// fast ball pass completely through a wall between two frames. Substepping
/// keeps each movement slice smaller than the ball, so the ball always
/// overlaps whatever it crosses.
fn move_ball(
    mut commands: Commands,
    time: Res<Time>,
    ball_query: Single<(&mut Ball, &mut Velocity, &mut Transform), With<Ball>>,
    collider_query: Query<
        (
            Entity,
            &Collider,
            &Transform,
            Option<&WallLocation>,
            Option<&Paddle>,
            Option<&PowerUp>,
        ),
        (With<Collider>, Without<Ball>),
    >,
    mut lose_life: MessageWriter<LoseLife>,
    mut power_up_hit: MessageWriter<PowerUpHit>,
) {
    let (mut ball, mut ball_velocity, mut ball_transform) = ball_query.into_inner();

    let delta = time.delta_secs().min(MAX_FRAME_DELTA);
    let speed = ball_velocity.length();
    if speed <= f32::EPSILON || delta <= f32::EPSILON {
        return;
    }

    let radius = ball_transform.scale.x / 2.0;
    let substeps = substep_count(speed, delta, radius);
    let substep_delta = delta / substeps as f32;

    // Collected power-ups are despawned with deferred commands, so the query
    // still returns them for the rest of this frame.
    let mut collected = Vec::new();

    for _ in 0..substeps {
        ball_transform.translation.x += ball_velocity.x * substep_delta;
        ball_transform.translation.y += ball_velocity.y * substep_delta;

        let mut hit_wall = false;

        for (collider_entity, collider, collider_transform, wall_location, paddle, power_up) in
            &collider_query
        {
            // Skip if player already collected this power up
            if collected.contains(&collider_entity) {
                continue;
            }

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

            // Skip if there is no collision with a Collider
            let Some(collision) = collision else {
                continue;
            };

            match collider.0 {
                ColliderType::Wall => {
                    hit_wall = true;

                    if let Some(wall_location) = wall_location {
                        lose_life.write(LoseLife {
                            paddle_location: wall_location.owner(),
                            from_wall_hit: true,
                        });
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
                        power_up_hit.write(PowerUpHit {
                            kind: power_up.kind,
                            paddle: last_hit_by,
                        });
                        collected.push(collider_entity);
                        commands.entity(collider_entity).despawn();
                    }
                }
            }
        }

        // A wall hit scores a point, and the ball is reset later this frame,
        // so there is nothing left to simulate.
        if hit_wall {
            break;
        }
    }
}

/// Number of substeps needed so the ball never moves more than a fraction of
/// its radius between collision checks.
fn substep_count(speed: f32, delta: f32, radius: f32) -> u32 {
    let max_step = (radius * MAX_STEP_RADIUS_FRACTION).max(0.5);
    ((speed * delta / max_step).ceil() as u32).clamp(1, MAX_SUBSTEPS)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_ball_moves_less_than_half_its_radius_per_substep() {
        // Worst case for the current rules: 4x speed on a 1080p window, after
        // a frame long enough to hit the delta cap.
        let radius = 16.2;
        let speed = 2160.0;
        let delta = MAX_FRAME_DELTA;

        let steps = substep_count(speed, delta, radius);

        assert!(steps > 1);
        assert!(speed * delta / steps as f32 <= radius * MAX_STEP_RADIUS_FRACTION + 1e-3);
    }

    #[test]
    fn slow_ball_uses_a_single_substep() {
        assert_eq!(substep_count(10.0, 1.0 / 60.0, 12.0), 1);
    }

    #[test]
    fn substep_count_is_always_at_least_one() {
        assert_eq!(substep_count(0.0, 0.0, 0.0), 1);
    }
}
