use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy::math::bounding::{Aabb2d, Bounded2d, BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::time::common_conditions::on_timer;
use bevy::time::Stopwatch;

const WALL_INTERVAL: Duration = Duration::from_millis(1500);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, (setup, spawn_sprite, reset_sprite).chain())
        .add_systems(Update, track_high_score.run_if(in_state(GameState::InProgress)))
        .add_systems(Update, flap.run_if(in_state(GameState::InProgress).and_then(input_just_pressed(KeyCode::Space))))
        .add_systems(Update, (restart_game, reset_sprite).chain().run_if(in_state(GameState::GameOver).and_then(input_just_pressed(KeyCode::Escape))))
        .add_event::<Despawn>()
        .add_systems(FixedUpdate, (gravity, hit_ground, move_walls, update_bounding_circle, hit_wall, cleared_wall).run_if(in_state(GameState::InProgress)))
        .add_systems(FixedUpdate, spawn_wall.run_if(in_state(GameState::InProgress).and_then(on_timer(WALL_INTERVAL))))
        .add_systems(FixedPostUpdate, despawn.run_if(on_event::<Despawn>()))
        .init_state::<GameState>()
        .insert_resource(Score::default())
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .add_systems(OnEnter(GameState::InProgress), new_game)
        .run();
}

fn setup(
    mut commands: Commands,
    score: Res<Score>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(
        TextBundle {
            text: Text::from_section(
                format!("High Score: {}\nCurrent Score: {}", score.high, score.current),
                TextStyle {
                    color: Color::BLACK,
                    font_size: 80.0,
                    ..default()
                }
            ),
            ..default()
        }
    );
}

fn spawn_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("bevy.png"), // 256x256
            transform: Transform {
                scale: Vec3::new(0.5, 0.5, 1.0), // 50% scale == 128x128  (64px radius)
                ..default()
            },
            ..default()
        },
        Mass,
        Velocity::default(),
        Player::default(),
    ));
}

fn reset_sprite(
    windows: Query<&Window>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut Player)>,
) {
    let window = windows.single();
    let translation = Vec3::new(-window.width() / 2.0 * 0.7, 0.0, 0.0);

    let (mut transform, mut velocity, mut player) = player.single_mut();

    transform.translation = translation;
    velocity.0 = Vec2::default();
    player.bounding_circle = Circle::new(64.).bounding_circle(translation.truncate(), 0.0);
}

#[derive(Component)]
struct Mass;

#[derive(Component, Default)]
struct Velocity(Vec2);

const GRAVITY: f32 = -0.2;

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

impl Default for Player {
    fn default() -> Self {
        Self {
            bounding_circle: BoundingCircle {
                center: Vec2::new(0.0, 0.0),
                circle: Circle::new(0.0),
            }
        }
    }
}

const IMPULSE: f32 = 4.0;

fn flap(
    mut player: Query<(&mut Velocity, &Transform), With<Player>>,
    windows: Query<&Window>,
) {
    let (mut velocity, position) = player.single_mut();
    let window = windows.single();
    if position.translation.y < window.height() / 2.0 {
        velocity.0.y = IMPULSE;
    }
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GameState {
    #[default]
    InProgress,
    GameOver,
}

fn hit_ground(
    mut player: Query<(&Transform, &Player)>,
    windows: Query<&Window>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (transform, player) = player.single_mut();
    let window = windows.single();

    if transform.translation.y < -window.height() / 2.0 + player.bounding_circle.circle.radius {
        next_state.set(GameState::GameOver);
    }
}

#[derive(Component)]
struct Wall {
    rectangle: Rectangle,
    bounding_box: Aabb2d,
}

const WALL_WIDTH: f32 = 100.0;

fn spawn_wall(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
    player: Query<&Player>
) {
    let window = windows.single();
    let player = player.single();

    let wall_height = window.height();
    let wall_spawn_x = window.width() / 2.0 + WALL_WIDTH;
    let minimum_hole_size = player.bounding_circle.radius() * 2.0;
    let hole_size = minimum_hole_size * 2.0;

    fn wall(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        transform: Transform,
        wall_height: f32,
    ) {
        let rectangle = Rectangle::new(WALL_WIDTH, wall_height);

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

    let top_wall = Transform::from_xyz(wall_spawn_x, (wall_height + hole_size) / 2.0, 0.0);
    wall(&mut commands, &mut meshes, &mut materials, top_wall, wall_height);

    let bottom_wall = Transform::from_xyz(wall_spawn_x,  -(wall_height + hole_size) / 2.0, 0.0);
    wall(&mut commands, &mut meshes, &mut materials, bottom_wall, wall_height);
}

const WALL_SPEED: f32 = -4.0;

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
    player.bounding_circle.center = transform.translation.truncate();
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

#[derive(Event)]
struct Despawn(Entity);

fn despawn(
    mut commands: Commands,
    mut reader: EventReader<Despawn>,
) {
    for despawn in reader.read() {
        commands.entity(despawn.0).despawn_recursive();
    }
}

fn restart_game(
    query: Query<Entity, With<Wall>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut writer: EventWriter<Despawn>,
) {
    for e in &query {
        writer.send(Despawn(e));
    }

    next_state.set(GameState::InProgress);
}

fn cleared_wall(
    entities: Query<(Entity, &Transform), With<Wall>>,
    window: Query<&Window>,
    mut writer: EventWriter<Despawn>,
) {
    let window = window.single();
    for (entity, transform) in entities.iter() {
        if transform.translation.x < -window.width() {
            writer.send(Despawn(entity));
        }
    }
}

#[derive(Resource, Default)]
struct Score {
    high: u64,
    current: u64,
    stopwatch: Stopwatch,
}

fn track_high_score(
    mut score: ResMut<Score>,
    time: Res<Time>,
    mut text: Query<&mut Text>,
) {
    score.stopwatch.tick(time.delta());
    score.current = score.stopwatch.elapsed().as_secs();
    score.high = score.current.max(score.high);

    let mut text = text.single_mut();
    text.sections[0].value = format!("High Score: {}\nCurrent Score: {}", score.high, score.current);
}

fn game_over(
    mut time: ResMut<Time<Virtual>>,
) {
    time.pause();
}

fn new_game(
    mut time: ResMut<Time<Virtual>>,
    mut score: ResMut<Score>,
) {
    score.stopwatch.reset();
    time.unpause();
}