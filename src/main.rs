use bevy::math::bounding::{Aabb2d, Bounded2d, BoundingCircle};
use bevy::prelude::*;
use bevy::time::Stopwatch;

mod game_over;
mod new_game;
mod in_game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Score::default())
        .init_state::<GameState>()
        .add_plugins((game_over::plugin, new_game::plugin, in_game::plugin))
        .add_systems(Startup, (setup, spawn_sprite, reset_sprite).chain())
        .add_systems(Update, track_high_score)
        .add_event::<Despawn>()
        .add_systems(Update, despawn.run_if(on_event::<Despawn>()))
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

fn track_high_score(
    mut score: ResMut<Score>,
    time: Res<Time>,
    mut text: Query<&mut Text, With<Scores>>,
) {
    score.stopwatch.tick(time.delta());
    score.current = score.stopwatch.elapsed().as_secs();
    score.high = score.current.max(score.high);

    let mut text = text.single_mut();
    text.sections[0].value = format!("High Score: {}\nCurrent Score: {}", score.high, score.current);
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