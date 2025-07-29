use bevy::prelude::*;
use crate::components::GamePhase;
use crate::resources::GameState;
use crate::save_system::{CampaignProgress, MissionId, DifficultyLevel};
use crate::components::{Unit, Faction, UnitType};

// ==================== CAMPAIGN MANAGEMENT ====================

#[derive(Resource)]
pub struct Campaign {
    pub progress: CampaignProgress,
    pub mission_timer: f32,
    pub objectives_completed: u32,
    pub current_objectives: Vec<ObjectiveStatus>,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            progress: CampaignProgress::default(),
            mission_timer: 0.0,
            objectives_completed: 0,
            current_objectives: Vec::new(),
        }
    }
}

// ==================== OBJECTIVE TRACKING ====================

#[derive(Clone, Debug)]
pub struct ObjectiveStatus {
    pub objective: MissionObjective,
    pub completed: bool,
    pub progress: f32, // 0.0 to 1.0
}

#[derive(Clone, Debug)]
pub enum MissionResult {
    Victory(VictoryType),
    Defeat(DefeatType),
    InProgress,
}

#[derive(Clone, Debug)]
pub enum VictoryType {
    AllObjectivesComplete,
    TimeLimit,
    EnemiesEliminated,
    TargetSurvived,
}

#[derive(Clone, Debug)]
pub enum DefeatType {
    TargetLost,
    TimeExpired,
    AllUnitsDead,
    ObjectiveFailed,
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

#[derive(Clone, Debug)]
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
        GamePhase::MainMenu | GamePhase::SaveMenu | GamePhase::LoadMenu | GamePhase::MissionBriefing => campaign.progress.current_mission.clone(),
        GamePhase::Preparation | GamePhase::InitialRaid => MissionId::InitialRaid,
        GamePhase::BlockConvoy => MissionId::UrbanWarfare,
        GamePhase::ApplyPressure => MissionId::GovernmentResponse,
        GamePhase::HoldTheLine => MissionId::Resolution,
        GamePhase::Victory | GamePhase::Defeat | GamePhase::GameOver => return, // No mission updates when game is over
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

// ==================== OBJECTIVE EVALUATION SYSTEM ====================

pub fn evaluate_mission_objectives(
    campaign: &mut Campaign,
    game_state: &GameState,
    unit_query: &Query<&Unit>,
) -> MissionResult {
    let mission_config = MissionConfig::get_mission_config(&campaign.progress.current_mission);
    
    // Initialize objectives if empty
    if campaign.current_objectives.is_empty() {
        campaign.current_objectives = mission_config.objectives.iter()
            .map(|obj| ObjectiveStatus {
                objective: obj.clone(),
                completed: false,
                progress: 0.0,
            })
            .collect();
    }
    
    // Count units by faction
    let cartel_units = unit_query.iter().filter(|u| u.faction == Faction::Cartel && u.health > 0.0).count() as u32;
    let military_units = unit_query.iter().filter(|u| u.faction == Faction::Military && u.health > 0.0).count() as u32;
    let dead_military = unit_query.iter().filter(|u| u.faction == Faction::Military && u.health <= 0.0).count() as u32;
    let ovidio_alive = unit_query.iter().any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);
    
    // Check for immediate defeat conditions
    if !ovidio_alive {
        return MissionResult::Defeat(DefeatType::TargetLost);
    }
    
    if cartel_units == 0 {
        return MissionResult::Defeat(DefeatType::AllUnitsDead);
    }
    
    // Check time limit expiration
    if let Some(time_limit) = mission_config.time_limit {
        if game_state.mission_timer >= time_limit {
            // For timed missions, surviving the time limit is victory
            return MissionResult::Victory(VictoryType::TimeLimit);
        }
    }
    
    // Update objective progress
    let mut all_completed = true;
    
    for objective_status in &mut campaign.current_objectives {
        match &objective_status.objective {
            MissionObjective::SurviveTime(target_time) => {
                objective_status.progress = (game_state.mission_timer / target_time).min(1.0);
                objective_status.completed = objective_status.progress >= 1.0;
            },
            MissionObjective::DefendTarget(target_name) => {
                // For now, this is just keeping Ovidio alive
                if target_name == "Ovidio" {
                    objective_status.completed = ovidio_alive;
                    objective_status.progress = if ovidio_alive { 1.0 } else { 0.0 };
                }
            },
            MissionObjective::EliminateEnemies(target_count) => {
                objective_status.progress = (dead_military as f32 / *target_count as f32).min(1.0);
                objective_status.completed = dead_military >= *target_count;
            },
            MissionObjective::ControlArea(_area_name) => {
                // Simplified: control area by having more cartel than military units
                let control_ratio = if military_units > 0 {
                    cartel_units as f32 / (cartel_units + military_units) as f32
                } else {
                    1.0
                };
                objective_status.progress = control_ratio;
                objective_status.completed = control_ratio >= 0.7; // 70% control
            },
        }
        
        if !objective_status.completed {
            all_completed = false;
        }
    }
    
    // Check for victory conditions
    if all_completed {
        return MissionResult::Victory(VictoryType::AllObjectivesComplete);
    }
    
    // Special victory condition: eliminate all enemies
    if military_units == 0 && cartel_units > 0 {
        return MissionResult::Victory(VictoryType::EnemiesEliminated);
    }
    
    MissionResult::InProgress
}

pub fn get_objective_summary(campaign: &Campaign) -> String {
    let mut summary = String::new();
    
    for (i, obj_status) in campaign.current_objectives.iter().enumerate() {
        let status_icon = if obj_status.completed { "✅" } else { "🔄" };
        let progress_text = match &obj_status.objective {
            MissionObjective::SurviveTime(time) => {
                format!("Survive {:.0}s ({:.1}%)", time, obj_status.progress * 100.0)
            },
            MissionObjective::DefendTarget(target) => {
                format!("Protect {} ({})", target, if obj_status.completed { "Safe" } else { "At Risk" })
            },
            MissionObjective::EliminateEnemies(count) => {
                format!("Eliminate {} enemies ({:.1}%)", count, obj_status.progress * 100.0)
            },
            MissionObjective::ControlArea(area) => {
                format!("Control {} ({:.1}%)", area, obj_status.progress * 100.0)
            },
        };
        
        summary.push_str(&format!("{}. {} {}\n", i + 1, status_icon, progress_text));
    }
    
    summary
}