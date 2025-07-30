// ==================== UTILITY MODULES ====================

pub mod formation;
pub mod unit_queries;
pub mod particles;
pub mod ui_builders;
pub mod combat;
pub mod abilities;
pub mod spatial;
pub mod particle_pool;
pub mod ai_optimizer;

// Re-export commonly used functions
pub use formation::*;
pub use unit_queries::*;
pub use particles::*;
pub use ui_builders::*;
pub use combat::*;
pub use abilities::*;
pub use spatial::*;
pub use particle_pool::*;
pub use ai_optimizer::*;

// ==================== SHARED UTILITY FUNCTIONS ====================

use bevy::prelude::*;

pub fn play_tactical_sound(sound_type: &str, message: &str) {
    // Console-based tactical audio system for atmospheric feedback
    // This is now a fallback system for when the enhanced audio system is not available
    match sound_type {
        "radio" => println!("📻 [RADIO] {}", message),
        "gunfire" => println!("🔫 [GUNFIRE] {}", message),
        "explosion" => println!("💥 [EXPLOSION] {}", message),
        "vehicle" => println!("🚗 [VEHICLE] {}", message),
        "ability" => println!("⚡ [ABILITY] {}", message),
        _ => println!("🔊 [AUDIO] {}", message),
    }
}

pub fn play_tactical_sound_at_position(sound_type: &str, message: &str, _position: Vec3) {
    // Enhanced version that includes position information
    // For now, fallback to console output
    match sound_type {
        "radio" => println!("📻 [RADIO] {} (at {:.1}, {:.1})", message, _position.x, _position.y),
        "gunfire" => println!("🔫 [GUNFIRE] {} (at {:.1}, {:.1})", message, _position.x, _position.y),
        "explosion" => println!("💥 [EXPLOSION] {} (at {:.1}, {:.1})", message, _position.x, _position.y),
        "vehicle" => println!("🚗 [VEHICLE] {} (at {:.1}, {:.1})", message, _position.x, _position.y),
        "ability" => println!("⚡ [ABILITY] {} (at {:.1}, {:.1})", message, _position.x, _position.y),
        _ => println!("🔊 [AUDIO] {} (at {:.1}, {:.1})", message, _position.x, _position.y),
    }
}

pub fn world_to_iso(world_pos: Vec3) -> Vec3 {
    // Convert world coordinates to isometric perspective
    let iso_x = (world_pos.x - world_pos.y) * 0.866; // cos(30°)
    let iso_y = (world_pos.x + world_pos.y) * 0.5;   // sin(30°)
    Vec3::new(iso_x, iso_y, world_pos.z)
}