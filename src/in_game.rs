use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy::math::bounding::{Bounded2d, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::time::common_conditions::on_timer;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{Despawn, despawn_all_walls, GameState, Mass, Player, reset_score, reset_sprite, unpause_time, Velocity, Wall};

const WALL_INTERVAL: Duration = Duration::from_millis(1500);

const RANDOM_SEED: u64 = 42;

pub fn plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::InProgress), (unpause_time, reset_score, reset_sprite, despawn_all_walls, reset_hole_info, reset_rng))
        .add_systems(FixedUpdate, (gravity, hit_ground, move_walls, update_bounding_circle, hit_wall, cleared_wall).run_if(in_state(GameState::InProgress)))
        .add_systems(FixedUpdate, spawn_wall.run_if(in_state(GameState::InProgress).and_then(on_timer(WALL_INTERVAL))))
        .add_systems(Update, flap.run_if(in_state(GameState::InProgress).and_then(input_just_pressed(MouseButton::Left))))
        .insert_resource(RNG(ChaCha8Rng::seed_from_u64(RANDOM_SEED)))
        .insert_resource(PreviousHole::default());
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

const IMPULSE: f32 = 6.0;

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

#[derive(Resource, Default)]
struct PreviousHole {
    height: f32,
    index: u32,
}

fn reset_hole_info(
    mut previous_hole: ResMut<PreviousHole>
) {
    let default = PreviousHole::default();
    previous_hole.height = default.height;
    previous_hole.index = default.index;
}

#[derive(Resource)]
struct RNG(ChaCha8Rng);

fn reset_rng(
    mut rng: ResMut<RNG>,
) {
    rng.0 = ChaCha8Rng::seed_from_u64(RANDOM_SEED)
}

/// Walls are spawned with a hole size and a hole height `h`.
///
/// Holes have a randomly-generated size within [`min_hole_size`, `max_hole_size`].
/// As time increases, `min_hole_size` and `max_hole_size` both decrease, to a minimum of 110% sprite diameter.
///
/// Holes have an `h` which is a distance `delta_h` from the previous hole, `h = h_prev +/- delta_h`.
/// Holes have a randomly-generated `delta_h` within [0, `delta_h_max`].
/// As time increases, `delta_h_max` increases, but `h` is always clamped to fall within the window.
fn spawn_wall(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
    player: Query<&Player>,
    mut previous_hole: ResMut<PreviousHole>,
    mut rng: ResMut<RNG>,
) {
    let window = windows.single();
    let sprite_diameter = player.single().bounding_circle.radius() * 2.0;
    let hole_index = previous_hole.index as f32;

    // minimum hole width starts at 3x diameter and decreases over time
    // maximum hole width starts at 5x diameter and decreases over time
    let min_hole_size = (sprite_diameter * 1.1) + (sprite_diameter * 1.9) / (1.0 + hole_index / 10.0);
    let max_hole_size = (sprite_diameter * 1.1) + (sprite_diameter * 3.9) / (1.0 + hole_index / 10.0);

    let half_hole_size = rng.0.gen_range(min_hole_size..=max_hole_size) / 2.0;

    // maximum delta_h starts near 0 and increases toward window height over time
    let delta_h_max = window.height() * hole_index / (10.0 + hole_index);
    let delta_h = rng.0.gen_range(-delta_h_max..=delta_h_max);

    // the hole should never extend beyond the window
    let half_window_height = window.height() / 2.0;
    let h_limit = half_window_height - half_hole_size - 20.0;
    let h = (previous_hole.height + delta_h).clamp(-h_limit, h_limit);

    info!("hole: {} -> {}", h - half_hole_size, h + half_hole_size);

    previous_hole.height = h;
    previous_hole.index += 1;

    // let wall_height = window.height();
    let wall_spawn_x = window.width() / 2.0 + WALL_WIDTH;

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

    let top_wall = Transform::from_xyz(wall_spawn_x, half_window_height + half_hole_size + h, 0.0);
    wall(&mut commands, &mut meshes, &mut materials, top_wall, half_window_height * 2.0);

    let bottom_wall = Transform::from_xyz(wall_spawn_x, -half_window_height - half_hole_size + h, 0.0);
    wall(&mut commands, &mut meshes, &mut materials, bottom_wall, half_window_height * 2.0);
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