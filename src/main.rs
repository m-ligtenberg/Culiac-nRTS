// ==================== BATTLE OF CULIACÁN RTS GAME ====================
// Historical RTS simulation based on the events of October 17, 2019
// Built with Rust and Bevy Engine
//
// This game simulates the urban warfare that unfolded during the failed
// attempt to capture Ovidio Guzmán López in Culiacán, Mexico.
// =====================================================================

use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioPlugin as KiraAudioPlugin;

// Import our modular components
mod ai;
mod audio_system;
mod auth;
mod campaign;
mod components;
mod config;
mod coordination;
mod environmental_systems;
mod game_systems;
mod intel_system;
mod multiplayer_system;
mod political_system;
mod resources;
mod save_system;
mod spawners;
mod systems;
mod ui;
mod unit_systems;
mod utils;

use ai::{ai_director_system, difficulty_settings_system};
use audio_system::{
    background_music_system, radio_chatter_system, setup_audio_system, spatial_audio_system,
};
use campaign::{campaign_system, Campaign};
use config::{config_hotkeys_system, performance_monitor_system, setup_config_system};
use coordination::{
    advanced_tactical_ai_system, communication_system, formation_movement_system,
    squad_management_system,
};
use environmental_systems::{
    spawn_weather_particles, trigger_weather_change, update_ambient_lighting,
    update_environmental_time, update_weather_particles, EnvironmentalAmbientLight,
    EnvironmentalState,
};
use game_systems::*;
use intel_system::IntelSystemPlugin;
use multiplayer_system::MultiplayerSystemPlugin;
use political_system::PoliticalSystemPlugin;
use resources::{not_in_menu_phase, *};
use systems::*;
use ui::*;
use utils::{
    adaptive_ai_scheduler_system, optimized_unit_ai_system, setup_ai_optimizer,
    setup_particle_pool, update_pooled_particles_system,
};

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
        .add_plugins(IntelSystemPlugin)
        .add_plugins(PoliticalSystemPlugin)
        //.add_plugins(MultiplayerSystemPlugin)  // Temporarily disabled until implemented
        .init_resource::<GameState>()
        .init_resource::<AiDirector>()
        .init_resource::<Campaign>()
        .init_resource::<EnvironmentalState>()
        .init_resource::<EnvironmentalAmbientLight>()
        .add_systems(
            Startup,
            (
                setup_config_system,
                setup_assets,
                setup_ui,
                setup_audio_system,
                setup_particle_pool,
                setup_ai_optimizer,
            ),
        )
        .add_systems(
            Startup,
            setup_game
                .run_if(resource_exists::<GameAssets>())
                .run_if(not(resource_exists::<GameSetupComplete>()))
                .run_if(not_in_menu_phase),
        )
        .add_systems(Update, main_menu_system)
        .add_systems(Update, mission_briefing_system)
        .add_systems(Update, victory_defeat_system)
        .add_systems(
            Update,
            (
                camera_control_system,
                unit_selection_system,
                selection_indicator_system,
                target_indicator_system,
                minimap_system,
                mission_system,
                campaign_system,
                ai_director_system,
                wave_spawner_system,
                optimized_unit_ai_system,
                adaptive_ai_scheduler_system,
                squad_management_system,
                formation_movement_system,
                communication_system,
                advanced_tactical_ai_system,
                pathfinding_system,
                movement_system,
                difficulty_settings_system,
            )
                .run_if(resource_exists::<GameSetupComplete>()),
        )
        .add_systems(
            Update,
            (
                combat_system,
                ability_system,
                ability_effect_system,
                health_bar_system,
                update_pooled_particles_system,
                damage_indicator_system,
                sprite_animation_system,
                movement_animation_system,
            )
                .run_if(resource_exists::<GameSetupComplete>()),
        )
        .add_systems(
            Update,
            (
                ui_update_system,
                game_phase_system,
                handle_input,
                background_music_system,
                radio_chatter_system,
                spatial_audio_system,
            )
                .run_if(resource_exists::<GameSetupComplete>()),
        )
        .add_systems(
            Update,
            (
                update_environmental_time,
                update_ambient_lighting,
                spawn_weather_particles,
                update_weather_particles,
                trigger_weather_change,
                config_hotkeys_system,
                performance_monitor_system,
            )
                .run_if(resource_exists::<GameSetupComplete>()),
        )
        .run();
}
