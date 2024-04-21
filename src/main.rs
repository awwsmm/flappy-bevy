use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::math::bounding::{Aabb2d, Bounded2d, BoundingCircle};
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::window::WindowResized;
use bevy_pkv::PkvStore;

mod game_over;
mod new_game;
mod in_game;

// PkvStore is at
//   (macOS desktop) ~/Library/Application\ Support/awwsmm.flappy-bevy/bevy_pkv.redb
//   (macOS Chrome)  ~/Library/Application\ Support/Google/Chrome/Default/Local\ Storage/leveldb/
//       clear local storage in Developer Tools > Application > Local Storage

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never) // https://github.com/bevyengine/bevy/issues/10157#issuecomment-1849092112
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#html-canvas-id".into()),
                    resize_constraints: WindowResizeConstraints {
                        min_width: 800.0,
                        min_height: 600.0,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
        )
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Score::default())
        .init_state::<GameState>()
        .add_plugins((game_over::plugin, new_game::plugin, in_game::plugin))
        .add_systems(Startup, (setup, spawn_sprite, reset_sprite, load_high_score).chain())
        .add_systems(Update, lock_sprite_x_position.run_if(on_event::<WindowResized>()))
        .add_event::<Despawn>()
        .add_systems(Update, despawn.run_if(on_event::<Despawn>()))
        .insert_resource(PkvStore::new("awwsmm", "flappy-bevy"))
        .run();
}

fn despawn_all_walls(
    query: Query<Entity, With<Wall>>,
    mut writer: EventWriter<Despawn>,
) {
    for e in &query {
        writer.send(Despawn(e));
    }
}

#[derive(Component)]
struct Scores;

fn load_high_score(
    mut score: ResMut<Score>,
    mut pkv: ResMut<PkvStore>,
) {
    if let Ok(high_score) = pkv.get::<u64>("high score") {
        score.high = high_score;

    } else {
        pkv.set::<u64>("high score", &0)
            .expect("failed to store high score");
    }
}

fn setup(
    mut commands: Commands,
    score: Res<Score>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
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
        },
        Scores
    ));
}

#[derive(Component)]
struct AnimationConfig {
    sprite_indices: Vec<usize>,
    current_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

impl AnimationConfig {
    fn new(sprite_indices: Vec<usize>, fps: u8) -> Self {
        assert!(sprite_indices.len() > 0);
        Self {
            sprite_indices,
            current_sprite_index: 0,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

fn spawn_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("bird.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(64.0, 64.0), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let sprite_indices: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 0];

    let animation_config = AnimationConfig::new(sprite_indices, 24);

    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
            transform: Transform::from_scale(Vec3::splat(3.0)),
            ..default()
        },
        Mass,
        Velocity::default(),
        Player::default(),
        animation_config
    ));
}

fn reset_sprite(
    windows: Query<&Window>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut Player)>,
) {
    let window = windows.single();
    let (mut transform, mut velocity, mut player) = player.single_mut();

    let translation = Vec3::new(-window.width() / 2.0 + 2.0 * player.bounding_circle.circle.radius, 0.0, 0.0);

    transform.translation = translation;
    velocity.0 = Vec2::default();
    player.bounding_circle = Circle::new(64.).bounding_circle(translation.truncate(), 0.0);
}

fn lock_sprite_x_position(
    windows: Query<&Window>,
    mut player: Query<(&mut Transform, &mut Player)>,
) {
    let window = windows.single();
    let (mut transform, mut player) = player.single_mut();

    let x = -window.width() / 2.0 + 2.0 * player.bounding_circle.circle.radius;
    let y = player.bounding_circle.center.y;
    let translation = Vec3::new(x, y, 0.0);

    transform.translation = translation;
    player.bounding_circle = Circle::new(64.).bounding_circle(translation.truncate(), 0.0);
}

#[derive(Component)]
struct Mass;

#[derive(Component, Default)]
struct Velocity(Vec2);

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

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GameState {
    #[default]
    PreGame,
    InProgress,
    GameOver,
}

#[derive(Component)]
struct Wall {
    rectangle: Rectangle,
    center: Vec2,
    bounding_box: Aabb2d,
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

#[derive(Resource, Default)]
struct Score {
    high: u64,
    current: u64,
    stopwatch: Stopwatch,
}

fn pause_time(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause_time(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

fn reset_score(mut score: ResMut<Score>) {
    score.stopwatch.reset();
}

fn handle_button_event<B: Component, E: Default + Event, D: Component>(
    mut button: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<B>)>,
    mut writer: EventWriter<E>,
    mut commands: Commands,
    menu_to_despawn: Query<Entity, With<D>>,
) {
    for (interaction, mut color) in &mut button {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgba(0.0, 1.0, 0.0, 1.0).into();
                writer.send(E::default());
                commands.entity(menu_to_despawn.single()).despawn_recursive();
            }
            Interaction::Hovered => {
                *color = Color::rgba(0.0, 1.0, 0.0, 0.75).into();
            }
            Interaction::None => {
                *color = Color::rgba(0.0, 1.0, 0.0, 0.5).into();
            }
        }
    }
}