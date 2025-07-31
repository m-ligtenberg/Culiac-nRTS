use crate::components::GamePhase;
use crate::components::{Faction, Unit, UnitType};
use crate::resources::GameState;
use crate::save::save_system::{CampaignProgress, DifficultyLevel, MissionId};
use bevy::log::info;
use bevy::prelude::*;

// ==================== CAMPAIGN MANAGEMENT ====================

#[derive(Resource)]
pub struct Campaign {
    pub progress: CampaignProgress,
    pub mission_timer: f32,
    pub objectives_completed: u32,
    pub current_objectives: Vec<ObjectiveStatus>,
    pub political_pressure: PoliticalPressure,
}

// ==================== POLITICAL PRESSURE SYSTEM ====================

#[derive(Clone, Debug)]
pub struct PoliticalPressure {
    pub civilian_impact: f32, // Civilian casualties and displacement (0.0-1.0)
    pub economic_disruption: f32, // Business closures, blocked roads (0.0-1.0)
    pub media_attention: f32, // International coverage pressure (0.0-1.0)
    pub political_families: f32, // Pressure from wealthy/political families (0.0-1.0)
    pub military_morale: f32, // Government forces demoralization (0.0-1.0)
    pub total_pressure: f32,  // Combined pressure score (0.0-1.0)
}

impl Default for PoliticalPressure {
    fn default() -> Self {
        Self {
            civilian_impact: 0.1, // Minor initial impact
            economic_disruption: 0.05,
            media_attention: 0.2, // Event started with media coverage
            political_families: 0.0,
            military_morale: 0.0,
            total_pressure: 0.0,
        }
    }
}

impl PoliticalPressure {
    pub fn update_pressure(&mut self) {
        // Calculate total pressure as weighted average
        self.total_pressure = (self.civilian_impact * 0.25
            + self.economic_disruption * 0.20
            + self.media_attention * 0.15
            + self.political_families * 0.25
            + self.military_morale * 0.15)
            .clamp(0.0, 1.0);
    }

    pub fn add_civilian_impact(&mut self, impact: f32) {
        self.civilian_impact = (self.civilian_impact + impact * 0.1).clamp(0.0, 1.0);
        info!(
            "📰 Civilian casualties reported - Political pressure increasing: {:.1}%",
            self.civilian_impact * 100.0
        );
    }

    pub fn add_economic_disruption(&mut self, disruption: f32) {
        self.economic_disruption = (self.economic_disruption + disruption * 0.15).clamp(0.0, 1.0);
        info!(
            "💼 Economic disruption spreads - Business leaders demand action: {:.1}%",
            self.economic_disruption * 100.0
        );
    }

    pub fn increase_media_attention(&mut self, attention: f32) {
        self.media_attention = (self.media_attention + attention * 0.1).clamp(0.0, 1.0);
        info!(
            "📺 International media coverage intensifies - Global pressure: {:.1}%",
            self.media_attention * 100.0
        );
    }

    pub fn apply_political_family_pressure(&mut self, pressure: f32) {
        self.political_families = (self.political_families + pressure * 0.2).clamp(0.0, 1.0);
        info!(
            "🏛️ Political families demand resolution - Elite pressure: {:.1}%",
            self.political_families * 100.0
        );
    }

    pub fn reduce_military_morale(&mut self, reduction: f32) {
        self.military_morale = (self.military_morale + reduction * 0.12).clamp(0.0, 1.0);
        info!(
            "⚔️ Military casualties mount - Troop morale declining: {:.1}%",
            self.military_morale * 100.0
        );
    }

    pub fn get_pressure_level(&self) -> PressureLevel {
        match self.total_pressure {
            0.0..=0.2 => PressureLevel::Minimal,
            0.2..=0.4 => PressureLevel::Moderate,
            0.4..=0.6 => PressureLevel::Significant,
            0.6..=0.8 => PressureLevel::Critical,
            _ => PressureLevel::Unbearable,
        }
    }

    pub fn get_government_response_modifier(&self) -> f32 {
        // Higher pressure reduces government aggression
        1.0 - (self.total_pressure * 0.4)
    }
}

#[derive(Clone, Debug)]
pub enum PressureLevel {
    Minimal,     // Government operates normally
    Moderate,    // Some political discussions
    Significant, // Cabinet meetings, media pressure
    Critical,    // Presidential involvement, negotiations
    Unbearable,  // Ceasefire orders, withdrawal
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            progress: CampaignProgress::default(),
            mission_timer: 0.0,
            objectives_completed: 0,
            current_objectives: Vec::new(),
            political_pressure: PoliticalPressure::default(),
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
            // Phase 2 Missions
            MissionId::LasFloresiDefense => MissionConfig {
                id: mission_id.clone(),
                name: "Las Flores Defense",
                description: "Establish defensive perimeters in Las Flores neighborhood while protecting civilians.",
                time_limit: Some(240.0), // 4 minutes
                enemy_spawn_rate: 1.1,
                difficulty_modifier: 1.1,
                objectives: vec![
                    MissionObjective::ControlArea("Las Flores".to_string()),
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                ],
            },
            MissionId::TierraBlancaRoadblocks => MissionConfig {
                id: mission_id.clone(),
                name: "Tierra Blanca Roadblocks",
                description: "Deploy coordinated roadblocks to cut off military reinforcement routes.",
                time_limit: Some(360.0), // 6 minutes
                enemy_spawn_rate: 1.15,
                difficulty_modifier: 1.15,
                objectives: vec![
                    MissionObjective::ControlArea("Highway Access".to_string()),
                    MissionObjective::EliminateEnemies(15),
                ],
            },

            // Phase 3 Missions
            MissionId::CentroUrbanFight => MissionConfig {
                id: mission_id.clone(),
                name: "Centro Urban Battle",
                description: "Battle for downtown Culiacán. Control government buildings and key intersections.",
                time_limit: Some(480.0), // 8 minutes
                enemy_spawn_rate: 1.3,
                difficulty_modifier: 1.25,
                objectives: vec![
                    MissionObjective::ControlArea("City Center".to_string()),
                    MissionObjective::EliminateEnemies(25),
                ],
            },
            MissionId::LasQuintasSiege => MissionConfig {
                id: mission_id.clone(),
                name: "Las Quintas Siege",
                description: "Secure wealthy Las Quintas district to apply pressure on political families.",
                time_limit: Some(420.0), // 7 minutes
                enemy_spawn_rate: 1.25,
                difficulty_modifier: 1.3,
                objectives: vec![
                    MissionObjective::ControlArea("Las Quintas".to_string()),
                    MissionObjective::SurviveTime(420.0),
                ],
            },
            MissionId::AirportAssault => MissionConfig {
                id: mission_id.clone(),
                name: "Airport Control",
                description: "Control Bachigualato Airport to secure escape routes and limit air support.",
                time_limit: Some(540.0), // 9 minutes
                enemy_spawn_rate: 1.4,
                difficulty_modifier: 1.35,
                objectives: vec![
                    MissionObjective::ControlArea("Airport".to_string()),
                    MissionObjective::EliminateEnemies(30),
                ],
            },

            // Phase 4 Missions
            MissionId::GovernmentResponse => MissionConfig {
                id: mission_id.clone(),
                name: "Government Counter-Offensive",
                description: "Military escalation reaches peak. Survive overwhelming government response.",
                time_limit: Some(600.0), // 10 minutes
                enemy_spawn_rate: 1.6,
                difficulty_modifier: 1.4,
                objectives: vec![
                    MissionObjective::SurviveTime(600.0),
                    MissionObjective::EliminateEnemies(40),
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                ],
            },
            MissionId::CivilianEvacuation => MissionConfig {
                id: mission_id.clone(),
                name: "Civilian Protection",
                description: "Protect civilian evacuation zones while maintaining humanitarian corridors.",
                time_limit: Some(480.0), // 8 minutes
                enemy_spawn_rate: 1.3,
                difficulty_modifier: 1.45,
                objectives: vec![
                    MissionObjective::ControlArea("Evacuation Zone".to_string()),
                    MissionObjective::DefendTarget("Civilians".to_string()),
                ],
            },
            MissionId::PoliticalNegotiation => MissionConfig {
                id: mission_id.clone(),
                name: "Political Pressure",
                description: "Hold positions while behind-scenes political negotiations proceed.",
                time_limit: Some(720.0), // 12 minutes
                enemy_spawn_rate: 1.2,
                difficulty_modifier: 1.5,
                objectives: vec![
                    MissionObjective::SurviveTime(720.0),
                    MissionObjective::ControlArea("Strategic Points".to_string()),
                ],
            },

            // Phase 5 Missions
            MissionId::CeasefireNegotiation => MissionConfig {
                id: mission_id.clone(),
                name: "Ceasefire Management",
                description: "Presidential ceasefire order arrives. Manage transition while maintaining advantage.",
                time_limit: Some(300.0), // 5 minutes
                enemy_spawn_rate: 0.8,
                difficulty_modifier: 1.2,
                objectives: vec![
                    MissionObjective::SurviveTime(300.0),
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                ],
            },
            MissionId::OrderedWithdrawal => MissionConfig {
                id: mission_id.clone(),
                name: "Ordered Withdrawal",
                description: "Government forces ordered to withdraw. Ensure orderly retreat without casualties.",
                time_limit: Some(240.0), // 4 minutes
                enemy_spawn_rate: 0.6,
                difficulty_modifier: 1.1,
                objectives: vec![
                    MissionObjective::ControlArea("Withdrawal Routes".to_string()),
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                ],
            },
            MissionId::Resolution => MissionConfig {
                id: mission_id.clone(),
                name: "Victory Secured",
                description: "Final mission complete. Ovidio's freedom secured through political pressure victory.",
                time_limit: None, // No time limit - victory achieved
                enemy_spawn_rate: 0.5,
                difficulty_modifier: 1.0,
                objectives: vec![
                    MissionObjective::DefendTarget("Ovidio".to_string()),
                    MissionObjective::SurviveTime(180.0), // 3 minutes to secure victory
                ],
            },
        }
    }
}

// ==================== CAMPAIGN SYSTEM ====================

pub fn campaign_system(
    mut campaign: ResMut<Campaign>,
    game_state: Res<GameState>,
    unit_query: Query<&Unit>,
    time: Res<Time>,
) {
    campaign.mission_timer += time.delta_seconds();

    // Map game phases to mission progression
    let current_mission = match game_state.game_phase {
        GamePhase::MainMenu
        | GamePhase::SaveMenu
        | GamePhase::LoadMenu
        | GamePhase::MissionBriefing => campaign.progress.current_mission.clone(),
        GamePhase::Preparation | GamePhase::InitialRaid => MissionId::InitialRaid,
        GamePhase::BlockConvoy => MissionId::UrbanWarfare,
        GamePhase::ApplyPressure => MissionId::GovernmentResponse,
        GamePhase::HoldTheLine => MissionId::Resolution,
        GamePhase::Victory | GamePhase::Defeat | GamePhase::GameOver => return, // No mission updates when game is over
    };

    campaign.progress.current_mission = current_mission.clone();

    // Update political pressure based on current mission and events
    update_political_pressure(
        &mut campaign.political_pressure,
        &current_mission,
        &game_state,
        &unit_query,
        time.delta_seconds(),
    );

    // Display pressure updates periodically
    static mut PRESSURE_TIMER: f32 = 0.0;
    unsafe {
        PRESSURE_TIMER += time.delta_seconds();
        if PRESSURE_TIMER > 45.0 {
            // Every 45 seconds
            PRESSURE_TIMER = 0.0;
            let pressure_level = campaign.political_pressure.get_pressure_level();
            info!(
                "🏛️ Political Pressure Status: {:?} ({:.1}% total)",
                pressure_level,
                campaign.political_pressure.total_pressure * 100.0
            );

            match pressure_level {
                PressureLevel::Critical => {
                    info!("📞 Presidential advisors urging immediate resolution")
                }
                PressureLevel::Unbearable => {
                    info!("📞 BREAKING: Presidential intervention imminent - ceasefire likely")
                }
                _ => {}
            }
        }
    }

    // Check for mission completion
    if game_state.game_phase == GamePhase::GameOver && !game_state.ovidio_captured {
        let mission_score = calculate_mission_score(&game_state, campaign.mission_timer);
        let current_mission = campaign.progress.current_mission.clone();
        let timer = campaign.mission_timer;
        campaign
            .progress
            .complete_mission(current_mission, timer, mission_score);

        info!(
            "✅ Mission completed! Score: {}, Time: {:.1}s",
            mission_score, campaign.mission_timer
        );
    }
}

fn update_political_pressure(
    pressure: &mut PoliticalPressure,
    mission_id: &MissionId,
    game_state: &GameState,
    unit_query: &Query<&Unit>,
    delta_time: f32,
) {
    // Count casualties for pressure calculation
    let military_dead = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Military && u.health <= 0.0)
        .count();
    let cartel_dead = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Cartel && u.health <= 0.0)
        .count();

    // Mission-specific pressure increases
    match mission_id {
        MissionId::InitialRaid => {
            pressure.increase_media_attention(delta_time * 0.5);
        }
        MissionId::UrbanWarfare => {
            pressure.add_civilian_impact(delta_time * 0.3);
            pressure.add_economic_disruption(delta_time * 0.4);
        }
        MissionId::LasFloresiDefense => {
            pressure.add_civilian_impact(delta_time * 0.6); // Residential area
        }
        MissionId::TierraBlancaRoadblocks => {
            pressure.add_economic_disruption(delta_time * 0.8); // Major disruption
        }
        MissionId::CentroUrbanFight => {
            pressure.add_economic_disruption(delta_time * 0.7);
            pressure.increase_media_attention(delta_time * 0.4);
        }
        MissionId::LasQuintasSiege => {
            pressure.apply_political_family_pressure(delta_time * 1.0); // Wealthy families
        }
        MissionId::AirportAssault => {
            pressure.increase_media_attention(delta_time * 0.6); // International attention
        }
        MissionId::GovernmentResponse => {
            pressure.reduce_military_morale(delta_time * 0.5);
        }
        MissionId::CivilianEvacuation => {
            pressure.add_civilian_impact(delta_time * 0.8); // Humanitarian crisis
        }
        MissionId::PoliticalNegotiation => {
            // Pressure peaks during negotiations
            pressure.apply_political_family_pressure(delta_time * 0.4);
        }
        _ => {}
    }

    // Casualties increase military morale loss
    if military_dead > 0 {
        pressure.reduce_military_morale(military_dead as f32 * 0.1);
    }

    // Update total pressure calculation
    pressure.update_pressure();
}

fn calculate_mission_score(game_state: &GameState, completion_time: f32) -> u32 {
    let base_score = game_state.cartel_score;
    let time_bonus = (600.0 - completion_time.min(600.0)) as u32; // Bonus for faster completion
    let survival_bonus = if !game_state.ovidio_captured { 500 } else { 0 };

    base_score + time_bonus + survival_bonus
}

// ==================== DIFFICULTY SYSTEM ====================

pub fn difficulty_system(campaign: Res<Campaign>, _game_state: ResMut<GameState>) {
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
            }
            MissionObjective::DefendTarget(target) => {
                briefing.push_str(&format!("{}. Protect {}\n", i + 1, target));
            }
            MissionObjective::EliminateEnemies(count) => {
                briefing.push_str(&format!("{}. Eliminate {} enemy units\n", i + 1, count));
            }
            MissionObjective::ControlArea(area) => {
                briefing.push_str(&format!("{}. Control {}\n", i + 1, area));
            }
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
        campaign.current_objectives = mission_config
            .objectives
            .iter()
            .map(|obj| ObjectiveStatus {
                objective: obj.clone(),
                completed: false,
                progress: 0.0,
            })
            .collect();
    }

    // Count units by faction
    let cartel_units = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Cartel && u.health > 0.0)
        .count() as u32;
    let military_units = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Military && u.health > 0.0)
        .count() as u32;
    let dead_military = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Military && u.health <= 0.0)
        .count() as u32;
    let ovidio_alive = unit_query
        .iter()
        .any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);

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
            }
            MissionObjective::DefendTarget(target_name) => {
                // For now, this is just keeping Ovidio alive
                if target_name == "Ovidio" {
                    objective_status.completed = ovidio_alive;
                    objective_status.progress = if ovidio_alive { 1.0 } else { 0.0 };
                }
            }
            MissionObjective::EliminateEnemies(target_count) => {
                objective_status.progress = (dead_military as f32 / *target_count as f32).min(1.0);
                objective_status.completed = dead_military >= *target_count;
            }
            MissionObjective::ControlArea(_area_name) => {
                // Simplified: control area by having more cartel than military units
                let control_ratio = if military_units > 0 {
                    cartel_units as f32 / (cartel_units + military_units) as f32
                } else {
                    1.0
                };
                objective_status.progress = control_ratio;
                objective_status.completed = control_ratio >= 0.7; // 70% control
            }
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
            }
            MissionObjective::DefendTarget(target) => {
                format!(
                    "Protect {} ({})",
                    target,
                    if obj_status.completed {
                        "Safe"
                    } else {
                        "At Risk"
                    }
                )
            }
            MissionObjective::EliminateEnemies(count) => {
                format!(
                    "Eliminate {} enemies ({:.1}%)",
                    count,
                    obj_status.progress * 100.0
                )
            }
            MissionObjective::ControlArea(area) => {
                format!("Control {} ({:.1}%)", area, obj_status.progress * 100.0)
            }
        };

        summary.push_str(&format!("{}. {} {}\n", i + 1, status_icon, progress_text));
    }

    summary
}
