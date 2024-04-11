use bevy::prelude::*;

use crate::{Despawn, GameState, reset_sprite, Wall};

pub fn plugin(app: &mut App) {
    app
        .add_event::<Restart>()
        .add_event::<BackToMenu>()
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .add_systems(Update, button_event::<RestartButton, Restart>.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, button_event::<BackToMenuButton, BackToMenu>.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (despawn_all_walls, restart_game, reset_sprite).run_if(in_state(GameState::GameOver).and_then(on_event::<Restart>())))
        .add_systems(Update, (despawn_all_walls, back_to_menu, reset_sprite).run_if(in_state(GameState::GameOver).and_then(on_event::<BackToMenu>())));
}

fn despawn_all_walls(
    query: Query<Entity, With<Wall>>,
    mut writer: EventWriter<Despawn>,
) {
    for e in &query {
        writer.send(Despawn(e));
    }
}

fn restart_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InProgress);
}

fn back_to_menu(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::PreGame);
}

#[derive(Component)]
struct GameOverMenu;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct BackToMenuButton;

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
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
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
                            height: Val::Percent(40.0),
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

                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(3.0)),
                            width: Val::Percent(80.0),
                            height: Val::Percent(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::rgba(0.0, 1.0, 0.0, 0.5).into(),
                        ..default()
                    },
                    BackToMenuButton
                )).with_children(|parent| {
                    parent.spawn(
                        TextBundle {
                            text: Text::from_section(
                                "Back to Menu",
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

#[derive(Event, Default)]
struct Restart;

#[derive(Event, Default)]
struct BackToMenu;

fn button_event<B: Component, E: Default + Event>(
    mut button: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<B>)>,
    mut writer: EventWriter<E>,
    mut commands: Commands,
    game_over_menu: Query<Entity, With<GameOverMenu>>,
) {
    for (interaction, mut color) in &mut button {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgba(0.0, 1.0, 0.0, 1.0).into();
                writer.send(E::default());
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