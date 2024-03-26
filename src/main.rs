use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy::math::bounding::{Aabb2d, Bounded2d, BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::time::common_conditions::on_real_timer;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
        .add_systems(Update, (gravity, hit_ground, move_walls, update_bounding_circle, hit_wall).run_if(in_state(GameState::InProgress)))
        .add_systems(Update, flap.run_if(in_state(GameState::InProgress).and_then(input_just_pressed(KeyCode::Space))))
        .add_systems(Update, spawn_wall.run_if(in_state(GameState::InProgress).and_then(on_real_timer(Duration::from_millis(1500)))))
        .init_state::<GameState>()
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());
    spawn_sprite(commands, asset_server);
}

fn spawn_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let translation = Vec3::new(-400.0, 0.0, 0.0);

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("bevy.png"), // 256x256
            transform: Transform {
                translation,
                scale: Vec3::new(0.5, 0.5, 1.0), // 50% scale == 128x128
                ..default()
            },
            ..default()
        },
        Mass,
        Velocity::default(),
        Player { bounding_circle: Circle::new(64.).bounding_circle(translation.truncate(), 0.0) },
    ));
}

#[derive(Component)]
struct Mass;

#[derive(Component, Default)]
struct Velocity(Vec2);

const GRAVITY: f32 = -0.1;

fn gravity(
    mut query: Query<(&mut Velocity, &mut Transform), With<Mass>>,
) {
    for (mut velocity, mut transform) in query.iter_mut() {
        velocity.0.y += GRAVITY;
        transform.translation += velocity.0.extend(0.0);
    }
}

#[derive(Component)]
struct Player {
    bounding_circle: BoundingCircle,
}

const IMPULSE: f32 = 2.0;

fn flap(
    mut player: Query<&mut Velocity, With<Player>>,
) {
    let mut velocity = player.single_mut();
    velocity.0.y = IMPULSE;
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GameState {
    #[default]
    InProgress,
    GameOver,
}

fn hit_ground(
    mut player: Query<&Transform, With<Player>>,
    windows: Query<&Window>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let transform = player.single_mut();
    let window = windows.single();

    if transform.translation.y < -window.height() / 2.0 + 64.0 {
        next_state.set(GameState::GameOver);
    }
}

#[derive(Component)]
struct Wall {
    rectangle: Rectangle,
    bounding_box: Aabb2d,
}

const WALL_WIDTH: f32 = 100.0;
const WALL_HEIGHT: f32 = 1000.0;
const HOLE_SIZE: f32 = 300.0;
const WALL_SPAWN_X: f32 = 800.0;

fn spawn_wall(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    fn wall(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        transform: Transform,
    ) {
        let rectangle = Rectangle::new(WALL_WIDTH, WALL_HEIGHT);

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(rectangle)),
                material: materials.add(Color::RED),
                transform,
                ..default()
            },
            Wall {
                rectangle,
                bounding_box: rectangle.aabb_2d(transform.translation.truncate(), 0.0),
            }
        ));
    }

    let top_wall = Transform::from_xyz(WALL_SPAWN_X, WALL_HEIGHT-500.0 + HOLE_SIZE / 2.0, 0.0);
    wall(&mut commands, &mut meshes, &mut materials, top_wall);

    let bottom_wall = Transform::from_xyz(WALL_SPAWN_X, -500.0 - HOLE_SIZE / 2.0, 0.0);
    wall(&mut commands, &mut meshes, &mut materials, bottom_wall);
}

const WALL_SPEED: f32 = -2.0;

fn move_walls(
    mut walls: Query<(&mut Transform, &mut Wall)>,
) {
    for (mut transform, mut wall) in walls.iter_mut() {
        transform.translation.x += WALL_SPEED;
        wall.bounding_box = wall.rectangle.aabb_2d(transform.translation.truncate(), 0.0);
    }
}

fn update_bounding_circle(
    mut player: Query<(&Transform, &mut Player)>,
) {
    let (transform, mut player) = player.single_mut();
    player.bounding_circle = Circle::new(64.).bounding_circle(transform.translation.truncate(), 0.0);
}

fn hit_wall(
    player: Query<&Player>,
    mut next_state: ResMut<NextState<GameState>>,
    walls: Query<&Wall>,
) {
    let player = player.single();

    for wall in walls.iter() {
        if player.bounding_circle.intersects(&wall.bounding_box) {
            next_state.set(GameState::GameOver);
        }
    }
}