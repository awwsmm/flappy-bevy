use bevy::prelude::*;

use crate::{GameState, reset_sprite};

pub fn plugin(app: &mut App) {
    app
        .add_event::<NewGame>()
        .add_systems(OnEnter(GameState::PreGame), pre_game)
        .add_systems(Update, new_game_button.run_if(in_state(GameState::PreGame)))
        .add_systems(Update, (start_game, reset_sprite).chain().run_if(in_state(GameState::PreGame).and_then(on_event::<NewGame>())));
}

fn start_game(
    mut next_state: ResMut<NextState<GameState>>,
) {
    next_state.set(GameState::InProgress);
}

#[derive(Component)]
struct NewGameMenu;

#[derive(Component)]
struct NewGameButton;

fn pre_game(
    mut commands: Commands,
) {
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
        NewGameMenu
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
                            "Flappy Bevy",
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
                    NewGameButton
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
struct NewGame;

fn new_game_button(
    mut button: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<NewGameButton>)>,
    mut writer: EventWriter<NewGame>,
    mut commands: Commands,
    new_game_menu: Query<Entity, With<NewGameMenu>>,
) {
    for (interaction, mut color) in &mut button {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::rgba(0.0, 1.0, 0.0, 1.0).into();
                writer.send(NewGame);
                commands.entity(new_game_menu.single()).despawn_recursive();
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