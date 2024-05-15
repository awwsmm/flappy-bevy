use std::time::Duration;

use bevy::input::common_conditions::input_just_pressed;
use bevy::math::bounding::{Aabb2d, Bounded2d, IntersectsVolume};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_pkv::PkvStore;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{AnimationConfig, Despawn, despawn_all_walls, GameState, Mass, Player, reset_score, reset_sprite, Score, Scores, unpause_time, Velocity, Wall};

const WALL_INTERVAL: Duration = Duration::from_millis(1500);

const RANDOM_SEED: u64 = 42;

pub fn plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::InProgress), (unpause_time, reset_score, reset_sprite, despawn_all_walls, reset_hole_info, reset_rng))
        .add_systems(Update, (track_high_score, execute_animations).run_if(in_state(GameState::InProgress)))
        .add_systems(FixedUpdate, (gravity, hit_ground, move_walls, update_player_bounds, hit_wall, cleared_wall).run_if(in_state(GameState::InProgress)))
        .add_systems(FixedUpdate, spawn_wall.run_if(in_state(GameState::InProgress).and_then(on_timer(WALL_INTERVAL))))
        .add_systems(Update, flap.run_if(in_state(GameState::InProgress).and_then(input_just_pressed(MouseButton::Left).or_else(just_touched()))))
        // .add_systems(PostUpdate, debug_bounds)
        .insert_resource(RNG(ChaCha8Rng::seed_from_u64(RANDOM_SEED)))
        .insert_resource(PreviousHole::default());
}

pub fn just_touched() -> impl FnMut(Res<Touches>) -> bool {
    move |touch_input: Res<Touches>| touch_input.any_just_pressed()
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
    mut player: Query<(&mut Velocity, &Transform, &mut AnimationConfig), With<Player>>,
    windows: Query<&Window>,
) {
    let (mut velocity, position, mut animation) = player.single_mut();
    let window = windows.single();
    if position.translation.y < window.height() / 2.0 {
        velocity.0.y = IMPULSE;
    }
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
    animation.current_sprite_index = 0;
}

const WALL_WIDTH: f32 = 128.0;

fn update_player_bounds(
    mut player: Query<(&Transform, &mut Player)>,
) {
    let (transform, mut player) = player.single_mut();
    player.body.center = transform.translation.truncate() + Vec2::new(27.0, -27.0); // fine-tuned
    player.head.center = transform.translation.truncate() + Vec2::new(47.0, 25.0); // fine-tuned
}

// fn debug_bounds(
//     mut gizmos: Gizmos,
//     players: Query<&Player>,
//     walls: Query<&Wall>,
// ) {
//     let player = players.single();
//     gizmos.circle_2d(player.body.center, player.body.circle.radius, Color::RED);
//     gizmos.circle_2d(player.head.center, player.head.circle.radius, Color::RED);
//
//     for wall in walls.iter() {
//         let bottom_left = wall.bounding_box.min;
//         let top_right = wall.bounding_box.max;
//         let dimensions = top_right - bottom_left;
//         let center = (bottom_left + top_right) / 2.0;
//         gizmos.rect_2d(center, 0.0, dimensions, Color::RED);
//     }
// }

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

const TILE_SIZE: f32 = 128.0; // px

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
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    windows: Query<&Window>,
    player: Query<&Player>,
    mut previous_hole: ResMut<PreviousHole>,
    mut rng: ResMut<RNG>,
) {
    let texture = asset_server.load("wall.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(128.0, 128.0), 6, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // (x, y) position is at the center of the rectangle
    // x increases to the right, y increases to the top
    //
    //   +------------------------------------+
    //   |                     (x2, y2)       |
    //   |    (x1, y1)             |          |
    //   |        |           +----|----+     |
    //   |   +----|----+      |    v    |     |
    //   |   |    v    |      |    x    |     |
    //   |   |    x    |      |         |     |
    //   |   |         |      +---------+     |
    //   |   +---------+                      |
    //   |                  x1 < x2, y1 < y2  |
    //   +------------------------------------+

    fn spawn_bottom_wall(
        commands: &mut Commands,
        top_left_corner: Vec2,
        texture: Handle<Image>,
        texture_atlas_layout: Handle<TextureAtlasLayout>,
        half_window_height: f32
    ) {
        let wall_height = half_window_height + top_left_corner.y;

        commands.spawn((
            SpatialBundle {
                transform: Transform {
                    translation: (top_left_corner + Vec2::new(WALL_WIDTH / 2.0, -TILE_SIZE / 2.0)).extend(0.0),
                    ..default()
                },
                ..default()
            },
            Wall {
                rectangle: Rectangle::new(WALL_WIDTH, wall_height),
                center: top_left_corner + Vec2::new(WALL_WIDTH / 2.0, -wall_height / 2.0),
                bounding_box: Aabb2d::new(Vec2::ZERO, Vec2::ZERO),
            }
        )).with_children(|parent| {
            parent.spawn(
                SpriteSheetBundle {
                    texture: texture.clone(),
                    atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 5,
                    },
                    ..default()
                }
            );

            let mut bottom_of_wall = top_left_corner.y - TILE_SIZE;

            while bottom_of_wall > -half_window_height {
                parent.spawn(
                    SpriteSheetBundle {
                        texture: texture.clone(),
                        atlas: TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index: 3,
                        },
                        transform: Transform::from_translation(Vec3::Y * (bottom_of_wall - top_left_corner.y)),
                        ..default()
                    }
                );

                bottom_of_wall -= TILE_SIZE;
            }
        });
    }

    fn spawn_top_wall(
        commands: &mut Commands,
        bottom_left_corner: Vec2,
        texture: Handle<Image>,
        texture_atlas_layout: Handle<TextureAtlasLayout>,
        half_window_height: f32
    ) {
        let wall_height = half_window_height - bottom_left_corner.y;

        commands.spawn((
            SpatialBundle {
                transform: Transform {
                    translation: (bottom_left_corner + Vec2::splat(TILE_SIZE / 2.0)).extend(0.0),
                    ..default()
                },
                ..default()
            },
            Wall {
                rectangle: Rectangle::new(WALL_WIDTH, wall_height),
                center: bottom_left_corner + Vec2::new(WALL_WIDTH / 2.0, wall_height / 2.0),
                bounding_box: Aabb2d::new(Vec2::ZERO, Vec2::ZERO),
            }
        )).with_children(|parent| {
            parent.spawn(
                SpriteSheetBundle {
                    texture: texture.clone(),
                    atlas: TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index: 4,
                    },
                    ..default()
                }
            );

            let mut top_of_wall = bottom_left_corner.y + TILE_SIZE;

            while top_of_wall < half_window_height {
                parent.spawn(
                    SpriteSheetBundle {
                        texture: texture.clone(),
                        atlas: TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index: 3,
                        },
                        transform: Transform::from_translation(Vec3::Y * (top_of_wall - bottom_left_corner.y)),
                        ..default()
                    }
                );

                top_of_wall += TILE_SIZE;
            }
        });
    }

    let window = windows.single();
    let half_window_height = window.height() / 2.0;
    let half_window_width = window.width() / 2.0;

    let sprite_height = player.single().body.radius() * 5.0; // fine-tuned
    let hole_index = previous_hole.index as f32;

    // minimum hole width starts at 3x diameter and decreases to 1.1x over time
    // maximum hole width starts at 5x diameter and decreases to 1.1x over time
    let min_hole_size = (sprite_height * 1.1) + (sprite_height * 1.9) / (1.0 + hole_index / 10.0);
    let max_hole_size = (sprite_height * 1.1) + (sprite_height * 3.9) / (1.0 + hole_index / 10.0);

    let half_hole_size = rng.0.gen_range(min_hole_size..=max_hole_size.min(min_hole_size)) / 2.0;

    // maximum delta_h starts near 0 and increases toward window height over time
    let delta_h_max = window.height() * hole_index / (10.0 + hole_index);
    let delta_h = rng.0.gen_range(-delta_h_max..=delta_h_max);

    // the hole should never extend beyond the window
    let h_limit = (half_window_height - half_hole_size - 20.0).max(0.0);
    let h = (previous_hole.height + delta_h).clamp(-h_limit, h_limit);

    let bottom_of_hole = h - half_hole_size;
    let top_of_hole = h + half_hole_size;

    debug!("hole: {} -> {}", bottom_of_hole, top_of_hole);

    previous_hole.height = h;
    previous_hole.index += 1;

    let bottom_left_corner = Vec2::new(half_window_width + WALL_WIDTH, top_of_hole);
    spawn_top_wall(&mut commands, bottom_left_corner, texture.clone(), texture_atlas_layout.clone(), half_window_height);

    let top_left_corner = Vec2::new(half_window_width + WALL_WIDTH, bottom_of_hole);
    spawn_bottom_wall(&mut commands, top_left_corner, texture.clone(), texture_atlas_layout.clone(), half_window_height);
}

const WALL_SPEED: f32 = -4.0;

fn move_walls(
    mut walls: Query<(&mut Transform, &mut Wall)>,
) {
    for (mut transform, mut wall) in walls.iter_mut() {
        transform.translation.x += WALL_SPEED;
        wall.center.x += WALL_SPEED;
        wall.bounding_box = wall.rectangle.aabb_2d(wall.center, 0.0);
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

    if transform.translation.y < -window.height() / 2.0 + 2.0 * player.body.circle.radius { // fine-tuned
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
        if player.body.intersects(&wall.bounding_box) || player.head.intersects(&wall.bounding_box) {
            next_state.set(GameState::GameOver);
        }
    }
}

fn track_high_score(
    mut score: ResMut<Score>,
    time: Res<Time>,
    mut text: Query<&mut Text, With<Scores>>,
    mut pkv: ResMut<PkvStore>,
) {
    score.stopwatch.tick(time.delta());
    score.current = score.stopwatch.elapsed().as_secs();
    score.high = score.current.max(score.high);

    let mut text = text.single_mut();
    text.sections[0].value = format!("High Score: {}\nCurrent Score: {}", score.high, score.current);

    pkv.set("high score", &score.high).unwrap()
}

fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas)>,
) {
    for (mut config, mut atlas) in &mut query {
        config.frame_timer.tick(time.delta());
        if config.frame_timer.just_finished() {
            if config.current_sprite_index == config.sprite_indices.len() - 1 {
                atlas.index = 0;
                config.current_sprite_index = 0;
            } else {
                config.current_sprite_index += 1;
                atlas.index = config.sprite_indices[config.current_sprite_index];
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}