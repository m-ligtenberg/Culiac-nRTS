use bevy::prelude::*;

// ==================== UTILITY FUNCTIONS ====================

// Enhanced procedural audio with richer atmospheric descriptions
pub fn play_tactical_sound(sound_type: &str, description: &str) {
    match sound_type {
        "gunfire" => info!("🔊 *RATATATATA* {} 🔊", description),
        "explosion" => info!("💥 *BOOM* {} 💥", description),
        "movement" => info!("👥 *FOOTSTEPS* {} 👥", description),
        "radio" => info!("📻 *RADIO STATIC* {} 📻", description),
        "vehicle" => info!("🚗 *ENGINE SOUNDS* {} 🚗", description),
        "construction" => info!("🔨 *CONSTRUCTION* {} 🔨", description),
        _ => info!("🎵 {} 🎵", description),
    }
}

// Isometric transformation helper function
pub fn world_to_iso(world_pos: Vec3) -> Vec3 {
    let x = (world_pos.x - world_pos.y) * 0.5; // Moderate isometric angle
    let y = (world_pos.x + world_pos.y) * 0.25; // Better perspective depth
    Vec3::new(x, y, world_pos.z)
}