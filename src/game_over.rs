use bevy::prelude::*;

use crate::{GameState, handle_button_event, pause_time};

pub fn plugin(app: &mut App) {
    app
        .add_systems(OnEnter(GameState::GameOver), (pause_time, game_over))
        .add_event::<Restart>()
        .add_systems(Update, handle_button_event::<RestartButton, Restart, GameOverMenu>.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, restart_game.run_if(in_state(GameState::GameOver).and_then(on_event::<Restart>())))
        .add_event::<BackToMenu>()
        .add_systems(Update, handle_button_event::<BackToMenuButton, BackToMenu, GameOverMenu>.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, back_to_menu.run_if(in_state(GameState::GameOver).and_then(on_event::<BackToMenu>())));
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