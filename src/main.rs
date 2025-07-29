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
mod ui_systems;
mod game_systems;
mod ai;
mod campaign;
mod save_system;
mod utils;
mod spawners;

use resources::*;
use systems::*;
use ui_systems::*;
use game_systems::*;
use ai::{ai_director_system, unit_ai_system};

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
        .add_systems(Startup, (setup_assets, setup_ui))
        .add_systems(Update, setup_game.run_if(resource_exists::<GameAssets>()).run_if(not(resource_exists::<GameSetupComplete>())))
        .add_systems(Update, (
            camera_control_system,
            unit_selection_system,
            selection_indicator_system,
            minimap_system,
            mission_system,
            ai_director_system,
            wave_spawner_system,
            unit_ai_system,
            movement_system,
            combat_system,
            health_bar_system,
            particle_system,
            damage_indicator_system,
            ui_update_system,
            game_phase_system,
            handle_input,
        ).run_if(resource_exists::<GameSetupComplete>()))
        .run();
}