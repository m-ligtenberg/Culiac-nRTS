use bevy::log::info;
use bevy::prelude::*;
use rand::prelude::*;
use std::f32::consts::PI;

#[derive(Resource)]
pub struct EnvironmentalState {
    pub time_of_day: f32, // 0.0 = midnight, 0.5 = noon, 1.0 = midnight
    pub weather_type: WeatherType,
    pub weather_intensity: f32,   // 0.0 to 1.0
    pub wind_direction: f32,      // radians
    pub wind_strength: f32,       // 0.0 to 1.0
    pub visibility_modifier: f32, // multiplier for vision range
    pub movement_modifier: f32,   // multiplier for movement speed
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeatherType {
    Clear,
    Overcast,
    LightRain,
    HeavyRain,
    Fog,
}

#[derive(Component)]
pub struct WeatherParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub particle_type: WeatherParticleType,
}

#[derive(Clone, Copy)]
pub enum WeatherParticleType {
    Rain,
    Fog,
    Dust,
}

#[derive(Resource)]
pub struct EnvironmentalAmbientLight {
    pub base_color: Color,
    pub intensity_modifier: f32,
}

impl Default for EnvironmentalAmbientLight {
    fn default() -> Self {
        Self {
            base_color: Color::rgb(1.0, 0.95, 0.8),
            intensity_modifier: 0.8,
        }
    }
}

impl Default for EnvironmentalState {
    fn default() -> Self {
        Self {
            time_of_day: 0.75, // Late afternoon (October 17, 2019 was around 3-4 PM)
            weather_type: WeatherType::Clear,
            weather_intensity: 0.0,
            wind_direction: 0.0,
            wind_strength: 0.2,
            visibility_modifier: 1.0,
            movement_modifier: 1.0,
        }
    }
}

impl EnvironmentalState {
    pub fn get_ambient_light_color(&self) -> Color {
        let time_factor = (self.time_of_day * 2.0 * PI).cos();

        match self.weather_type {
            WeatherType::Clear => {
                if self.time_of_day < 0.25 || self.time_of_day > 0.75 {
                    // Night/Evening - cooler tones
                    Color::rgb(
                        0.2 + time_factor * 0.1,
                        0.2 + time_factor * 0.15,
                        0.4 + time_factor * 0.2,
                    )
                } else {
                    // Day - warmer tones
                    Color::rgb(1.0, 0.95 + time_factor * 0.05, 0.8 + time_factor * 0.1)
                }
            }
            WeatherType::Overcast => Color::rgb(0.6, 0.6, 0.7) * (0.7 + time_factor * 0.2),
            WeatherType::LightRain | WeatherType::HeavyRain => {
                Color::rgb(0.4, 0.45, 0.6) * (0.6 + time_factor * 0.15)
            }
            WeatherType::Fog => Color::rgb(0.7, 0.7, 0.8) * (0.5 + time_factor * 0.2),
        }
    }

    pub fn get_ambient_intensity(&self) -> f32 {
        let base_intensity = if self.time_of_day < 0.25 || self.time_of_day > 0.75 {
            0.3 // Night
        } else {
            0.8 // Day
        };

        let weather_modifier = match self.weather_type {
            WeatherType::Clear => 1.0,
            WeatherType::Overcast => 0.8,
            WeatherType::LightRain => 0.7,
            WeatherType::HeavyRain => 0.6,
            WeatherType::Fog => 0.5,
        };

        base_intensity * weather_modifier
    }

    pub fn update_gameplay_modifiers(&mut self) {
        let old_visibility = self.visibility_modifier;
        let old_movement = self.movement_modifier;

        // Weather affects visibility and movement
        match self.weather_type {
            WeatherType::Clear => {
                self.visibility_modifier = 1.0;
                self.movement_modifier = 1.0;
            }
            WeatherType::Overcast => {
                self.visibility_modifier = 0.9;
                self.movement_modifier = 1.0;
            }
            WeatherType::LightRain => {
                self.visibility_modifier = 0.8;
                self.movement_modifier = 0.95;
            }
            WeatherType::HeavyRain => {
                self.visibility_modifier = 0.6;
                self.movement_modifier = 0.8;
            }
            WeatherType::Fog => {
                self.visibility_modifier = 0.4;
                self.movement_modifier = 0.9;
            }
        }

        // Adjust based on intensity
        let intensity_factor = self.weather_intensity;
        self.visibility_modifier = 1.0 - (1.0 - self.visibility_modifier) * intensity_factor;
        self.movement_modifier = 1.0 - (1.0 - self.movement_modifier) * intensity_factor;

        // Console feedback for significant environmental effects
        if (old_visibility - self.visibility_modifier).abs() > 0.1
            || (old_movement - self.movement_modifier).abs() > 0.1
        {
            match self.weather_type {
                WeatherType::HeavyRain => info!(
                    "🌧️ Heavy rain reduces visibility by {}% and slows movement by {}%",
                    ((1.0 - self.visibility_modifier) * 100.0) as i32,
                    ((1.0 - self.movement_modifier) * 100.0) as i32
                ),
                WeatherType::Fog => info!(
                    "🌫️ Dense fog severely limits visibility by {}% - units harder to detect",
                    ((1.0 - self.visibility_modifier) * 100.0) as i32
                ),
                WeatherType::LightRain => info!(
                    "🌦️ Light rain reduces visibility by {}% and movement by {}%",
                    ((1.0 - self.visibility_modifier) * 100.0) as i32,
                    ((1.0 - self.movement_modifier) * 100.0) as i32
                ),
                _ => {}
            }
        }
    }
}

pub fn update_environmental_time(
    time: Res<Time>,
    mut env_state: ResMut<EnvironmentalState>,
    mut time_display_timer: Local<f32>,
) {
    // Time progresses slowly during battle
    let time_speed = 0.01; // Very slow progression
    let old_time = env_state.time_of_day;
    env_state.time_of_day = (env_state.time_of_day + time.delta_seconds() * time_speed) % 1.0;
    env_state.update_gameplay_modifiers();

    // Display time of day status every 30 seconds
    *time_display_timer += time.delta_seconds();
    if *time_display_timer > 30.0 {
        *time_display_timer = 0.0;

        let time_hour = (env_state.time_of_day * 24.0) as i32;
        let time_period = if time_hour < 6 || time_hour >= 20 {
            "Night"
        } else if time_hour < 12 {
            "Morning"
        } else if time_hour < 18 {
            "Afternoon"
        } else {
            "Evening"
        };

        info!(
            "🕒 Time: {:02}:00 ({}) | Weather: {:?} | Visibility: {:.0}% | Movement: {:.0}%",
            time_hour,
            time_period,
            env_state.weather_type,
            env_state.visibility_modifier * 100.0,
            env_state.movement_modifier * 100.0
        );
    }

    // Check for major time transitions (day/night cycle impacts)
    let night_threshold = 0.8; // 8 PM
    let dawn_threshold = 0.25; // 6 AM

    if (old_time < night_threshold && env_state.time_of_day >= night_threshold)
        || (old_time < dawn_threshold && env_state.time_of_day >= dawn_threshold)
    {
        let light_level =
            if env_state.time_of_day >= night_threshold || env_state.time_of_day < dawn_threshold {
                "darkness"
            } else {
                "daylight"
            };
        info!(
            "🌅 Environmental transition: {} affects unit detection and ambient lighting",
            light_level
        );
    }
}

pub fn update_ambient_lighting(
    env_state: Res<EnvironmentalState>,
    mut ambient_light_res: ResMut<EnvironmentalAmbientLight>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    if env_state.is_changed() {
        let new_color = env_state.get_ambient_light_color();
        let new_intensity = env_state.get_ambient_intensity();

        ambient_light_res.base_color = new_color;
        ambient_light_res.intensity_modifier = new_intensity;

        // Update Bevy's ambient light
        ambient_light.color = new_color;
        ambient_light.brightness = new_intensity;
    }
}

pub fn spawn_weather_particles(
    mut commands: Commands,
    env_state: Res<EnvironmentalState>,
    time: Res<Time>,
    mut particle_spawn_timer: Local<f32>,
) {
    *particle_spawn_timer += time.delta_seconds();

    if *particle_spawn_timer < 0.1 {
        return;
    }
    *particle_spawn_timer = 0.0;

    match env_state.weather_type {
        WeatherType::LightRain | WeatherType::HeavyRain => {
            let spawn_rate = match env_state.weather_type {
                WeatherType::LightRain => 5,
                WeatherType::HeavyRain => 15,
                _ => 0,
            };

            for _ in 0..spawn_rate {
                let mut rng = thread_rng();
                let x = rng.gen::<f32>() * 40.0 - 20.0;
                let z = rng.gen::<f32>() * 40.0 - 20.0;
                let y = 15.0;

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(0.7, 0.8, 1.0, 0.6),
                            custom_size: Some(Vec2::new(0.1, 0.5)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(x, y, z)),
                        ..default()
                    },
                    WeatherParticle {
                        velocity: Vec3::new(
                            env_state.wind_direction.cos() * env_state.wind_strength * 2.0,
                            -8.0,
                            env_state.wind_direction.sin() * env_state.wind_strength * 2.0,
                        ),
                        lifetime: 0.0,
                        max_lifetime: 2.0,
                        particle_type: WeatherParticleType::Rain,
                    },
                ));
            }
        }
        WeatherType::Fog => {
            let mut rng = thread_rng();
            if rng.gen::<f32>() < 0.3 {
                let x = rng.gen::<f32>() * 50.0 - 25.0;
                let z = rng.gen::<f32>() * 50.0 - 25.0;
                let y = rng.gen::<f32>() * 5.0 + 2.0;

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(0.9, 0.9, 0.95, 0.2),
                            custom_size: Some(Vec2::new(3.0, 3.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(x, y, z)),
                        ..default()
                    },
                    WeatherParticle {
                        velocity: Vec3::new(
                            env_state.wind_direction.cos() * env_state.wind_strength * 0.5,
                            0.0,
                            env_state.wind_direction.sin() * env_state.wind_strength * 0.5,
                        ),
                        lifetime: 0.0,
                        max_lifetime: 10.0,
                        particle_type: WeatherParticleType::Fog,
                    },
                ));
            }
        }
        _ => {}
    }
}

pub fn update_weather_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut WeatherParticle, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, mut sprite) in particle_query.iter_mut() {
        particle.lifetime += time.delta_seconds();

        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // Update position
        transform.translation += particle.velocity * time.delta_seconds();

        // Update alpha based on lifetime
        let alpha_factor = 1.0 - (particle.lifetime / particle.max_lifetime);
        match particle.particle_type {
            WeatherParticleType::Rain => {
                sprite.color.set_a(0.6 * alpha_factor);
            }
            WeatherParticleType::Fog => {
                sprite.color.set_a(0.2 * alpha_factor);
                // Fog particles grow slightly over time
                let scale = 1.0 + particle.lifetime * 0.1;
                transform.scale = Vec3::splat(scale);
            }
            WeatherParticleType::Dust => {
                sprite.color.set_a(0.4 * alpha_factor);
            }
        }

        // Remove particles that fall below ground
        if transform.translation.y < -1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn trigger_weather_change(
    mut env_state: ResMut<EnvironmentalState>,
    time: Res<Time>,
    mut weather_timer: Local<f32>,
) {
    *weather_timer += time.delta_seconds();

    // Weather changes every 2-5 minutes during battle
    let mut rng = thread_rng();
    if *weather_timer > 120.0 + rng.gen::<f32>() * 180.0 {
        *weather_timer = 0.0;

        // Random weather transition (historically accurate for October in Culiacán)
        env_state.weather_type = match rng.gen_range(0..=100u32) {
            0..=60 => WeatherType::Clear,
            61..=75 => WeatherType::Overcast,
            76..=85 => WeatherType::LightRain,
            86..=90 => WeatherType::HeavyRain,
            91..=100 => WeatherType::Fog,
            _ => WeatherType::Clear,
        };

        env_state.weather_intensity = 0.5 + rng.gen::<f32>() * 0.5;
        env_state.wind_direction = rng.gen::<f32>() * 2.0 * PI;
        env_state.wind_strength = 0.1 + rng.gen::<f32>() * 0.4;

        let tactical_info = match env_state.weather_type {
            WeatherType::Clear => {
                "Optimal visibility and movement - all units at full effectiveness"
            }
            WeatherType::Overcast => "Slightly reduced visibility - minor tactical impact",
            WeatherType::LightRain => {
                "Reduced visibility and movement - consider defensive positions"
            }
            WeatherType::HeavyRain => {
                "Severely impaired visibility and movement - ambush opportunities increased"
            }
            WeatherType::Fog => "Extremely limited visibility - close-quarters combat favored",
        };

        info!(
            "🌤️ Weather changed to: {:?} (Intensity: {:.1})",
            env_state.weather_type, env_state.weather_intensity
        );
        info!("📊 Tactical Impact: {}", tactical_info);
    }
}
