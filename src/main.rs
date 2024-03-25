use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup)
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
    commands.spawn(
        SpriteBundle {
            texture: asset_server.load("bevy.png"), // 256x256
            transform: Transform {
                translation: Vec3::new(-400.0, 0.0, 0.0),
                scale: Vec3::new(0.5, 0.5, 1.0), // 50% scale == 128x128
                ..default()
            },
            ..default()
        },
    );
}