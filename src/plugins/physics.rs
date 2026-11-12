use crate::plugins::ball::Ball;
use crate::plugins::paddle::LoseLife;
use crate::plugins::walls::WallLocation;
use bevy::prelude::*;

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
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(&Collider, &Transform, Option<&WallLocation>), With<Collider>>,
    mut lose_life: MessageWriter<LoseLife>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();

    for (collider, collider_transform, wall_location) in &collider_query {
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
                ColliderType::PowerUp => todo!(),
            }
        }
    }
}
