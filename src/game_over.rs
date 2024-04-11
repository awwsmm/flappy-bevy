use bevy::prelude::*;

use crate::{Despawn, GameState, reset_sprite, Wall};

pub fn plugin(app: &mut App) {
    app
        .add_event::<Restart>()
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .add_systems(Update, restart_button.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (restart_game, reset_sprite).chain().run_if(in_state(GameState::GameOver).and_then(on_event::<Restart>())));
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

#[derive(Component)]
struct GameOverMenu;

#[derive(Component)]
struct RestartButton;

fn game_over(
    mut time: ResMut<Time<Virtual>>,
    mut commands: Commands,
) {
    time.pause();

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.50).into(),
            ..default()
        },
        GameOverMenu
    )).with_children(|parent| {
        parent.spawn(
            NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(3.0)),
                    width: Val::Percent(50.0),
                    height: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                border_color: Color::BLACK.into(),
                background_color: Color::WHITE.into(),
                ..default()
            }
        ).with_children(|parent| {

            parent.spawn(
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }
            ).with_children(|parent| {
                parent.spawn(
                    TextBundle {
                        text: Text::from_section(
                            "Game Over :(",
                            TextStyle {
                                color: Color::BLACK,
                                font_size: 80.0,
                                ..default()
                            }
                        ),
                        ..default()
                    }
                );
            });

            parent.spawn(
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }
            ).with_children(|parent| {
                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(3.0)),
                            width: Val::Percent(80.0),
                            height: Val::Percent(80.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::rgba(0.0, 1.0, 0.0, 0.5).into(),
                        ..default()
                    },
                    RestartButton
                )).with_children(|parent| {
                    parent.spawn(
                        TextBundle {
                            text: Text::from_section(
                                "Start New Game",
                                TextStyle {
                                    color: Color::BLACK,
                                    font_size: 60.0,
                                    ..default()
                                }
                            ),
                            ..default()
                        }
                    );
                });
            });
        });
    });
}

#[derive(Event)]
struct Restart;

fn restart_button(
    mut button: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<RestartButton>)>,
    mut writer: EventWriter<Restart>,
    mut commands: Commands,
    game_over_menu: Query<Entity, With<GameOverMenu>>,
) {
    for (interaction, mut color) in &mut button {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgba(0.0, 1.0, 0.0, 1.0).into();
                writer.send(Restart);
                commands.entity(game_over_menu.single()).despawn_recursive();
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