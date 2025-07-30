use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::utils::play_tactical_sound;
use crate::campaign::Campaign;
use crate::save_system::{save_game, load_game, has_save_file};
use crate::campaign::{get_objective_summary, MissionConfig};

// ==================== MISSION BRIEFING SYSTEM ====================

pub fn mission_briefing_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    campaign: Res<Campaign>,
    input: Res<Input<KeyCode>>,
    briefing_query: Query<Entity, With<MissionBriefing>>,
) {
    // Only show briefing when in MissionBriefing phase
    if game_state.game_phase == GamePhase::MissionBriefing {
        // Remove any existing briefing UI
        for entity in briefing_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Get current mission config
        let mission_config = crate::campaign::MissionConfig::get_mission_config(&campaign.progress.current_mission);
        
        // Create mission briefing UI
        create_mission_briefing_ui(&mut commands, &mission_config);
        
        // Check for input to start mission
        if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Return) {
            // Clear briefing UI
            for entity in briefing_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            // Start the actual mission
            game_state.game_phase = GamePhase::Preparation;
            play_tactical_sound("radio", &format!("Mission: {} - Begin operation!", mission_config.name));
        }
    } else {
        // Clean up any lingering briefing UI when not in briefing phase
        for entity in briefing_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// TODO: Extract remaining menu systems from ui_systems.rs:
// - main_menu_system
// - victory_defeat_system  
// - create_mission_briefing_ui helper function
// - Additional menu helper functions

// Placeholder implementations - will be extracted from ui_systems.rs
pub fn main_menu_system() {
    // TODO: Extract from ui_systems.rs
}

pub fn victory_defeat_system() {
    // TODO: Extract from ui_systems.rs  
}

fn create_mission_briefing_ui(commands: &mut Commands, mission_config: &crate::campaign::MissionConfig) {
    // TODO: Extract from ui_systems.rs - this is a large function
}