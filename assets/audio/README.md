# Battle of Culiacán - Audio System

## Overview

The enhanced audio system replaces the console-based audio with a comprehensive spatial audio engine featuring:

- **Spatial 3D Audio**: Combat sounds positioned in 3D space relative to the camera
- **Background Music**: Dynamic music tracks that change based on game phase
- **Radio Chatter**: Queued radio message system with priority handling
- **Volume Controls**: Master, SFX, Music, and Radio volume controls
- **Fallback System**: Console output when audio files are missing

## Directory Structure

```
assets/audio/
├── combat/           # Combat sound effects
├── ui/              # User interface sounds
├── ambient/         # Background environmental sounds
├── radio/           # Radio chatter and communication sounds
└── music/           # Background music tracks
```

## Required Audio Files

### Combat Sounds (`combat/`)
- `gunfire_pistol.ogg` - Pistol gunshot
- `gunfire_rifle.ogg` - Rifle gunshot
- `gunfire_machinegun.ogg` - Machine gun burst
- `explosion_small.ogg` - Grenade/small explosion
- `explosion_large.ogg` - Large explosion/bombardment
- `vehicle_engine.ogg` - Vehicle engine sounds
- `helicopter.ogg` - Helicopter rotors
- `reload.ogg` - Weapon reload sound
- `ricochet.ogg` - Bullet ricochet

### UI Sounds (`ui/`)
- `button_click.ogg` - Button click
- `button_hover.ogg` - Button hover
- `menu_open.ogg` - Menu opening
- `menu_close.ogg` - Menu closing
- `notification.ogg` - General notification
- `warning.ogg` - Warning alert
- `victory.ogg` - Victory fanfare
- `defeat.ogg` - Defeat sound

### Ambient Sounds (`ambient/`)
- `city_ambience.ogg` - Urban background noise
- `wind.ogg` - Wind effects
- `distant_sirens.ogg` - Emergency sirens
- `crowd_panic.ogg` - Crowd panic sounds

### Radio Sounds (`radio/`)
- `radio_static.ogg` - Radio static
- `radio_beep.ogg` - Radio beep
- `radio_voice_cartel.ogg` - Cartel radio voice
- `radio_voice_military.ogg` - Military radio voice

### Background Music (`music/`)
- `menu_theme.ogg` - Main menu music
- `battle_theme.ogg` - Combat music
- `tension_theme.ogg` - Tension/briefing music
- `victory_theme.ogg` - Victory music
- `defeat_theme.ogg` - Defeat music

## Audio System Features

### Spatial Audio
- Combat sounds are positioned in 3D space
- Volume attenuates based on distance from camera
- Supports configurable audio range per sound source

### Background Music System
- Automatically switches tracks based on game phase
- Smooth transitions between tracks
- Looped playback for atmospheric music

### Radio Chatter System
- Priority-based message queue
- Radio static effects before messages
- Configurable message duration
- Console fallback for text display

### Volume Controls
- Master volume (affects everything)
- SFX volume (combat, UI, ambient sounds)
- Music volume (background tracks)
- Radio volume (radio chatter)

## Integration

The audio system is integrated into the main game loop with these systems:
- `setup_audio_system()` - Loads all audio resources
- `background_music_system()` - Manages music playback
- `radio_chatter_system()` - Handles radio message queue
- `spatial_audio_system()` - Updates 3D audio positioning

## Usage Examples

```rust
// Play spatial combat sound
play_spatial_sound(
    &mut commands, 
    &audio_manager, 
    &audio, 
    "combat", 
    "gunfire_rifle", 
    position, 
    0.8
);

// Play UI sound
play_ui_sound(&audio_manager, &audio, "button_click");

// Queue radio message
queue_radio_message(
    &mut radio_player_query, 
    "Enemy units detected in sector 7", 
    "radio", 
    8  // High priority
);
```

## Console Fallback

When audio files are missing, the system gracefully falls back to console output:
```
🔫 [COMBAT] Playing: gunfire_rifle at 150.0, 75.0
📻 [RADIO] Queued: Enemy units detected in sector 7 (Priority: 8)
🎵 [MUSIC] Now playing: battle_theme
```

## Performance

- Audio files are loaded once at startup
- Spatial audio calculations are optimized for real-time performance
- Memory-efficient audio resource management
- Automatic cleanup of expired audio sources