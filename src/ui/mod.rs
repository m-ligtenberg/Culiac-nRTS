// UI Module Organization
// This module splits the massive ui_systems.rs file into focused components

pub mod ui_core;        // Core UI updates, health bars, damage indicators, particles
pub mod ui_camera;      // Camera control system  
pub mod ui_selection;   // Unit selection and target indicators
pub mod ui_menus;       // Main menu, mission briefing, victory/defeat screens
pub mod ui_minimap;     // Minimap system
pub mod ui_animations;  // Sprite and movement animations

// Re-export all systems for easy access
pub use ui_core::*;
pub use ui_camera::*; 
pub use ui_selection::*;
pub use ui_menus::*;
pub use ui_minimap::*;
pub use ui_animations::*;