use crate::campaign::{get_objective_summary, Campaign, MissionConfig};
use crate::components::*;
use crate::resources::*;
use crate::save_system::{has_save_file, load_game, save_game};
use crate::utils::play_tactical_sound;
use bevy::prelude::*;

// ==================== MISSION BRIEFING SYSTEM ====================

pub fn mission_briefing_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    campaign: Res<Campaign>,
    input: Res<Input<KeyCode>>,
    briefing_query: Query<Entity, With<MissionBriefing>>,
) {
    // Only show briefing when in MissionBriefing phase
    if game_state.game_phase == GamePhase::MissionBriefing {
        // Remove any existing briefing UI
        for entity in briefing_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Get current mission config
        let mission_config =
            crate::campaign::MissionConfig::get_mission_config(&campaign.progress.current_mission);

        // Create mission briefing UI
        create_mission_briefing_ui(&mut commands, &mission_config);

        // Check for input to start mission
        if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
            // Clear briefing UI
            for entity in briefing_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Start the actual mission
            game_state.game_phase = GamePhase::Preparation;
            play_tactical_sound(
                "radio",
                &format!("Mission: {} - Begin operation!", mission_config.name),
            );
        }
    } else {
        // Clean up any lingering briefing UI when not in briefing phase
        for entity in briefing_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ==================== MAIN MENU SYSTEM ====================

pub fn main_menu_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    input: Res<Input<KeyCode>>,
    menu_query: Query<Entity, With<SaveLoadMenu>>,
) {
    match game_state.game_phase {
        GamePhase::MainMenu => {
            // Remove any existing menu UI
            for entity in menu_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Create main menu UI
            create_main_menu_ui(&mut commands);

            // Handle input
            if input.just_pressed(KeyCode::Key1) {
                game_state.game_phase = GamePhase::MissionBriefing;
                play_tactical_sound("radio", "New campaign starting!");
            } else if input.just_pressed(KeyCode::Key2) && has_save_file() {
                game_state.game_phase = GamePhase::LoadMenu;
                play_tactical_sound("radio", "Accessing saved campaigns...");
            } else if input.just_pressed(KeyCode::Key3) {
                game_state.game_phase = GamePhase::SaveMenu;
                play_tactical_sound("radio", "Opening save menu...");
            }
        }
        GamePhase::SaveMenu => {
            // Handle save menu
            if menu_query.is_empty() {
                create_save_menu_ui(&mut commands);
            }

            if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
            } else if input.just_pressed(KeyCode::Key1) {
                // Save to slot 1
                if let Err(e) = save_game(&game_state) {
                    error!("Failed to save game: {}", e);
                    play_tactical_sound("radio", "Save failed!");
                } else {
                    play_tactical_sound("radio", "Game saved successfully!");
                    game_state.game_phase = GamePhase::MainMenu;
                }
            }
        }
        GamePhase::LoadMenu => {
            // Handle load menu
            if menu_query.is_empty() {
                create_load_menu_ui(&mut commands);
            }

            if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
            } else if input.just_pressed(KeyCode::Key1) && has_save_file() {
                // Load from slot 1
                match load_game() {
                    Ok(save_data) => {
                        *game_state = save_data.game_state;
                        play_tactical_sound(
                            "radio",
                            "Game loaded successfully! Resuming operation...",
                        );
                    }
                    Err(e) => {
                        error!("Failed to load game: {}", e);
                        play_tactical_sound("radio", "Load failed!");
                        game_state.game_phase = GamePhase::MainMenu;
                    }
                }
            }
        }
        _ => {
            // Clean up any lingering menu UI when not in menu phases
            for entity in menu_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// ==================== VICTORY/DEFEAT SYSTEM ====================

pub fn victory_defeat_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    campaign: Res<Campaign>,
    input: Res<Input<KeyCode>>,
    result_query: Query<Entity, Or<(With<VictoryScreen>, With<DefeatScreen>)>>,
) {
    match game_state.game_phase {
        GamePhase::Victory => {
            // Remove any existing result UI
            for entity in result_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Create victory screen
            create_victory_screen(&mut commands, &game_state, &campaign);

            // Handle input to continue
            if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
                advance_campaign_or_end(&mut game_state, &campaign);
            } else if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
                play_tactical_sound("radio", "Returning to main menu...");
            }
        }
        GamePhase::Defeat => {
            // Remove any existing result UI
            for entity in result_query.iter() {
                commands.entity(entity).despawn_recursive();
            }

            // Create defeat screen
            create_defeat_screen(&mut commands, &game_state, &campaign);

            // Handle input to continue
            if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
                // On defeat, return to main menu or retry
                game_state.game_phase = GamePhase::MainMenu;
                play_tactical_sound("radio", "Operation terminated. Regrouping...");
            } else if input.just_pressed(KeyCode::Escape) {
                game_state.game_phase = GamePhase::MainMenu;
                play_tactical_sound("radio", "Returning to main menu...");
            }
        }
        _ => {
            // Clean up any lingering result UI when not in victory/defeat phases
            for entity in result_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// ==================== UI CREATION HELPER FUNCTIONS ====================

fn create_mission_briefing_ui(
    commands: &mut Commands,
    mission_config: &crate::campaign::MissionConfig,
) {
    // Main briefing container
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.9)),
                ..default()
            },
            MissionBriefing,
        ))
        .with_children(|parent| {
            // Mission title
            parent.spawn((
                TextBundle::from_section(
                    format!("🎯 MISSION: {}", mission_config.name.to_uppercase()),
                    TextStyle {
                        font_size: 48.0,
                        color: Color::rgb(1.0, 0.8, 0.0),
                        ..default()
                    },
                ),
                MissionTitle,
            ));

            // Spacer
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(30.0),
                    ..default()
                },
                ..default()
            });

            // Mission description
            parent.spawn((
                TextBundle::from_section(
                    mission_config.description,
                    TextStyle {
                        font_size: 24.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    max_width: Val::Px(800.0),
                    ..default()
                }),
                MissionDescription,
            ));

            // Spacer
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(40.0),
                    ..default()
                },
                ..default()
            });

            // Objectives section
            parent.spawn((
                TextBundle::from_section(
                    "📋 OBJECTIVES:",
                    TextStyle {
                        font_size: 28.0,
                        color: Color::rgb(0.3, 0.8, 1.0),
                        ..default()
                    },
                ),
                MissionObjectives,
            ));

            // List objectives
            for (i, objective) in mission_config.objectives.iter().enumerate() {
                let objective_text = match objective {
                    crate::campaign::MissionObjective::SurviveTime(time) => {
                        format!("{}. Survive for {:.0} seconds", i + 1, time)
                    }
                    crate::campaign::MissionObjective::DefendTarget(target) => {
                        format!("{}. Protect {}", i + 1, target)
                    }
                    crate::campaign::MissionObjective::EliminateEnemies(count) => {
                        format!("{}. Eliminate {} enemy units", i + 1, count)
                    }
                    crate::campaign::MissionObjective::ControlArea(area) => {
                        format!("{}. Control {}", i + 1, area)
                    }
                };

                parent.spawn(
                    TextBundle::from_section(
                        objective_text,
                        TextStyle {
                            font_size: 20.0,
                            color: Color::rgb(0.8, 0.8, 0.8),
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    }),
                );
            }

            // Time limit info
            if let Some(time_limit) = mission_config.time_limit {
                parent.spawn(NodeBundle {
                    style: Style {
                        height: Val::Px(20.0),
                        ..default()
                    },
                    ..default()
                });

                parent.spawn(TextBundle::from_section(
                    format!("⏰ Time Limit: {:.0} seconds", time_limit),
                    TextStyle {
                        font_size: 18.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                        ..default()
                    },
                ));
            }

            // Instructions
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(60.0),
                    ..default()
                },
                ..default()
            });

            parent.spawn(TextBundle::from_section(
                "Press SPACE or ENTER to begin mission",
                TextStyle {
                    font_size: 22.0,
                    color: Color::rgb(0.0, 1.0, 0.0),
                    ..default()
                },
            ));
        });
}

fn create_main_menu_ui(commands: &mut Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.95)),
                ..default()
            },
            SaveLoadMenu,
        ))
        .with_children(|parent| {
            // Game title
            parent.spawn(
                TextBundle::from_section(
                    "🏛️ BATTLE OF CULIACÁN 🏛️\nEl Culiacanazo RTS",
                    TextStyle {
                        font_size: 56.0,
                        color: Color::rgb(1.0, 0.8, 0.0),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                }),
            );

            // Menu options
            parent.spawn(
                TextBundle::from_section(
                    "1. New Campaign",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );

            let load_color = if has_save_file() {
                Color::WHITE
            } else {
                Color::rgb(0.5, 0.5, 0.5)
            };
            parent.spawn(
                TextBundle::from_section(
                    if has_save_file() {
                        "2. Load Campaign"
                    } else {
                        "2. Load Campaign (No Save Found)"
                    },
                    TextStyle {
                        font_size: 32.0,
                        color: load_color,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );

            parent.spawn(
                TextBundle::from_section(
                    "3. Save Current Game",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }),
            );

            // Instructions
            parent.spawn(
                TextBundle::from_section(
                    "Press 1-3 to select option",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::rgb(0.7, 0.7, 0.7),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                }),
            );
        });
}

fn create_save_menu_ui(commands: &mut Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.9)),
                ..default()
            },
            SaveLoadMenu,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "💾 SAVE GAME",
                    TextStyle {
                        font_size: 48.0,
                        color: Color::rgb(0.3, 0.8, 1.0),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                }),
            );

            parent.spawn(
                TextBundle::from_section(
                    "1. Save Slot 1",
                    TextStyle {
                        font_size: 28.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                }),
            );

            parent.spawn(
                TextBundle::from_section(
                    "Press 1 to save, ESC to cancel",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::rgb(0.7, 0.7, 0.7),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                }),
            );
        });
}

fn create_load_menu_ui(commands: &mut Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.9)),
                ..default()
            },
            SaveLoadMenu,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "📂 LOAD GAME",
                    TextStyle {
                        font_size: 48.0,
                        color: Color::rgb(0.3, 0.8, 1.0),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                }),
            );

            let load_text = if has_save_file() {
                "1. Load Slot 1 (Available)"
            } else {
                "1. Load Slot 1 (Empty)"
            };

            let load_color = if has_save_file() {
                Color::WHITE
            } else {
                Color::rgb(0.5, 0.5, 0.5)
            };

            parent.spawn(
                TextBundle::from_section(
                    load_text,
                    TextStyle {
                        font_size: 28.0,
                        color: load_color,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                }),
            );

            parent.spawn(
                TextBundle::from_section(
                    "Press 1 to load, ESC to cancel",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::rgb(0.7, 0.7, 0.7),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                }),
            );
        });
}

fn create_victory_screen(commands: &mut Commands, game_state: &GameState, campaign: &Campaign) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.3, 0.0, 0.95)),
            ..default()
        },
        VictoryScreen,
    )).with_children(|parent| {
        // Victory title
        parent.spawn((
            TextBundle::from_section(
                "🏆 ¡VICTORIA! 🏆",
                TextStyle {
                    font_size: 64.0,
                    color: Color::rgb(1.0, 0.8, 0.0),
                    ..default()
                },
            ),
            MissionResultText,
        ));

        // Mission name
        let mission_config = MissionConfig::get_mission_config(&campaign.progress.current_mission);
        parent.spawn(TextBundle::from_section(
            format!("Mission: {} Complete", mission_config.name),
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        // Historical context
        parent.spawn(TextBundle::from_section(
            "Historical Outcome: The Sinaloa Cartel successfully\npressured the Mexican government to release Ovidio Guzmán.\nThis event became known as 'El Culiacanazo' or 'Black Thursday'.",
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::vertical(Val::Px(30.0)),
            max_width: Val::Px(800.0),
            ..default()
        }));

        // Objectives summary
        parent.spawn(TextBundle::from_section(
            "📊 MISSION OBJECTIVES:",
            TextStyle {
                font_size: 24.0,
                color: Color::rgb(0.3, 0.8, 1.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            get_objective_summary(campaign),
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        // Score summary
        parent.spawn(TextBundle::from_section(
            format!("Final Score: {} | Time: {:.1}s",
                game_state.cartel_score,
                game_state.mission_timer
            ),
            TextStyle {
                font_size: 22.0,
                color: Color::rgb(0.0, 1.0, 0.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        }));

        // Continue instructions
        parent.spawn(TextBundle::from_section(
            "Press SPACE to continue | ESC for main menu",
            TextStyle {
                font_size: 18.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(40.0)),
            ..default()
        }));
    });
}

fn create_defeat_screen(commands: &mut Commands, game_state: &GameState, campaign: &Campaign) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.3, 0.0, 0.0, 0.95)),
            ..default()
        },
        DefeatScreen,
    )).with_children(|parent| {
        // Defeat title
        parent.spawn((
            TextBundle::from_section(
                "💀 MISIÓN FALLIDA 💀",
                TextStyle {
                    font_size: 64.0,
                    color: Color::rgb(1.0, 0.3, 0.3),
                    ..default()
                },
            ),
            MissionResultText,
        ));

        // Mission name
        let mission_config = MissionConfig::get_mission_config(&campaign.progress.current_mission);
        parent.spawn(TextBundle::from_section(
            format!("Mission: {} Failed", mission_config.name),
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        // Failure context
        parent.spawn(TextBundle::from_section(
            "The government forces succeeded in their objective.\nHowever, this simulation helps understand the complex\ndynamics that led to the actual historical outcome.",
            TextStyle {
                font_size: 20.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::vertical(Val::Px(30.0)),
            max_width: Val::Px(800.0),
            ..default()
        }));

        // Objectives summary
        parent.spawn(TextBundle::from_section(
            "📊 MISSION OBJECTIVES:",
            TextStyle {
                font_size: 24.0,
                color: Color::rgb(0.3, 0.8, 1.0),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        }));

        parent.spawn(TextBundle::from_section(
            get_objective_summary(campaign),
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        }));

        // Score summary
        parent.spawn(TextBundle::from_section(
            format!("Final Score: {} | Survived: {:.1}s",
                game_state.cartel_score,
                game_state.mission_timer
            ),
            TextStyle {
                font_size: 22.0,
                color: Color::rgb(1.0, 0.5, 0.5),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(30.0)),
            ..default()
        }));

        // Continue instructions
        parent.spawn(TextBundle::from_section(
            "Press SPACE to try again | ESC for main menu",
            TextStyle {
                font_size: 18.0,
                color: Color::rgb(0.7, 0.7, 0.7),
                ..default()
            },
        ).with_style(Style {
            margin: UiRect::top(Val::Px(40.0)),
            ..default()
        }));
    });
}

fn advance_campaign_or_end(game_state: &mut GameState, _campaign: &Campaign) {
    // For now, return to main menu after victory
    // In the future, this could advance to the next mission
    game_state.game_phase = GamePhase::MainMenu;
    play_tactical_sound("radio", "Mission complete. Ready for next operation...");

    // Reset mission timer for potential replay
    game_state.mission_timer = 0.0;
}
