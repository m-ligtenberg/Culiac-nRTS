use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::Utc;
use crate::components::GamePhase;
use crate::resources::{GameState, SaveData};

// ==================== SAVE SYSTEM ====================

pub fn save_game(game_state: &GameState) -> Result<(), Box<dyn std::error::Error>> {
    let save_data = SaveData {
        game_state: game_state.clone(),
        timestamp: Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    };
    
    let save_json = serde_json::to_string_pretty(&save_data)?;
    fs::write("save_game.json", save_json)?;
    
    info!("✅ Game saved successfully!");
    Ok(())
}

pub fn load_game() -> Result<SaveData, Box<dyn std::error::Error>> {
    let save_json = fs::read_to_string("save_game.json")?;
    let save_data: SaveData = serde_json::from_str(&save_json)?;
    
    info!("✅ Game loaded successfully from {}", save_data.timestamp);
    Ok(save_data)
}

pub fn has_save_file() -> bool {
    std::path::Path::new("save_game.json").exists()
}

// ==================== CAMPAIGN PROGRESS ====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignProgress {
    pub current_mission: MissionId,
    pub completed_missions: Vec<MissionId>,
    pub difficulty_level: DifficultyLevel,
    pub total_score: u32,
    pub best_times: std::collections::HashMap<MissionId, f32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionId {
    InitialRaid,
    UrbanWarfare,
    GovernmentResponse,
    Resolution,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Recruit,  // Easy - reduced enemy spawns, longer timers
    Veteran,  // Normal - balanced gameplay
    Elite,    // Hard - increased difficulty, more enemies
}

impl Default for CampaignProgress {
    fn default() -> Self {
        Self {
            current_mission: MissionId::InitialRaid,
            completed_missions: vec![],
            difficulty_level: DifficultyLevel::Veteran,
            total_score: 0,
            best_times: std::collections::HashMap::new(),
        }
    }
}

impl CampaignProgress { 
    pub fn complete_mission(&mut self, mission_id: MissionId, completion_time: f32, score: u32) {
        if !self.completed_missions.contains(&mission_id) {
            self.completed_missions.push(mission_id.clone());
        }
        
        // Update best time if this is better
        if let Some(best_time) = self.best_times.get(&mission_id) {
            if completion_time < *best_time {
                self.best_times.insert(mission_id.clone(), completion_time);
            }
        } else {
            self.best_times.insert(mission_id.clone(), completion_time);
        }
        
        self.total_score += score;
        
        // Advance to next mission
        self.current_mission = match mission_id {
            MissionId::InitialRaid => MissionId::UrbanWarfare,
            MissionId::UrbanWarfare => MissionId::GovernmentResponse,
            MissionId::GovernmentResponse => MissionId::Resolution,
            MissionId::Resolution => MissionId::Resolution, // Final mission
        };
    }
    
    pub fn is_mission_unlocked(&self, mission_id: &MissionId) -> bool {
        match mission_id {
            MissionId::InitialRaid => true,
            MissionId::UrbanWarfare => self.completed_missions.contains(&MissionId::InitialRaid),
            MissionId::GovernmentResponse => self.completed_missions.contains(&MissionId::UrbanWarfare),
            MissionId::Resolution => self.completed_missions.contains(&MissionId::GovernmentResponse),
        }
    }
    
    pub fn get_mission_description(&self, mission_id: &MissionId) -> &'static str {
        match mission_id {
            MissionId::InitialRaid => "The initial attempt to capture Ovidio begins. Defend the safehouse.",
            MissionId::UrbanWarfare => "Urban combat escalates. Control key intersections.",
            MissionId::GovernmentResponse => "Government pressure mounts. Show the cost of this operation.",
            MissionId::Resolution => "Final showdown. Hold the line until government withdrawal.",
        }
    }
}

// ==================== SAVE SYSTEM EVENTS ====================

#[derive(Event)]
pub struct SaveGameEvent;

#[derive(Event)]
pub struct LoadGameEvent;

pub fn handle_save_events(
    mut save_events: EventReader<SaveGameEvent>,
    game_state: Res<GameState>,
) {
    for _ in save_events.read() {
        if let Err(e) = save_game(&game_state) {
            error!("Failed to save game: {}", e);
        }
    }
}

pub fn handle_load_events(
    mut load_events: EventReader<LoadGameEvent>,
    mut game_state: ResMut<GameState>,
) {
    for _ in load_events.read() {
        match load_game() {
            Ok(save_data) => {
                *game_state = save_data.game_state;
                info!("Game state loaded successfully");
            },
            Err(e) => {
                error!("Failed to load game: {}", e);
            }
        }
    }
}

// ==================== AUTO-SAVE SYSTEM ====================

#[derive(Resource)]
pub struct AutoSaveTimer {
    pub timer: Timer,
    pub enabled: bool,
}

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(60.0, TimerMode::Repeating), // Auto-save every minute
            enabled: true,
        }
    }
}

pub fn auto_save_system(
    mut auto_save_timer: ResMut<AutoSaveTimer>,
    game_state: Res<GameState>,
    time: Res<Time>,
) {
    if !auto_save_timer.enabled {
        return;
    }
    
    auto_save_timer.timer.tick(time.delta());
    
    if auto_save_timer.timer.just_finished() {
        // Only auto-save if game is in progress
        if game_state.game_phase != GamePhase::GameOver {
            if let Err(e) = save_game(&game_state) {
                warn!("Auto-save failed: {}", e);
            } else {
                info!("🔄 Auto-save completed");
            }
        }
    }
}