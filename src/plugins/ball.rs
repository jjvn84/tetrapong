use crate::game_state::GameState;
use crate::plugins::paddle::{PaddleLocation, PaddleSet};
use crate::plugins::physics::{PhysicsSet, Velocity};
use crate::theme;
use bevy::{prelude::*, window::WindowResized};

pub const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_DIAMETER: f32 = 3.0; // As a percentage of the window's minimum dimension
pub const BALL_SPEED: f32 = 50.0; // As a percentage of the window's minimum dimension

// Speed power-ups multiply the ball's speed, but never beyond these bounds.
const MIN_SPEED_MULTIPLIER: f32 = 1.0;
const MAX_SPEED_MULTIPLIER: f32 = 4.0;

const SERVE_PAUSE_SECONDS: f32 = 2.0;

#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Clone)]
pub struct BallSet;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BallReset>();
        app.add_systems(OnEnter(GameState::Game), setup);
        // `reset_ball` must land after the point is scored (PaddleSet) and before
        // the ball is next moved (PhysicsSet), so a parked ball is never advanced
        // with the reflected velocity from the wall hit.
        app.add_systems(
            Update,
            ball_reset_on_resize.before(reset_ball).in_set(BallSet),
        );
        app.add_systems(Update, tick_serve_pause.before(reset_ball).in_set(BallSet));
        app.add_systems(
            Update,
            reset_ball
                .after(PaddleSet)
                .before(PhysicsSet)
                .in_set(BallSet),
        );
    }
}

#[derive(Component, Copy, Clone)]
pub struct Ball {
    pub last_hit_by: Option<PaddleLocation>,
    pub speed_multiplier: f32,
}

impl Ball {
    /// Multiplies the ball's speed, clamping it to `0.5..=4.0` times the
    /// original speed so repeated power-ups can't stall or launch the ball.
    pub fn multiply_speed(&mut self, factor: f32) {
        self.speed_multiplier =
            (self.speed_multiplier * factor).clamp(MIN_SPEED_MULTIPLIER, MAX_SPEED_MULTIPLIER);
    }

    /// Current speed in pixels per second for the given window size.
    pub fn speed(&self, min_window_dimension: f32) -> f32 {
        BALL_SPEED * self.speed_multiplier * min_window_dimension / 100.0
    }

    fn initial_ball_direction() -> Vec2 {
        // Pick a quadrant (0..4) and a safe offset (15°..75°)
        let quadrant = rand::random_range(0..4) as f32;
        let offset = rand::random_range(15.0..75.0);
        let angle_radians = (quadrant * 90.0 + offset).to_radians();
        Vec2::from_angle(angle_radians)
    }
}

/// Sent whenever the ball must return to the centre: after a point or a resize.
///
/// `reset_ball` is the only handler, so the ball is always parked, resized and
/// served the same way regardless of what triggered the reset.
#[derive(Message)]
pub struct BallReset;

#[derive(Component)]
pub struct ServePause(pub Timer);

impl ServePause {
    pub fn new() -> Self {
        Self(Timer::from_seconds(SERVE_PAUSE_SECONDS, TimerMode::Once))
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&mut Window>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let min_window_dimension = window.width().min(window.height());

    // Ball
    let ball_struct = Ball {
        last_hit_by: None,
        speed_multiplier: 1.0,
    };

    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(theme::BALL)),
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER * min_window_dimension / 100.0).extend(1.)),
        ball_struct,
        Velocity(
            Ball::initial_ball_direction().normalize() * ball_struct.speed(min_window_dimension),
        ),
        // The ball belongs to the round: it is rebuilt when a new round starts.
        DespawnOnExit(GameState::Game),
    ));
}

fn ball_reset_on_resize(
    mut resize_reader: MessageReader<WindowResized>,
    mut reset_writer: MessageWriter<BallReset>,
) {
    if resize_reader.read().next().is_some() {
        reset_writer.write(BallReset);
    }
}

/// Parks the ball at the centre, restores its size for the current window and
/// starts the serve pause. Both points and window resizes go through here.
fn reset_ball(
    mut commands: Commands,
    mut reset_reader: MessageReader<BallReset>,
    ball: Single<(Entity, &mut Ball, &mut Transform, &mut Velocity), With<Ball>>,
    window: Query<&Window>,
) {
    if reset_reader.read().next().is_none() {
        return;
    }
    let Ok(window) = window.single() else {
        return;
    };
    let min_window_dimension = window.width().min(window.height());

    let (entity, mut ball, mut ball_transform, mut ball_velocity) = ball.into_inner();
    ball_transform.translation = BALL_STARTING_POSITION;
    ball_transform.scale = Vec2::splat(BALL_DIAMETER * min_window_dimension / 100.0).extend(1.);
    ball_velocity.0 = Vec2::ZERO;
    ball.last_hit_by = None;
    ball.speed_multiplier = 1.0;

    commands.entity(entity).insert(ServePause::new());
}

fn tick_serve_pause(
    mut commands: Commands,
    time: Res<Time>,
    ball: Query<(Entity, &mut Velocity, &Ball, &mut ServePause), With<Ball>>,
    window: Query<&mut Window>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let min_window_dimension = window.width().min(window.height());

    for (entity, mut velocity, ball, mut pause) in ball {
        pause.0.tick(time.delta());

        if pause.0.just_finished() {
            velocity.0 =
                Ball::initial_ball_direction().normalize() * ball.speed(min_window_dimension);
            commands.entity(entity).remove::<ServePause>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;

    #[test]
    fn ball_reset_parks_the_ball_and_starts_a_serve_pause() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_plugins(BallPlugin);
        app.add_message::<WindowResized>();

        app.world_mut().spawn(Window::default());
        let ball = app
            .world_mut()
            .spawn((
                Ball {
                    last_hit_by: Some(PaddleLocation::Bottom),
                    speed_multiplier: 2.0,
                },
                Transform::from_xyz(10.0, 10.0, 1.0),
                Velocity(Vec2::X),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Messages<BallReset>>()
            .write(BallReset);
        app.update();

        let world = app.world();
        assert_eq!(
            world.get::<Transform>(ball).unwrap().translation,
            BALL_STARTING_POSITION
        );
        assert_eq!(world.get::<Velocity>(ball).unwrap().0, Vec2::ZERO);
        assert_eq!(world.get::<Ball>(ball).unwrap().last_hit_by, None);
        assert_eq!(world.get::<Ball>(ball).unwrap().speed_multiplier, 1.0);
        assert!(world.get::<ServePause>(ball).is_some());
    }
}
