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

// ==================== INTEL SYSTEM RESOURCE ====================

#[derive(Resource)]
pub struct IntelSystem {
    pub global_intel_network: crate::components::IntelNetwork,
    pub radio_frequency: f32,
    pub jamming_active: bool,
    pub jamming_strength: f32,
    pub intercept_chance: f32,       // Base chance to intercept radio messages
    pub informant_reliability: f32,  // Base reliability of informant tips
    pub counter_intel_level: f32,    // Enemy counter-intelligence strength
}

impl Default for IntelSystem {
    fn default() -> Self {
        Self {
            global_intel_network: crate::components::IntelNetwork {
                active_intercepts: Vec::new(),
                informant_reports: Vec::new(),
                reconnaissance_data: Vec::new(),
                counter_intel_alerts: Vec::new(),
            },
            radio_frequency: 27.185, // Historical Sinaloa Cartel frequency
            jamming_active: false,
            jamming_strength: 0.0,
            intercept_chance: 0.3,
            informant_reliability: 0.7,
            counter_intel_level: 0.4,
        }
    }
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
    !matches!(game_state.game_phase, GamePhase::MainMenu | GamePhase::SaveMenu | GamePhase::LoadMenu | GamePhase::Victory | GamePhase::Defeat)
}