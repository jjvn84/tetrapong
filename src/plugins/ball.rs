use crate::plugins::physics::Velocity;
use bevy::{prelude::*, window::WindowResized};

pub const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_DIAMETER: f32 = 3.0; // As a percentage of the window's minimum dimension
const BALL_SPEED: f32 = 400.0;

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
pub struct Ball;

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
    let min_window_dimension = window
        .single()
        .unwrap()
        .resolution
        .width()
        .min(window.single().unwrap().resolution.height());

    // Ball
    let initial_ball_angle = rand::random_range(0.0..360.0) as f32;
    let initial_ball_direction = Vec2::new(initial_ball_angle.cos(), initial_ball_angle.sin());

    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER * min_window_dimension / 100.0).extend(1.)),
        Ball,
        Velocity(initial_ball_direction.normalize() * BALL_SPEED),
    ));
}

fn on_window_resize(
    mut ball_transform: Single<&mut Transform, With<Ball>>,
    mut resize_reader: MessageReader<WindowResized>,
) {
    for window in resize_reader.read() {
        let min_window_dimension = window.width.min(window.height);
        (ball_transform.translation.x, ball_transform.translation.y) = (0.0, 0.0);
        ball_transform.scale = Vec2::splat(BALL_DIAMETER * min_window_dimension / 100.0).extend(1.);
    }
}

fn tick_serve_pause(
    mut commands: Commands,
    time: Res<Time>,
    ball: Query<(Entity, &mut Velocity, &mut ServePause), With<Ball>>,
) {
    for (entity, mut velocity, mut pause) in ball {
        pause.0.tick(time.delta());

        if pause.0.just_finished() {
            let ball_angle = rand::random_range(0.0..360.0) as f32;
            let ball_direction = Vec2::new(ball_angle.cos(), ball_angle.sin());
            velocity.0 = ball_direction.normalize() * BALL_SPEED;
            commands.entity(entity).remove::<ServePause>();
        }
    }
}
