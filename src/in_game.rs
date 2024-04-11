use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy::math::bounding::{Bounded2d, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::time::common_conditions::on_timer;

use crate::{Despawn, despawn_all_walls, GameState, Mass, Player, reset_score, reset_sprite, unpause_time, Velocity, Wall};

const WALL_INTERVAL: Duration = Duration::from_millis(1500);

pub fn plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::InProgress), (unpause_time, reset_score, reset_sprite, despawn_all_walls))
        .add_systems(FixedUpdate, (gravity, hit_ground, move_walls, update_bounding_circle, hit_wall, cleared_wall).run_if(in_state(GameState::InProgress)))
        .add_systems(FixedUpdate, spawn_wall.run_if(in_state(GameState::InProgress).and_then(on_timer(WALL_INTERVAL))))
        .add_systems(Update, flap.run_if(in_state(GameState::InProgress).and_then(input_just_pressed(MouseButton::Left))));
}

const GRAVITY: f32 = -0.2;

fn gravity(
    mut query: Query<(&mut Velocity, &mut Transform), With<Mass>>,
) {
    for (mut velocity, mut transform) in query.iter_mut() {
        velocity.0.y += GRAVITY;
        transform.translation += velocity.0.extend(0.0);
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

const WALL_WIDTH: f32 = 100.0;

fn update_bounding_circle(
    mut player: Query<(&Transform, &mut Player)>,
) {
    let (transform, mut player) = player.single_mut();
    player.bounding_circle.center = transform.translation.truncate();
}

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