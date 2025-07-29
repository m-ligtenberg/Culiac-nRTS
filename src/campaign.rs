use bevy::prelude::*;
use crate::components::GamePhase;
use crate::resources::GameState;
use crate::save_system::{CampaignProgress, MissionId, DifficultyLevel};

// ==================== CAMPAIGN MANAGEMENT ====================

#[derive(Resource)]
pub struct Campaign {
    pub progress: CampaignProgress,
    pub mission_timer: f32,
    pub objectives_completed: u32,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            progress: CampaignProgress::default(),
            mission_timer: 0.0,
            objectives_completed: 0,
        }
    }
}

// ==================== MISSION DEFINITIONS ====================

pub struct MissionConfig {
    pub id: MissionId,
    pub name: &'static str,
    pub description: &'static str,
    pub time_limit: Option<f32>,
    pub enemy_spawn_rate: f32,
    pub difficulty_modifier: f32,
    pub objectives: Vec<MissionObjective>,
}

pub enum MissionObjective {
    SurviveTime(f32),
    DefendTarget(String),
    EliminateEnemies(u32),
    ControlArea(String),
}

impl MissionConfig {
    pub fn get_mission_config(mission_id: &MissionId) -> MissionConfig {
        match mission_id {
            MissionId::InitialRaid => MissionConfig {
                id: mission_id.clone(),
                name: "Initial Raid",
                description: "Government forces attempt to capture Ovidio. Defend the safehouse at all costs.",
                time_limit: Some(300.0), // 5 minutes
                enemy_spawn_rate: 1.0,
                difficulty_modifier: 1.0,
                objectives: vec![
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                    MissionObjective::SurviveTime(300.0),
                ],
            },
            MissionId::UrbanWarfare => MissionConfig {
                id: mission_id.clone(),
                name: "Urban Warfare",
                description: "Combat spreads through Culiacán's streets. Control key intersections.",
                time_limit: Some(450.0), // 7.5 minutes
                enemy_spawn_rate: 1.2,
                difficulty_modifier: 1.2,
                objectives: vec![
                    MissionObjective::ControlArea("Downtown".to_string()),
                    MissionObjective::EliminateEnemies(20),
                ],
            },
            MissionId::GovernmentResponse => MissionConfig {
                id: mission_id.clone(),
                name: "Government Response",
                description: "Military escalates response. Show them the cost of this operation.",
                time_limit: Some(600.0), // 10 minutes
                enemy_spawn_rate: 1.5,
                difficulty_modifier: 1.4,
                objectives: vec![
                    MissionObjective::SurviveTime(600.0),
                    MissionObjective::EliminateEnemies(35),
                ],
            },
            MissionId::Resolution => MissionConfig {
                id: mission_id.clone(),
                name: "Resolution",
                description: "Final confrontation. Hold the line until government withdrawal.",
                time_limit: None, // No time limit - fight until victory
                enemy_spawn_rate: 2.0,
                difficulty_modifier: 1.6,
                objectives: vec![
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                    MissionObjective::EliminateEnemies(50),
                ],
            },
        }
    }
}

// ==================== CAMPAIGN SYSTEM ====================

pub fn campaign_system(
    mut campaign: ResMut<Campaign>,
    game_state: Res<GameState>,
    time: Res<Time>,
) {
    campaign.mission_timer += time.delta_seconds();
    
    // Map game phases to mission progression
    let current_mission = match game_state.game_phase {
        GamePhase::Preparation | GamePhase::InitialRaid => MissionId::InitialRaid,
        GamePhase::BlockConvoy => MissionId::UrbanWarfare,
        GamePhase::ApplyPressure => MissionId::GovernmentResponse,
        GamePhase::HoldTheLine => MissionId::Resolution,
        GamePhase::GameOver => return, // No mission updates when game is over
    };
    
    campaign.progress.current_mission = current_mission;
    
    // Check for mission completion
    if game_state.game_phase == GamePhase::GameOver && !game_state.ovidio_captured {
        let mission_score = calculate_mission_score(&game_state, campaign.mission_timer);
        let current_mission = campaign.progress.current_mission.clone();
        let timer = campaign.mission_timer;
        campaign.progress.complete_mission(
            current_mission,
            timer,
            mission_score,
        );
        
        info!("✅ Mission completed! Score: {}, Time: {:.1}s", mission_score, campaign.mission_timer);
    }
}

fn calculate_mission_score(game_state: &GameState, completion_time: f32) -> u32 {
    let base_score = game_state.cartel_score;
    let time_bonus = (600.0 - completion_time.min(600.0)) as u32; // Bonus for faster completion
    let survival_bonus = if !game_state.ovidio_captured { 500 } else { 0 };
    
    base_score + time_bonus + survival_bonus
}

// ==================== DIFFICULTY SYSTEM ====================

pub fn difficulty_system(
    campaign: Res<Campaign>,
    _game_state: ResMut<GameState>,
) {
    // Apply difficulty modifiers based on campaign settings
    let difficulty_modifier = match campaign.progress.difficulty_level {
        DifficultyLevel::Recruit => 0.8,
        DifficultyLevel::Veteran => 1.0,
        DifficultyLevel::Elite => 1.3,
    };
    
    // This modifier could affect spawn rates, enemy health, etc.
    // For now, we'll just track it for future use
    let _current_difficulty = difficulty_modifier;
}

// ==================== MISSION BRIEFING ====================

pub fn get_mission_briefing(mission_id: &MissionId) -> String {
    let config = MissionConfig::get_mission_config(mission_id);
    let mut briefing = format!("🎯 Mission: {}\n\n", config.name);
    briefing.push_str(&format!("📝 {}\n\n", config.description));
    
    briefing.push_str("🎯 Objectives:\n");
    for (i, objective) in config.objectives.iter().enumerate() {
        match objective {
            MissionObjective::SurviveTime(time) => {
                briefing.push_str(&format!("{}. Survive for {:.0} seconds\n", i + 1, time));
            },
            MissionObjective::DefendTarget(target) => {
                briefing.push_str(&format!("{}. Protect {}\n", i + 1, target));
            },
            MissionObjective::EliminateEnemies(count) => {
                briefing.push_str(&format!("{}. Eliminate {} enemy units\n", i + 1, count));
            },
            MissionObjective::ControlArea(area) => {
                briefing.push_str(&format!("{}. Control {}\n", i + 1, area));
            },
        }
    }
    
    if let Some(time_limit) = config.time_limit {
        briefing.push_str(&format!("\n⏰ Time Limit: {:.0} seconds", time_limit));
    }
    
    briefing
}