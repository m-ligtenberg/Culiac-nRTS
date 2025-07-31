use crate::components::GamePhase;
use crate::resources::{GameState, SaveData};
use bevy::prelude::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;

// ==================== ENHANCED SAVE SYSTEM ====================

const SAVE_DIR: &str = ".culiacan-rts/saves";
const MAX_SAVE_SLOTS: usize = 10;

pub fn save_game_to_slot(
    game_state: &GameState,
    campaign: &CampaignProgress,
    slot: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if slot >= MAX_SAVE_SLOTS {
        return Err(format!("Save slot {} exceeds maximum {}", slot, MAX_SAVE_SLOTS).into());
    }

    let save_data = EnhancedSaveData {
        game_state: game_state.clone(),
        campaign_progress: campaign.clone(),
        timestamp: Utc::now().to_rfc3339(),
        version: "2.0.0".to_string(),
        slot_number: slot,
        mission_name: get_mission_display_name(&campaign.current_mission),
        playtime_seconds: game_state.mission_timer as u64,
    };

    let save_path = get_save_path(slot);

    // Create save directory if it doesn't exist
    if let Some(parent_dir) = save_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let save_json = serde_json::to_string_pretty(&save_data)?;
    fs::write(&save_path, save_json)?;

    info!("✅ Game saved to slot {} at: {:?}", slot, save_path);
    Ok(())
}

pub fn load_game_from_slot(slot: usize) -> Result<EnhancedSaveData, Box<dyn std::error::Error>> {
    if slot >= MAX_SAVE_SLOTS {
        return Err(format!("Save slot {} exceeds maximum {}", slot, MAX_SAVE_SLOTS).into());
    }

    let save_path = get_save_path(slot);
    let save_json = fs::read_to_string(&save_path)?;
    let save_data: EnhancedSaveData = serde_json::from_str(&save_json)?;

    info!(
        "✅ Game loaded from slot {} ({})",
        slot, save_data.timestamp
    );
    Ok(save_data)
}

pub fn get_save_slot_info(slot: usize) -> Option<SaveSlotInfo> {
    if slot >= MAX_SAVE_SLOTS {
        return None;
    }

    let save_path = get_save_path(slot);
    if !save_path.exists() {
        return None;
    }

    match load_game_from_slot(slot) {
        Ok(save_data) => Some(SaveSlotInfo {
            slot_number: slot,
            mission_name: save_data.mission_name,
            timestamp: save_data.timestamp,
            playtime_seconds: save_data.playtime_seconds,
            total_score: save_data.campaign_progress.total_score,
            completed_missions: save_data.campaign_progress.completed_missions.len(),
        }),
        Err(_) => None,
    }
}

pub fn list_all_saves() -> Vec<SaveSlotInfo> {
    let mut saves = Vec::new();

    for slot in 0..MAX_SAVE_SLOTS {
        if let Some(save_info) = get_save_slot_info(slot) {
            saves.push(save_info);
        }
    }

    // Sort by most recent first
    saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    saves
}

pub fn delete_save_slot(slot: usize) -> Result<(), Box<dyn std::error::Error>> {
    if slot >= MAX_SAVE_SLOTS {
        return Err(format!("Save slot {} exceeds maximum {}", slot, MAX_SAVE_SLOTS).into());
    }

    let save_path = get_save_path(slot);
    if save_path.exists() {
        fs::remove_file(&save_path)?;
        info!("🗑️ Deleted save slot {}", slot);
    }

    Ok(())
}

fn get_save_path(slot: usize) -> std::path::PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        home_dir
            .join(SAVE_DIR)
            .join(format!("save_slot_{}.json", slot))
    } else {
        // Fallback to current directory
        std::path::Path::new(&format!("save_slot_{}.json", slot)).to_path_buf()
    }
}

fn get_mission_display_name(mission_id: &MissionId) -> String {
    match mission_id {
        MissionId::InitialRaid => "Initial Raid",
        MissionId::UrbanWarfare => "Urban Warfare",
        MissionId::LasFloresiDefense => "Las Flores Defense",
        MissionId::TierraBlancaRoadblocks => "Tierra Blanca Roadblocks",
        MissionId::CentroUrbanFight => "Centro Battle",
        MissionId::LasQuintasSiege => "Las Quintas Siege",
        MissionId::AirportAssault => "Airport Control",
        MissionId::GovernmentResponse => "Government Response",
        MissionId::CivilianEvacuation => "Civilian Protection",
        MissionId::PoliticalNegotiation => "Political Pressure",
        MissionId::CeasefireNegotiation => "Ceasefire Management",
        MissionId::OrderedWithdrawal => "Ordered Withdrawal",
        MissionId::Resolution => "Victory Secured",
    }
    .to_string()
}

// Legacy save system compatibility
pub fn save_game(game_state: &GameState) -> Result<(), Box<dyn std::error::Error>> {
    let campaign = CampaignProgress::default(); // Use default if no campaign available
    save_game_to_slot(game_state, &campaign, 0) // Save to slot 0
}

pub fn load_game() -> Result<SaveData, Box<dyn std::error::Error>> {
    match load_game_from_slot(0) {
        Ok(enhanced_save) => Ok(SaveData {
            game_state: enhanced_save.game_state,
            timestamp: enhanced_save.timestamp,
            version: enhanced_save.version,
        }),
        Err(e) => Err(e),
    }
}

pub fn has_save_file() -> bool {
    get_save_path(0).exists()
}

// ==================== ENHANCED SAVE DATA STRUCTURES ====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnhancedSaveData {
    pub game_state: GameState,
    pub campaign_progress: CampaignProgress,
    pub timestamp: String,
    pub version: String,
    pub slot_number: usize,
    pub mission_name: String,
    pub playtime_seconds: u64,
}

#[derive(Clone, Debug)]
pub struct SaveSlotInfo {
    pub slot_number: usize,
    pub mission_name: String,
    pub timestamp: String,
    pub playtime_seconds: u64,
    pub total_score: u32,
    pub completed_missions: usize,
}

impl SaveSlotInfo {
    pub fn get_display_text(&self) -> String {
        let hours = self.playtime_seconds / 3600;
        let minutes = (self.playtime_seconds % 3600) / 60;

        format!(
            "Slot {}: {} | {}h {}m | Score: {} | Missions: {}",
            self.slot_number + 1,
            self.mission_name,
            hours,
            minutes,
            self.total_score,
            self.completed_missions
        )
    }

    pub fn get_formatted_timestamp(&self) -> String {
        // Parse and format the ISO timestamp for display
        if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&self.timestamp) {
            datetime.format("%Y-%m-%d %H:%M").to_string()
        } else {
            self.timestamp.clone()
        }
    }
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
    // Phase 1: The Attempt (3:15-3:30 PM)
    InitialRaid, // Original arrest attempt at residential complex

    // Phase 2: First Response (3:30-4:30 PM)
    UrbanWarfare,           // Street fighting breaks out
    LasFloresiDefense,      // Defense of Las Flores neighborhood
    TierraBlancaRoadblocks, // Coordinated roadblock deployment

    // Phase 3: City-Wide Escalation (4:30-6:00 PM)
    CentroUrbanFight, // Downtown Culiacán battle
    LasQuintasSiege,  // Wealthy neighborhood control
    AirportAssault,   // Securing escape routes

    // Phase 4: Political Pressure (6:00-7:30 PM)
    GovernmentResponse,   // Military escalation
    CivilianEvacuation,   // Protecting non-combatants
    PoliticalNegotiation, // Behind-scenes pressure

    // Phase 5: Resolution (7:30-8:30 PM)
    CeasefireNegotiation, // Diplomatic resolution
    OrderedWithdrawal,    // Government forces retreat
    Resolution,           // Final mission - securing victory
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Recruit, // Easy - reduced enemy spawns, longer timers
    Veteran, // Normal - balanced gameplay
    Elite,   // Hard - increased difficulty, more enemies
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

        // Advance to next mission following historical timeline
        self.current_mission = match mission_id {
            // Phase 1 -> Phase 2
            MissionId::InitialRaid => MissionId::UrbanWarfare,
            MissionId::UrbanWarfare => MissionId::LasFloresiDefense,
            MissionId::LasFloresiDefense => MissionId::TierraBlancaRoadblocks,

            // Phase 2 -> Phase 3
            MissionId::TierraBlancaRoadblocks => MissionId::CentroUrbanFight,
            MissionId::CentroUrbanFight => MissionId::LasQuintasSiege,
            MissionId::LasQuintasSiege => MissionId::AirportAssault,

            // Phase 3 -> Phase 4
            MissionId::AirportAssault => MissionId::GovernmentResponse,
            MissionId::GovernmentResponse => MissionId::CivilianEvacuation,
            MissionId::CivilianEvacuation => MissionId::PoliticalNegotiation,

            // Phase 4 -> Phase 5
            MissionId::PoliticalNegotiation => MissionId::CeasefireNegotiation,
            MissionId::CeasefireNegotiation => MissionId::OrderedWithdrawal,
            MissionId::OrderedWithdrawal => MissionId::Resolution,
            MissionId::Resolution => MissionId::Resolution, // Final mission
        };
    }

    pub fn is_mission_unlocked(&self, mission_id: &MissionId) -> bool {
        match mission_id {
            // Phase 1 - Always unlocked
            MissionId::InitialRaid => true,

            // Phase 2 - Requires previous missions
            MissionId::UrbanWarfare => self.completed_missions.contains(&MissionId::InitialRaid),
            MissionId::LasFloresiDefense => {
                self.completed_missions.contains(&MissionId::UrbanWarfare)
            }
            MissionId::TierraBlancaRoadblocks => self
                .completed_missions
                .contains(&MissionId::LasFloresiDefense),

            // Phase 3 - Neighborhood battles
            MissionId::CentroUrbanFight => self
                .completed_missions
                .contains(&MissionId::TierraBlancaRoadblocks),
            MissionId::LasQuintasSiege => self
                .completed_missions
                .contains(&MissionId::CentroUrbanFight),
            MissionId::AirportAssault => self
                .completed_missions
                .contains(&MissionId::LasQuintasSiege),

            // Phase 4 - Political pressure builds
            MissionId::GovernmentResponse => {
                self.completed_missions.contains(&MissionId::AirportAssault)
            }
            MissionId::CivilianEvacuation => self
                .completed_missions
                .contains(&MissionId::GovernmentResponse),
            MissionId::PoliticalNegotiation => self
                .completed_missions
                .contains(&MissionId::CivilianEvacuation),

            // Phase 5 - Resolution
            MissionId::CeasefireNegotiation => self
                .completed_missions
                .contains(&MissionId::PoliticalNegotiation),
            MissionId::OrderedWithdrawal => self
                .completed_missions
                .contains(&MissionId::CeasefireNegotiation),
            MissionId::Resolution => self
                .completed_missions
                .contains(&MissionId::OrderedWithdrawal),
        }
    }

    pub fn get_mission_description(&self, mission_id: &MissionId) -> &'static str {
        match mission_id {
            // Phase 1: The Attempt (3:15-3:30 PM)
            MissionId::InitialRaid => "3:15 PM - Government forces storm residential complex. Defend Ovidio during the initial arrest attempt.",

            // Phase 2: First Response (3:30-4:30 PM)
            MissionId::UrbanWarfare => "3:30 PM - Street fighting erupts as cartel responds. Coordinate counter-attack across multiple fronts.",
            MissionId::LasFloresiDefense => "3:45 PM - Defend Las Flores neighborhood. Establish defensive perimeters around civilian areas.",
            MissionId::TierraBlancaRoadblocks => "4:00 PM - Deploy roadblocks across Tierra Blanca. Cut off military reinforcement routes.",

            // Phase 3: City-Wide Escalation (4:30-6:00 PM)
            MissionId::CentroUrbanFight => "4:30 PM - Battle for downtown Culiacán. Control key government buildings and intersections.",
            MissionId::LasQuintasSiege => "5:00 PM - Secure Las Quintas wealthy district. Apply pressure on political families.",
            MissionId::AirportAssault => "5:30 PM - Control Bachigualato Airport. Secure escape routes and limit government air support.",

            // Phase 4: Political Pressure (6:00-7:30 PM)
            MissionId::GovernmentResponse => "6:00 PM - Military escalation reaches peak. Survive overwhelming government counter-offensive.",
            MissionId::CivilianEvacuation => "6:30 PM - Protect civilian evacuation zones. Maintain humanitarian corridors under fire.",
            MissionId::PoliticalNegotiation => "7:00 PM - Behind-scenes political pressure mounts. Hold positions while negotiations proceed.",

            // Phase 5: Resolution (7:30-8:30 PM)
            MissionId::CeasefireNegotiation => "7:30 PM - Presidential order arrives. Manage ceasefire while maintaining tactical advantage.",
            MissionId::OrderedWithdrawal => "8:00 PM - Government forces ordered to withdraw. Ensure orderly retreat without further casualties.",
            MissionId::Resolution => "8:30 PM - Final mission complete. Secure the victory and Ovidio's freedom through political pressure.",
        }
    }
}

// ==================== SAVE SYSTEM EVENTS ====================

#[derive(Event)]
pub struct SaveGameEvent;

#[derive(Event)]
pub struct LoadGameEvent;

pub fn handle_save_events(mut save_events: EventReader<SaveGameEvent>, game_state: Res<GameState>) {
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
            }
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
