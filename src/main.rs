use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
        .add_systems(Update, (gravity, hit_ground).run_if(in_state(GameState::InProgress)))
        .add_systems(Update, flap.run_if(in_state(GameState::InProgress).and_then(input_just_pressed(KeyCode::Space))))
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
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("bevy.png"), // 256x256
            transform: Transform {
                translation: Vec3::new(-400.0, 0.0, 0.0),
                scale: Vec3::new(0.5, 0.5, 1.0), // 50% scale == 128x128
                ..default()
            },
            ..default()
        },
        Mass,
        Velocity::default(),
        Player,
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
struct Player;

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