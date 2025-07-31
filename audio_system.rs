use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio, AudioControl};
use bevy_kira_audio::AudioSource as KiraAudioSource;
use std::collections::HashMap;

// ==================== AUDIO SYSTEM COMPONENTS ====================

#[derive(Resource)]
pub struct AudioManager {
    pub combat_sounds: HashMap<String, Handle<KiraAudioSource>>,
    pub ui_sounds: HashMap<String, Handle<KiraAudioSource>>,
    pub ambient_sounds: HashMap<String, Handle<KiraAudioSource>>,
    pub radio_sounds: HashMap<String, Handle<KiraAudioSource>>,
    pub background_music: HashMap<String, Handle<KiraAudioSource>>,
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub radio_volume: f32,
    pub spatial_audio_enabled: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self {
            combat_sounds: HashMap::new(),
            ui_sounds: HashMap::new(),
            ambient_sounds: HashMap::new(),
            radio_sounds: HashMap::new(),
            background_music: HashMap::new(),
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.6,
            radio_volume: 0.9,
            spatial_audio_enabled: true,
        }
    }
}

#[derive(Component)]
pub struct AudioSource3D {
    pub position: Vec3,
    pub range: f32,
    pub volume: f32,
    pub is_playing: bool,
}

#[derive(Component)]
pub struct BackgroundMusicPlayer {
    pub current_track: Option<String>,
    pub fade_timer: Timer,
    pub is_fading: bool,
    pub target_volume: f32,
}

#[derive(Component)]
pub struct RadioChatterPlayer {
    pub message_queue: Vec<RadioMessage>,
    pub current_message: Option<RadioMessage>,
    pub playback_timer: Timer,
}

#[derive(Clone)]
pub struct RadioMessage {
    pub text: String,
    pub sound_type: String,
    pub priority: u8, // 1-10, higher is more urgent
    pub duration: f32,
}

// ==================== AUDIO LOADING SYSTEM ====================

pub fn setup_audio_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("🔊 Setting up enhanced audio system...");

    let mut audio_manager = AudioManager::default();

    // Load combat sounds
    audio_manager.combat_sounds.insert(
        "gunfire_pistol".to_string(),
        asset_server.load("audio/combat/gunfire_pistol.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "gunfire_rifle".to_string(),
        asset_server.load("audio/combat/gunfire_rifle.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "gunfire_machinegun".to_string(),
        asset_server.load("audio/combat/gunfire_machinegun.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "explosion_small".to_string(),
        asset_server.load("audio/combat/explosion_small.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "explosion_large".to_string(),
        asset_server.load("audio/combat/explosion_large.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "vehicle_engine".to_string(),
        asset_server.load("audio/combat/vehicle_engine.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "helicopter".to_string(),
        asset_server.load("audio/combat/helicopter.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "reload".to_string(),
        asset_server.load("audio/combat/reload.ogg"),
    );
    audio_manager.combat_sounds.insert(
        "ricochet".to_string(),
        asset_server.load("audio/combat/ricochet.ogg"),
    );

    // Load UI sounds
    audio_manager.ui_sounds.insert(
        "button_click".to_string(),
        asset_server.load("audio/ui/button_click.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "button_hover".to_string(),
        asset_server.load("audio/ui/button_hover.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "menu_open".to_string(),
        asset_server.load("audio/ui/menu_open.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "menu_close".to_string(),
        asset_server.load("audio/ui/menu_close.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "notification".to_string(),
        asset_server.load("audio/ui/notification.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "warning".to_string(),
        asset_server.load("audio/ui/warning.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "victory".to_string(),
        asset_server.load("audio/ui/victory.ogg"),
    );
    audio_manager.ui_sounds.insert(
        "defeat".to_string(),
        asset_server.load("audio/ui/defeat.ogg"),
    );

    // Load ambient sounds
    audio_manager.ambient_sounds.insert(
        "city_ambience".to_string(),
        asset_server.load("audio/ambient/city_ambience.ogg"),
    );
    audio_manager.ambient_sounds.insert(
        "wind".to_string(),
        asset_server.load("audio/ambient/wind.ogg"),
    );
    audio_manager.ambient_sounds.insert(
        "distant_sirens".to_string(),
        asset_server.load("audio/ambient/distant_sirens.ogg"),
    );
    audio_manager.ambient_sounds.insert(
        "crowd_panic".to_string(),
        asset_server.load("audio/ambient/crowd_panic.ogg"),
    );

    // Load radio chatter
    audio_manager.radio_sounds.insert(
        "radio_static".to_string(),
        asset_server.load("audio/radio/radio_static.ogg"),
    );
    audio_manager.radio_sounds.insert(
        "radio_beep".to_string(),
        asset_server.load("audio/radio/radio_beep.ogg"),
    );
    audio_manager.radio_sounds.insert(
        "radio_voice_cartel".to_string(),
        asset_server.load("audio/radio/radio_voice_cartel.ogg"),
    );
    audio_manager.radio_sounds.insert(
        "radio_voice_military".to_string(),
        asset_server.load("audio/radio/radio_voice_military.ogg"),
    );

    // Load background music
    audio_manager.background_music.insert(
        "menu_theme".to_string(),
        asset_server.load("audio/music/menu_theme.ogg"),
    );
    audio_manager.background_music.insert(
        "battle_theme".to_string(),
        asset_server.load("audio/music/battle_theme.ogg"),
    );
    audio_manager.background_music.insert(
        "tension_theme".to_string(),
        asset_server.load("audio/music/tension_theme.ogg"),
    );
    audio_manager.background_music.insert(
        "victory_theme".to_string(),
        asset_server.load("audio/music/victory_theme.ogg"),
    );
    audio_manager.background_music.insert(
        "defeat_theme".to_string(),
        asset_server.load("audio/music/defeat_theme.ogg"),
    );

    commands.insert_resource(audio_manager);

    // Spawn background music player
    commands.spawn(BackgroundMusicPlayer {
        current_track: None,
        fade_timer: Timer::from_seconds(2.0, TimerMode::Once),
        is_fading: false,
        target_volume: 0.6,
    });

    // Spawn radio chatter player
    commands.spawn(RadioChatterPlayer {
        message_queue: Vec::new(),
        current_message: None,
        playback_timer: Timer::from_seconds(1.0, TimerMode::Once),
    });

    info!("✅ Audio system setup complete!");
}

// ==================== ENHANCED AUDIO FUNCTIONS ====================

pub fn play_spatial_sound(
    commands: &mut Commands,
    audio_manager: &AudioManager,
    audio: &Audio,
    sound_type: &str,
    sound_name: &str,
    position: Vec3,
    volume: f32,
) {
    let sound_handle = match sound_type {
        "combat" => audio_manager.combat_sounds.get(sound_name),
        "ui" => audio_manager.ui_sounds.get(sound_name),
        "ambient" => audio_manager.ambient_sounds.get(sound_name),
        "radio" => audio_manager.radio_sounds.get(sound_name),
        _ => None,
    };

    if let Some(handle) = sound_handle {
        let final_volume = volume * audio_manager.master_volume * audio_manager.sfx_volume;

        if audio_manager.spatial_audio_enabled {
            // Create spatial audio source
            commands.spawn((
                AudioSource3D {
                    position,
                    range: 500.0,
                    volume: final_volume,
                    is_playing: true,
                },
                SpatialBundle::from_transform(Transform::from_translation(position)),
            ));

            audio.play(handle.clone()).with_volume(final_volume as f64);
        } else {
            // Play as regular 2D audio
            audio.play(handle.clone()).with_volume(final_volume as f64);
        }

        // Console fallback for debugging
        let icon = match sound_type {
            "combat" => "🔫",
            "ui" => "🔊",
            "ambient" => "🌆",
            "radio" => "📻",
            _ => "🔊",
        };
        info!(
            "{} [{}] Playing: {} at {:.1}, {:.1}",
            icon,
            sound_type.to_uppercase(),
            sound_name,
            position.x,
            position.y
        );
    } else {
        // Fallback to console audio for missing files
        play_console_fallback(sound_type, sound_name);
    }
}

pub fn play_ui_sound(audio_manager: &AudioManager, audio: &Audio, sound_name: &str) {
    if let Some(handle) = audio_manager.ui_sounds.get(sound_name) {
        let volume = audio_manager.master_volume * audio_manager.sfx_volume;
        audio.play(handle.clone()).with_volume(volume as f64);
        info!("🔊 [UI] Playing: {}", sound_name);
    } else {
        play_console_fallback("ui", sound_name);
    }
}

pub fn queue_radio_message(
    radio_player_query: &mut Query<&mut RadioChatterPlayer>,
    message: &str,
    sound_type: &str,
    priority: u8,
) {
    if let Ok(mut radio_player) = radio_player_query.get_single_mut() {
        let radio_message = RadioMessage {
            text: message.to_string(),
            sound_type: sound_type.to_string(),
            priority,
            duration: message.len() as f32 * 0.1 + 2.0, // Estimate duration
        };

        // Insert based on priority
        let insert_pos = radio_player
            .message_queue
            .iter()
            .position(|msg| msg.priority < priority)
            .unwrap_or(radio_player.message_queue.len());

        radio_player.message_queue.insert(insert_pos, radio_message);
        println!("📻 [RADIO] Queued: {} (Priority: {})", message, priority);
    }
}

fn play_console_fallback(sound_type: &str, sound_name: &str) {
    match sound_type {
        "combat" => info!("🔫 [COMBAT] {}", sound_name),
        "ui" => info!("🔊 [UI] {}", sound_name),
        "ambient" => info!("🌆 [AMBIENT] {}", sound_name),
        "radio" => info!("📻 [RADIO] {}", sound_name),
        _ => info!("🔊 [AUDIO] {}", sound_name),
    }
}

// ==================== AUDIO SYSTEMS ====================

pub fn background_music_system(
    mut music_player_query: Query<&mut BackgroundMusicPlayer>,
    audio_manager: Res<AudioManager>,
    audio: Res<Audio>,
    game_state: Res<GameState>,
    time: Res<Time>,
) {
    if let Ok(mut music_player) = music_player_query.get_single_mut() {
        music_player.fade_timer.tick(time.delta());

        // Determine what music should be playing based on game state
        let desired_track = match game_state.game_phase {
            GamePhase::MainMenu => "menu_theme",
            GamePhase::MissionBriefing => "tension_theme",
            GamePhase::Preparation
            | GamePhase::InitialRaid
            | GamePhase::BlockConvoy
            | GamePhase::ApplyPressure
            | GamePhase::HoldTheLine => "battle_theme",
            GamePhase::Victory => "victory_theme",
            GamePhase::Defeat => "defeat_theme",
            _ => "tension_theme",
        };

        // Change music if needed
        if music_player.current_track.as_deref() != Some(desired_track) {
            if let Some(handle) = audio_manager.background_music.get(desired_track) {
                let volume = audio_manager.master_volume * audio_manager.music_volume;
                audio
                    .play(handle.clone())
                    .with_volume(volume as f64)
                    .looped();

                music_player.current_track = Some(desired_track.to_string());
                println!("🎵 [MUSIC] Now playing: {}", desired_track);
            }
        }
    }
}

pub fn radio_chatter_system(
    mut radio_player_query: Query<&mut RadioChatterPlayer>,
    audio_manager: Res<AudioManager>,
    audio: Res<Audio>,
    time: Res<Time>,
) {
    if let Ok(mut radio_player) = radio_player_query.get_single_mut() {
        radio_player.playback_timer.tick(time.delta());

        // Start next message if current one is finished
        if radio_player.current_message.is_none() && !radio_player.message_queue.is_empty() {
            let message = radio_player.message_queue.remove(0);
            radio_player.playback_timer = Timer::from_seconds(message.duration, TimerMode::Once);

            // Play radio static first
            if let Some(static_handle) = audio_manager.radio_sounds.get("radio_static") {
                let volume = audio_manager.master_volume * audio_manager.radio_volume * 0.3;
                audio.play(static_handle.clone()).with_volume(volume as f64);
            }

            // Display the message
            println!("📻 [RADIO] {}", message.text);
            radio_player.current_message = Some(message);
        }

        // Clear current message when timer finishes
        if radio_player.playback_timer.finished() && radio_player.current_message.is_some() {
            radio_player.current_message = None;
        }
    }
}

pub fn spatial_audio_system(
    mut audio_3d_query: Query<(&mut AudioSource3D, &Transform)>,
    camera_query: Query<&Transform, (With<Camera>, Without<AudioSource3D>)>,
    _time: Res<Time>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let listener_pos = camera_transform.translation;

        for (mut audio_source, source_transform) in audio_3d_query.iter_mut() {
            let distance = listener_pos.distance(source_transform.translation);

            // Calculate volume based on distance
            let distance_factor = 1.0 - (distance / audio_source.range).clamp(0.0, 1.0);
            let _spatial_volume = audio_source.volume * distance_factor;

            // Update audio source properties based on distance
            if distance > audio_source.range {
                audio_source.is_playing = false;
            } else if !audio_source.is_playing && distance <= audio_source.range {
                audio_source.is_playing = true;
            }
        }
    }
}

// ==================== ENHANCED TACTICAL SOUND FUNCTION ====================

pub fn play_enhanced_tactical_sound(
    commands: &mut Commands,
    audio_manager: &Res<AudioManager>,
    audio: &Res<Audio>,
    radio_player_query: &mut Query<&mut RadioChatterPlayer>,
    sound_type: &str,
    message: &str,
    position: Option<Vec3>,
) {
    match sound_type {
        "gunfire" => {
            let sound_name = "gunfire_rifle";
            if let Some(pos) = position {
                play_spatial_sound(
                    commands,
                    audio_manager,
                    audio,
                    "combat",
                    sound_name,
                    pos,
                    0.8,
                );
            } else {
                play_ui_sound(audio_manager, audio, sound_name);
            }
        }
        "explosion" => {
            let sound_name = "explosion_small";
            if let Some(pos) = position {
                play_spatial_sound(
                    commands,
                    audio_manager,
                    audio,
                    "combat",
                    sound_name,
                    pos,
                    1.0,
                );
            } else {
                play_ui_sound(audio_manager, audio, sound_name);
            }
        }
        "vehicle" => {
            let sound_name = "vehicle_engine";
            if let Some(pos) = position {
                play_spatial_sound(
                    commands,
                    audio_manager,
                    audio,
                    "combat",
                    sound_name,
                    pos,
                    0.7,
                );
            } else {
                play_ui_sound(audio_manager, audio, sound_name);
            }
        }
        "ability" => {
            play_ui_sound(audio_manager, audio, "notification");
        }
        "radio" => {
            queue_radio_message(radio_player_query, message, "radio", 5);
        }
        _ => {
            play_console_fallback(sound_type, message);
        }
    }
}
