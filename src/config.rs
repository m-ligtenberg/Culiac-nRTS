use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ==================== CONFIGURATION SYSTEM ====================

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub gameplay: GameplayConfig,
    pub audio: AudioConfig,
    pub video: VideoConfig,
    pub controls: ControlsConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameplayConfig {
    pub difficulty_level: DifficultyLevel,
    pub auto_save_enabled: bool,
    pub auto_save_interval: f32, // seconds
    pub show_unit_health_bars: bool,
    pub show_damage_numbers: bool,
    pub unit_selection_multi: bool,
    pub camera_edge_scrolling: bool,
    pub pause_on_focus_loss: bool,
    pub historical_accuracy_mode: bool, // Stricter mission objectives
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32, // 0.0 - 1.0
    pub sfx_volume: f32,    // 0.0 - 1.0
    pub music_volume: f32,  // 0.0 - 1.0
    pub voice_volume: f32,  // 0.0 - 1.0
    pub spatial_audio: bool,
    pub console_audio_fallback: bool, // Use console output when audio fails
    pub radio_chatter_frequency: f32, // Frequency of radio messages
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoConfig {
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub ui_scale: f32,         // UI scaling factor
    pub particle_density: f32, // Particle effect density (0.1 - 2.0)
    pub camera_smoothing: f32, // Camera movement smoothing
    pub show_fps: bool,
    pub weather_effects: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlsConfig {
    pub mouse_sensitivity: f32,
    pub camera_pan_speed: f32,
    pub camera_zoom_speed: f32,
    pub double_click_time: f32,  // seconds
    pub edge_scroll_margin: f32, // pixels from edge
    pub invert_camera_y: bool,
    // Key bindings could be added here
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub ai_update_frequency: f32, // How often AI updates (Hz)
    pub pathfinding_quality: PathfindingQuality,
    pub physics_timestep: f32,      // Physics simulation step
    pub max_units_per_faction: u32, // Performance limit
    pub debug_mode: bool,
    pub show_performance_stats: bool,
    pub log_level: LogLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Recruit,    // Easy - more forgiving timers, weaker enemies
    Veteran,    // Normal - balanced gameplay
    Elite,      // Hard - tougher enemies, strict objectives
    Historical, // Maximum realism - based on actual event constraints
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PathfindingQuality {
    Fast,     // Basic pathfinding for performance
    Balanced, // Good balance of quality and performance
    Accurate, // Best pathfinding, may impact performance
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gameplay: GameplayConfig::default(),
            audio: AudioConfig::default(),
            video: VideoConfig::default(),
            controls: ControlsConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            difficulty_level: DifficultyLevel::Veteran,
            auto_save_enabled: true,
            auto_save_interval: 60.0,
            show_unit_health_bars: true,
            show_damage_numbers: true,
            unit_selection_multi: true,
            camera_edge_scrolling: true,
            pause_on_focus_loss: true,
            historical_accuracy_mode: false,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            sfx_volume: 0.7,
            music_volume: 0.6,
            voice_volume: 0.8,
            spatial_audio: true,
            console_audio_fallback: true,
            radio_chatter_frequency: 1.0,
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            resolution_width: 1400,
            resolution_height: 900,
            fullscreen: false,
            vsync: true,
            ui_scale: 1.0,
            particle_density: 1.0,
            camera_smoothing: 0.1,
            show_fps: false,
            weather_effects: true,
        }
    }
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            camera_pan_speed: 500.0,
            camera_zoom_speed: 2.0,
            double_click_time: 0.3,
            edge_scroll_margin: 20.0,
            invert_camera_y: false,
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            ai_update_frequency: 30.0,
            pathfinding_quality: PathfindingQuality::Balanced,
            physics_timestep: 1.0 / 60.0,
            max_units_per_faction: 50,
            debug_mode: false,
            show_performance_stats: false,
            log_level: LogLevel::Info,
        }
    }
}

// ==================== CONFIGURATION MANAGEMENT ====================

const CONFIG_FILE: &str = "config.json";
const CONFIG_DIR: &str = ".culiacan-rts";

impl GameConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = get_config_path();

        if !config_path.exists() {
            // Create default config if none exists
            let default_config = Self::default();
            default_config.save()?;
            info!("📁 Created default configuration file at: {:?}", config_path);
            return Ok(default_config);
        }

        let config_content = fs::read_to_string(&config_path)?;
        let config: GameConfig = serde_json::from_str(&config_content)?;

        info!("⚙️ Loaded configuration from: {:?}", config_path);
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = get_config_path();

        // Create config directory if it doesn't exist
        if let Some(parent_dir) = config_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        let config_json = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_json)?;

        info!("💾 Configuration saved to: {:?}", config_path);
        Ok(())
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
        println!("🔄 Configuration reset to defaults");
    }

    pub fn validate(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Validate audio levels
        if self.audio.master_volume > 1.0 {
            self.audio.master_volume = 1.0;
            warnings.push("Master volume clamped to 100%".to_string());
        }

        // Validate video settings
        if self.video.resolution_width < 800 || self.video.resolution_height < 600 {
            self.video.resolution_width = 1400;
            self.video.resolution_height = 900;
            warnings.push("Resolution reset to 1400x900 (minimum 800x600)".to_string());
        }

        if self.video.particle_density > 3.0 {
            self.video.particle_density = 3.0;
            warnings.push("Particle density clamped to 300%".to_string());
        }

        // Validate advanced settings
        if self.advanced.max_units_per_faction > 200 {
            self.advanced.max_units_per_faction = 200;
            warnings.push("Max units per faction clamped to 200 for performance".to_string());
        }

        warnings
    }

    pub fn apply_difficulty_modifiers(&self) -> DifficultyModifiers {
        match self.gameplay.difficulty_level {
            DifficultyLevel::Recruit => DifficultyModifiers {
                enemy_health_multiplier: 0.7,
                enemy_damage_multiplier: 0.8,
                time_limit_multiplier: 1.3,
                spawn_rate_multiplier: 0.8,
                objective_tolerance: 0.9, // More forgiving objectives
            },
            DifficultyLevel::Veteran => DifficultyModifiers {
                enemy_health_multiplier: 1.0,
                enemy_damage_multiplier: 1.0,
                time_limit_multiplier: 1.0,
                spawn_rate_multiplier: 1.0,
                objective_tolerance: 1.0,
            },
            DifficultyLevel::Elite => DifficultyModifiers {
                enemy_health_multiplier: 1.3,
                enemy_damage_multiplier: 1.2,
                time_limit_multiplier: 0.8,
                spawn_rate_multiplier: 1.4,
                objective_tolerance: 1.1, // Stricter objectives
            },
            DifficultyLevel::Historical => DifficultyModifiers {
                enemy_health_multiplier: 1.5,
                enemy_damage_multiplier: 1.3,
                time_limit_multiplier: 0.7, // Historical time constraints
                spawn_rate_multiplier: 1.6,
                objective_tolerance: 1.2, // Must meet exact historical objectives
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct DifficultyModifiers {
    pub enemy_health_multiplier: f32,
    pub enemy_damage_multiplier: f32,
    pub time_limit_multiplier: f32,
    pub spawn_rate_multiplier: f32,
    pub objective_tolerance: f32,
}

fn get_config_path() -> std::path::PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(CONFIG_DIR).join(CONFIG_FILE)
    } else {
        // Fallback to current directory
        Path::new(CONFIG_FILE).to_path_buf()
    }
}

// ==================== CONFIGURATION SYSTEM ====================

pub fn setup_config_system(mut commands: Commands) {
    match GameConfig::load() {
        Ok(config) => {
            let mut config = config;
            let validation_warnings = config.validate();

            for warning in validation_warnings {
                warn!("Config validation: {}", warning);
            }

            // Save validated config
            if let Err(e) = config.save() {
                error!("Failed to save validated config: {}", e);
            }

            commands.insert_resource(config);
            info!("✅ Game configuration loaded successfully");
        }
        Err(e) => {
            error!("Failed to load config: {}, using defaults", e);
            commands.insert_resource(GameConfig::default());
        }
    }
}

pub fn config_hotkeys_system(keyboard: Res<Input<KeyCode>>, mut config: ResMut<GameConfig>) {
    // F11 - Toggle fullscreen
    if keyboard.just_pressed(KeyCode::F11) {
        config.video.fullscreen = !config.video.fullscreen;
        info!(
            "🖥️ Fullscreen: {}",
            if config.video.fullscreen { "ON" } else { "OFF" }
        );
    }

    // F3 - Toggle FPS display
    if keyboard.just_pressed(KeyCode::F3) {
        config.video.show_fps = !config.video.show_fps;
        info!(
            "📊 FPS Display: {}",
            if config.video.show_fps { "ON" } else { "OFF" }
        );
    }

    // Ctrl+S - Save config
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::S) {
        if let Err(e) = config.save() {
            error!("Failed to save config with hotkey: {}", e);
        }
    }
}

pub fn performance_monitor_system(
    config: Res<GameConfig>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut display_timer: Local<f32>,
    time: Res<Time>,
) {
    if !config.advanced.show_performance_stats {
        return;
    }

    *display_timer += time.delta_seconds();

    if *display_timer > 5.0 {
        // Every 5 seconds
        *display_timer = 0.0;

        if let Some(fps) = diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_value) = fps.smoothed() {
                println!("📊 Performance: {:.1} FPS", fps_value);
            }
        }

        if let Some(frame_time) =
            diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
        {
            if let Some(frame_time_ms) = frame_time.smoothed() {
                println!("📊 Frame Time: {:.2}ms", frame_time_ms * 1000.0);
            }
        }
    }
}
