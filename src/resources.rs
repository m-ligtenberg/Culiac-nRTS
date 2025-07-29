use bevy::prelude::*;
use bevy_kira_audio::prelude::AudioSource as KiraAudioSource;
use serde::{Deserialize, Serialize};
use crate::components::GamePhase;

// ==================== SETUP RESOURCES ====================

#[derive(Resource)]
pub struct GameSetupComplete;

// ==================== ASSET RESOURCES ====================

#[derive(Resource)]
pub struct GameAssets {
    // Individual sprite handles
    pub sicario_sprite: Handle<Image>,
    pub enforcer_sprite: Handle<Image>,
    pub ovidio_sprite: Handle<Image>,
    pub soldier_sprite: Handle<Image>,
    pub special_forces_sprite: Handle<Image>,
    pub vehicle_sprite: Handle<Image>,
    pub roadblock_sprite: Handle<Image>,
    pub safehouse_sprite: Handle<Image>,
    
    // Future expansion assets
    pub _health_bar_bg: Handle<Image>,
    pub _health_bar_fill: Handle<Image>,
    pub _main_font: Handle<Font>,
    pub _gunshot_sound: Handle<KiraAudioSource>,
    pub _explosion_sound: Handle<KiraAudioSource>,
    pub _radio_chatter: Handle<KiraAudioSource>,
}

// ==================== GAME STATE RESOURCES ====================

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub mission_timer: f32,
    pub current_wave: u32,
    pub cartel_score: u32,
    pub military_score: u32,
    pub game_phase: GamePhase,
    pub ovidio_captured: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            mission_timer: 0.0,
            current_wave: 0,
            cartel_score: 0,
            military_score: 0,
            game_phase: GamePhase::MainMenu,
            ovidio_captured: false,
        }
    }
}

// ==================== AI DIRECTOR RESOURCE ====================

#[derive(Resource)]
pub struct AiDirector {
    pub intensity_level: f32,
    pub last_spawn_time: f32,
    pub player_performance: f32,
    pub adaptive_difficulty: bool,
}

impl Default for AiDirector {
    fn default() -> Self {
        Self {
            intensity_level: 1.0,
            last_spawn_time: 0.0,
            player_performance: 0.5, // 0.0 = struggling, 1.0 = dominating
            adaptive_difficulty: true,
        }
    }
}

// ==================== SAVE/LOAD SYSTEM ====================

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub game_state: GameState,
    pub timestamp: String,
    pub version: String,
}

// ==================== CONDITION FUNCTIONS ====================

pub fn not_in_menu_phase(game_state: Res<GameState>) -> bool {
    !matches!(game_state.game_phase, GamePhase::MainMenu | GamePhase::SaveMenu | GamePhase::LoadMenu)
}