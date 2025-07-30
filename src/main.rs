// ==================== BATTLE OF CULIACÁN RTS GAME ====================
// Historical RTS simulation based on the events of October 17, 2019
// Built with Rust and Bevy Engine
// 
// This game simulates the urban warfare that unfolded during the failed 
// attempt to capture Ovidio Guzmán López in Culiacán, Mexico.
// =====================================================================

use bevy::prelude::*;
use bevy_kira_audio::prelude::{AudioPlugin as KiraAudioPlugin};

// Import our modular components
mod components;
mod resources;
mod systems;
mod ui;
mod game_systems;
mod ai;
mod campaign;
mod save_system;
mod utils;
mod spawners;
mod coordination;
mod unit_systems;

use resources::{*, not_in_menu_phase};
use systems::*;
use ui::*;
use game_systems::*;
use ai::{ai_director_system, unit_ai_system, difficulty_settings_system};
use campaign::{campaign_system, Campaign};
use coordination::{squad_management_system, formation_movement_system, communication_system, advanced_tactical_ai_system};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Battle of Culiacán - El Culiacanazo RTS".into(),
                resolution: (1400.0, 900.0).into(),
                resizable: true,
                present_mode: bevy::window::PresentMode::AutoVsync,
                mode: bevy::window::WindowMode::Windowed,
                visible: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(KiraAudioPlugin)
        .init_resource::<GameState>()
        .init_resource::<AiDirector>()
        .init_resource::<Campaign>()
        .add_systems(Startup, (setup_assets, setup_ui))
        .add_systems(Update, setup_game.run_if(resource_exists::<GameAssets>()).run_if(not(resource_exists::<GameSetupComplete>())).run_if(not_in_menu_phase))
        .add_systems(Update, main_menu_system)
        .add_systems(Update, mission_briefing_system)
        .add_systems(Update, victory_defeat_system)
        .add_systems(Update, (
            camera_control_system,
            unit_selection_system,
            selection_indicator_system,
            target_indicator_system,
            minimap_system,
            mission_system,
            campaign_system,
            ai_director_system,
            wave_spawner_system,
            unit_ai_system,
            squad_management_system,
            formation_movement_system,
            communication_system,
            advanced_tactical_ai_system,
            pathfinding_system,
            movement_system,
            difficulty_settings_system,
        ).run_if(resource_exists::<GameSetupComplete>()))
        .add_systems(Update, (
            combat_system,
            ability_system,
            ability_effect_system,
            health_bar_system,
            particle_system,
            damage_indicator_system,
            sprite_animation_system,
            movement_animation_system,
            ui_update_system,
            game_phase_system,
            handle_input,
        ).run_if(resource_exists::<GameSetupComplete>()))
        .run();
}