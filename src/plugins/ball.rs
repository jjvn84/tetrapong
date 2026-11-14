use crate::plugins::{paddle::PaddleLocation, physics::Velocity};
use bevy::{prelude::*, window::WindowResized};

pub const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_DIAMETER: f32 = 3.0; // As a percentage of the window's minimum dimension
pub const BALL_SPEED: f32 = 50.0;

// Speed power-ups multiply the ball's speed, but never beyond these bounds.
const MIN_SPEED_MULTIPLIER: f32 = 1.0;
const MAX_SPEED_MULTIPLIER: f32 = 4.0;

const SERVE_PAUSE_SECONDS: f32 = 2.0;

const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (on_window_resize, tick_serve_pause));
    }
}

#[derive(Component)]
pub struct Ball {
    pub last_hit_by: Option<PaddleLocation>,
    speed_multiplier: f32,
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
}

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
    let initial_ball_angle = rand::random_range(0.0..360.0) as f32;
    let initial_ball_direction = Vec2::new(initial_ball_angle.cos(), initial_ball_angle.sin());

    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER * min_window_dimension / 100.0).extend(1.)),
        Ball {
            last_hit_by: None,
            speed_multiplier: 1.0,
        },
        Velocity(initial_ball_direction.normalize() * BALL_SPEED * min_window_dimension / 100.0),
    ));
}

fn on_window_resize(
    ball: Single<(&mut Transform, &mut Velocity, &Ball), With<Ball>>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    let (mut ball_transform, mut ball_velocity, ball) = ball.into_inner();
    for window in resize_reader.read() {
        let min_window_dimension = window.width.min(window.height);
        (ball_transform.translation.x, ball_transform.translation.y) = (0.0, 0.0);
        ball_transform.scale = Vec2::splat(BALL_DIAMETER * min_window_dimension / 100.0).extend(1.);
        let initial_ball_angle = rand::random_range(0.0..360.0) as f32;
        let initial_ball_direction = Vec2::new(initial_ball_angle.cos(), initial_ball_angle.sin());
        ball_velocity.0 = initial_ball_direction.normalize() * ball.speed(min_window_dimension);
    }
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
            let ball_angle = rand::random_range(0.0..360.0) as f32;
            let ball_direction = Vec2::new(ball_angle.cos(), ball_angle.sin());
            velocity.0 = ball_direction.normalize() * ball.speed(min_window_dimension);
            commands.entity(entity).remove::<ServePause>();
        }
    }
}
