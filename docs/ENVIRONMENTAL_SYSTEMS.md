# Environmental Systems Documentation

## Overview

The Environmental Systems in Battle of Culiacán RTS provide dynamic weather conditions and time-of-day progression that directly impact tactical gameplay. This system adds strategic depth by creating varying battlefield conditions that affect unit movement, detection ranges, and visual aesthetics.

## Core Components

### EnvironmentalState Resource

The `EnvironmentalState` resource manages all environmental parameters:

```rust
pub struct EnvironmentalState {
    pub time_of_day: f32,           // 0.0 = midnight, 0.5 = noon, 1.0 = midnight
    pub weather_type: WeatherType,   // Current weather condition
    pub weather_intensity: f32,      // 0.0 to 1.0 intensity modifier
    pub wind_direction: f32,         // Wind direction in radians
    pub wind_strength: f32,          // Wind strength 0.0 to 1.0
    pub visibility_modifier: f32,    // Multiplier for vision range
    pub movement_modifier: f32,      // Multiplier for movement speed
}
```

### Weather Types

The system supports 5 distinct weather conditions:

| Weather Type | Visibility Impact | Movement Impact | Tactical Effect |
|--------------|------------------|-----------------|-----------------|
| **Clear** | No reduction (100%) | No penalty (100%) | Optimal conditions |
| **Overcast** | Minor reduction (90%) | No penalty (100%) | Slight tactical impact |
| **Light Rain** | Moderate reduction (80%) | Minor penalty (95%) | Defensive advantage |
| **Heavy Rain** | Major reduction (60%) | Significant penalty (80%) | Ambush opportunities |
| **Fog** | Severe reduction (40%) | Minor penalty (90%) | Close-quarters combat |

## Gameplay Integration

### Movement System

Environmental modifiers are applied directly to unit movement speed:

```rust
let environmental_speed = unit.movement_speed * environmental_state.movement_modifier;
```

### Combat Detection

Visibility modifiers affect unit detection ranges for combat engagement:

```rust
let effective_range = unit.range * environmental_state.visibility_modifier;
```

### Visual Effects

#### Weather Particles

- **Rain**: Animated droplets falling with wind influence
- **Fog**: Slowly moving cloud particles with alpha transparency
- **Particle Physics**: Realistic gravity, wind, and lifecycle management

#### Dynamic Lighting

- **Time-of-Day**: Color temperature and intensity changes
- **Weather Influence**: Overcast reduces brightness, fog creates diffuse lighting
- **Historical Accuracy**: Starts at late afternoon (October 17, 2019 timeline)

## System Architecture

### Core Systems

1. **`update_environmental_time`**: Manages time progression and day/night cycles
2. **`update_ambient_lighting`**: Adjusts scene lighting based on time and weather
3. **`spawn_weather_particles`**: Creates weather-specific particle effects
4. **`update_weather_particles`**: Updates particle positions and lifecycle
5. **`trigger_weather_change`**: Handles random weather transitions

### Integration Points

- **Movement System**: Applies movement speed modifiers
- **Combat System**: Uses visibility modifiers for detection
- **Particle System**: Manages weather effect rendering
- **Audio System**: Could be extended for weather-specific audio

## Configuration

### Time Progression

- **Speed**: Very slow progression (0.01 multiplier) for extended battles
- **Historical Start**: 18:00 (6 PM) representing late afternoon operation timing

### Weather Transitions

- **Frequency**: Random changes every 2-5 minutes
- **Probability Distribution**: 
  - Clear: 60% (most common)
  - Overcast: 15%
  - Light Rain: 10%
  - Heavy Rain: 5%
  - Fog: 10%

### Visual Parameters

- **Particle Density**: Weather-specific spawn rates
- **Wind Effects**: Influences particle movement direction
- **Lighting Curves**: Smooth transitions between day/night states

## Console Feedback

### Real-Time Status

The system provides tactical feedback every 30 seconds:

```
🕒 Time: 18:00 (Evening) | Weather: HeavyRain | Visibility: 60% | Movement: 80%
```

### Weather Change Notifications

```
🌤️ Weather changed to: HeavyRain (Intensity: 0.8)
📊 Tactical Impact: Severely impaired visibility and movement - ambush opportunities increased
```

### Environmental Effect Warnings

```
🌧️ Heavy rain reduces visibility by 40% and slows movement by 20%
🌫️ Dense fog severely limits visibility by 60% - units harder to detect
```

## Performance Considerations

### Particle Management

- **Automatic Cleanup**: Particles despawn after reaching maximum lifetime
- **Spawn Rate Limiting**: Weather-specific spawn throttling to maintain 60+ FPS
- **Distance Culling**: Particles removed when falling below ground level

### Update Frequency

- **Time Updates**: Every frame with small delta increments
- **Weather Transitions**: Checked every frame but triggered on timer basis
- **Lighting Updates**: Only when environmental state changes (change detection)

## Historical Accuracy

### Timeline Integration

- **Starting Conditions**: Late afternoon matching historical October 17, 2019 timing
- **Seasonal Weather**: Probability distributions match Culiacán climate patterns
- **Tactical Realism**: Weather effects reflect real-world impact on urban combat

### Educational Value

The environmental system enhances the educational simulation by:
- Demonstrating how weather affects urban warfare dynamics
- Showing the importance of environmental factors in tactical planning
- Providing realistic battlefield conditions for historical accuracy

## Future Enhancements

### Potential Additions

1. **Seasonal Variations**: Different weather patterns for different months
2. **Weather-Specific Audio**: Rain sounds, wind effects, thunder
3. **Advanced Particle Effects**: More sophisticated weather rendering
4. **Environmental Abilities**: Weather-specific tactical abilities
5. **Climate Data Integration**: Real historical weather data for October 17, 2019

### Performance Optimizations

1. **LOD System**: Distance-based particle detail reduction
2. **Batched Rendering**: Group similar particles for efficient rendering
3. **Predictive Loading**: Pre-calculate weather transitions
4. **Memory Pooling**: Reuse particle entities to reduce allocation overhead

## Technical Implementation Notes

### Bevy ECS Integration

- Uses Bevy's resource system for global environmental state
- Particle effects implemented as ECS entities with components
- System scheduling ensures proper update order

### Mathematical Models

- **Time Progression**: Linear interpolation with wraparound
- **Weather Intensity**: Exponential modifiers for realistic impact curves
- **Particle Physics**: Basic gravity and wind force simulation

This environmental system significantly enhances the tactical depth and visual appeal of Battle of Culiacán RTS while maintaining historical accuracy and educational value.