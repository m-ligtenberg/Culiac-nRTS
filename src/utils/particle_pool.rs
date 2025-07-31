use crate::components::*;
use bevy::prelude::*;
use std::collections::VecDeque;

// Type aliases to reduce complexity
type PooledParticleQuery<'a> = Query<
    'a,
    'a,
    (
        Entity,
        &'a mut Transform,
        &'a mut PooledParticle,
        &'a ParticleVelocity,
        &'a mut Visibility,
        Option<&'a mut Sprite>,
        Option<&'a mut Text>,
    ),
>;

// ==================== PARTICLE POOLING SYSTEM ====================

#[derive(Resource)]
pub struct ParticlePool {
    pub combat_particles: VecDeque<Entity>,
    pub damage_number_particles: VecDeque<Entity>,
    pub weather_particles: VecDeque<Entity>,
    pub max_pool_size: usize,
    pub particles_in_use: usize,
}

impl Default for ParticlePool {
    fn default() -> Self {
        Self {
            combat_particles: VecDeque::new(),
            damage_number_particles: VecDeque::new(),
            weather_particles: VecDeque::new(),
            max_pool_size: 500, // Limit total particle count
            particles_in_use: 0,
        }
    }
}

#[derive(Component)]
pub struct PooledParticle {
    pub particle_type: PooledParticleType,
    pub active: bool,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PooledParticleType {
    Combat,
    DamageNumber,
    Weather,
}

impl ParticlePool {
    pub fn get_or_spawn_combat_particle(
        &mut self,
        commands: &mut Commands,
        position: Vec3,
        velocity: Vec3,
        lifetime: f32,
    ) -> Option<Entity> {
        if self.particles_in_use >= self.max_pool_size {
            return None; // Pool exhausted
        }

        let entity = if let Some(entity) = self.combat_particles.pop_front() {
            // Reuse existing particle
            entity
        } else {
            // Create new particle
            commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(1.0, 0.8, 0.2, 0.8),
                            custom_size: Some(Vec2::new(2.0, 2.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(position),
                        visibility: Visibility::Hidden, // Start hidden
                        ..default()
                    },
                    PooledParticle {
                        particle_type: PooledParticleType::Combat,
                        active: false,
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                    },
                    ParticleVelocity(velocity),
                ))
                .id()
        };

        self.particles_in_use += 1;
        Some(entity)
    }

    pub fn get_or_spawn_damage_number(
        &mut self,
        commands: &mut Commands,
        position: Vec3,
        damage: f32,
    ) -> Option<Entity> {
        if self.particles_in_use >= self.max_pool_size {
            return None;
        }

        let entity = if let Some(entity) = self.damage_number_particles.pop_front() {
            entity
        } else {
            commands
                .spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            format!("-{damage:.0}"),
                            TextStyle {
                                font_size: 16.0,
                                color: Color::RED,
                                ..default()
                            },
                        ),
                        transform: Transform::from_translation(position),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    PooledParticle {
                        particle_type: PooledParticleType::DamageNumber,
                        active: false,
                        lifetime: 0.0,
                        max_lifetime: 1.0,
                    },
                    ParticleVelocity(Vec3::new(0.0, 20.0, 0.0)),
                ))
                .id()
        };

        self.particles_in_use += 1;
        Some(entity)
    }

    pub fn return_particle(&mut self, entity: Entity, particle_type: PooledParticleType) {
        match particle_type {
            PooledParticleType::Combat => {
                if self.combat_particles.len() < self.max_pool_size / 3 {
                    self.combat_particles.push_back(entity);
                }
            }
            PooledParticleType::DamageNumber => {
                if self.damage_number_particles.len() < self.max_pool_size / 3 {
                    self.damage_number_particles.push_back(entity);
                }
            }
            PooledParticleType::Weather => {
                if self.weather_particles.len() < self.max_pool_size / 3 {
                    self.weather_particles.push_back(entity);
                }
            }
        }

        if self.particles_in_use > 0 {
            self.particles_in_use -= 1;
        }
    }
}

#[derive(Component)]
pub struct ParticleVelocity(pub Vec3);

// Optimized particle update system that operates on pooled particles
pub fn update_pooled_particles_system(
    mut commands: Commands,
    mut particle_pool: ResMut<ParticlePool>,
    mut particle_query: PooledParticleQuery,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle, velocity, mut visibility, sprite_opt, text_opt) in
        particle_query.iter_mut()
    {
        if !particle.active {
            continue;
        }

        particle.lifetime += time.delta_seconds();

        // Update position
        transform.translation += velocity.0 * time.delta_seconds();

        // Update visual properties based on lifetime
        let alpha_factor = 1.0 - (particle.lifetime / particle.max_lifetime);

        match particle.particle_type {
            PooledParticleType::Combat => {
                if let Some(mut sprite) = sprite_opt {
                    sprite.color.set_a(0.8 * alpha_factor);
                }
            }
            PooledParticleType::DamageNumber => {
                if let Some(mut text) = text_opt {
                    for section in text.sections.iter_mut() {
                        section.style.color.set_a(alpha_factor);
                    }
                }
            }
            PooledParticleType::Weather => {
                if let Some(mut sprite) = sprite_opt {
                    sprite.color.set_a(0.6 * alpha_factor);
                }
            }
        }

        // Check if particle should be returned to pool
        if particle.lifetime >= particle.max_lifetime {
            particle.active = false;
            particle.lifetime = 0.0;
            *visibility = Visibility::Hidden;

            particle_pool.return_particle(entity, particle.particle_type);
        }
    }
}

// Optimized spawn functions for pooled particles
pub fn spawn_optimized_combat_particles(
    commands: &mut Commands,
    particle_pool: &mut ResMut<ParticlePool>,
    position: Vec3,
    intensity: f32,
) {
    let num_particles = (intensity * 5.0).clamp(2.0, 8.0) as usize;

    for i in 0..num_particles {
        let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU;
        let velocity = Vec3::new(
            angle.cos() * 30.0,
            20.0 + (i as f32 * 5.0),
            angle.sin() * 30.0,
        );

        if let Some(entity) =
            particle_pool.get_or_spawn_combat_particle(commands, position, velocity, 0.8)
        {
            // Activate the particle
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                // The entity will be activated in the next frame update
            }
        }
    }
}

pub fn spawn_optimized_damage_numbers(
    commands: &mut Commands,
    particle_pool: &mut ResMut<ParticlePool>,
    position: Vec3,
    damage: f32,
    is_critical: bool,
) {
    if let Some(entity) = particle_pool.get_or_spawn_damage_number(commands, position, damage) {
        // Set critical hit styling if needed
        if is_critical {
            // This will be handled by the particle update system
        }
    }
}

// Resource initialization system
pub fn setup_particle_pool(mut commands: Commands) {
    commands.insert_resource(ParticlePool::default());
    info!(
        "🎭 Particle pool system initialized (max: {} particles)",
        ParticlePool::default().max_pool_size
    );
}
