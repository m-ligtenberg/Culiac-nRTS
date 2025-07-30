use bevy::prelude::*;
use crate::components::*;
use crate::utils::play_tactical_sound;
use crate::spawners::spawn_unit;

// ==================== ABILITY HELPER FUNCTIONS ====================

pub fn get_default_ability(faction: &Faction, ability_index: usize) -> Option<AbilityType> {
    match (faction, ability_index) {
        (Faction::Cartel, 0) => Some(AbilityType::BurstFire),
        (Faction::Cartel, 1) => Some(AbilityType::Intimidate),
        (Faction::Military, 0) => Some(AbilityType::FragGrenade),
        (Faction::Military, 1) => Some(AbilityType::TacticalRetreat),
        _ => None,
    }
}

pub fn get_ability_cooldown(ability_type: &AbilityType) -> f32 {
    match ability_type {
        AbilityType::BurstFire => 8.0,
        AbilityType::Intimidate => 12.0,
        AbilityType::CallBackup => 20.0,
        AbilityType::FragGrenade => 10.0,
        AbilityType::AirStrike => 15.0,
        AbilityType::TacticalRetreat => 18.0,
        AbilityType::PrecisionShot => 8.0,
        AbilityType::SuppressiveFire => 12.0,
        AbilityType::FieldMedic => 6.0,
        AbilityType::TankShell => 15.0,
        AbilityType::StrafeRun => 20.0,
        AbilityType::DeployBarricade => 25.0,
        AbilityType::RepairVehicle => 10.0,
    }
}

pub fn get_ability_range(ability_type: &AbilityType) -> f32 {
    match ability_type {
        AbilityType::BurstFire => 0.0, // Self-target
        AbilityType::Intimidate => 80.0,
        AbilityType::CallBackup => 0.0, // Self-target
        AbilityType::FragGrenade => 120.0,
        AbilityType::AirStrike => 150.0,
        AbilityType::TacticalRetreat => 0.0, // Self-target
        AbilityType::PrecisionShot => 300.0,
        AbilityType::SuppressiveFire => 160.0,
        AbilityType::FieldMedic => 100.0,
        AbilityType::TankShell => 250.0,
        AbilityType::StrafeRun => 200.0,
        AbilityType::DeployBarricade => 50.0,
        AbilityType::RepairVehicle => 80.0,
    }
}

pub fn execute_ability_simple(
    commands: &mut Commands,
    caster_entity: Entity,
    caster_position: Vec3,
    _caster_unit: &mut Unit,
    ability_type: AbilityType,
    enemy_data: &[(Entity, Vec3, UnitType, f32)],
    game_assets: &Res<crate::resources::GameAssets>,
) {
    match ability_type {
        AbilityType::BurstFire => {
            // Temporary damage boost
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::DamageBoost(1.5),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                strength: 1.5,
            });
            play_tactical_sound("ability", "Burst fire activated! Increased damage for 3 seconds");
        },
        AbilityType::Intimidate => {
            // Find nearby enemies and apply intimidation
            let intimidation_range = 80.0;
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= intimidation_range && *enemy_health > 0.0 {
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::Intimidated,
                        duration: Timer::from_seconds(5.0, TimerMode::Once),
                        strength: 0.7, // 30% damage reduction
                    });
                }
            }
            play_tactical_sound("ability", "Intimidation used! Nearby enemies are demoralized");
        },
        AbilityType::CallBackup => {
            // Spawn a reinforcement unit near the caster
            let backup_pos = caster_position + Vec3::new(30.0, 30.0, 0.0);
            spawn_unit(commands, UnitType::Sicario, Faction::Cartel, backup_pos, game_assets);
            play_tactical_sound("ability", "Backup called! Reinforcement unit arriving");
        },
        AbilityType::FragGrenade => {
            // Create area damage around target location
            create_explosion_effect_simple(commands, caster_position, 60.0, 40.0, enemy_data);
            play_tactical_sound("ability", "Frag grenade thrown! Area damage inflicted");
        },
        AbilityType::AirStrike => {
            // Delayed area bombardment
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= 100.0 && *enemy_health > 0.0 {
                    // Apply delayed damage
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::Stunned,
                        duration: Timer::from_seconds(1.0, TimerMode::Once),
                        strength: 50.0, // Damage amount
                    });
                }
            }
            play_tactical_sound("ability", "Air strike called in! Incoming bombardment");
        },
        AbilityType::TacticalRetreat => {
            // Speed boost and damage reduction
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::SpeedBoost(1.8),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                strength: 1.8,
            });
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::DamageReduction(0.5),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                strength: 0.5,
            });
            play_tactical_sound("ability", "Tactical retreat! Speed boost and damage reduction active");
        },
        AbilityType::PrecisionShot => {
            // High-damage single shot with armor piercing
            if let Some((target_entity, _, _, _)) = enemy_data.iter()
                .find(|(_, pos, _, health)| caster_position.distance(*pos) <= 250.0 && *health > 0.0) {
                commands.entity(*target_entity).insert(AbilityEffect {
                    effect_type: EffectType::ArmorPiercing,
                    duration: Timer::from_seconds(0.1, TimerMode::Once),
                    strength: 120.0, // High damage
                });
            }
            play_tactical_sound("ability", "Precision shot! High-damage armor-piercing round fired");
        },
        AbilityType::SuppressiveFire => {
            // Area suppression effect
            let suppression_range = 120.0;
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= suppression_range && *enemy_health > 0.0 {
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::Suppressed,
                        duration: Timer::from_seconds(6.0, TimerMode::Once),
                        strength: 0.6, // 40% accuracy reduction
                    });
                }
            }
            play_tactical_sound("ability", "Suppressive fire! Enemy accuracy and movement reduced");
        },
        AbilityType::FieldMedic => {
            // Heal nearby allies
            // Note: Would need ally query to implement properly, using caster for now
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::Healing(25.0),
                duration: Timer::from_seconds(5.0, TimerMode::Once),
                strength: 25.0,
            });
            play_tactical_sound("ability", "Field medic! Healing allies in the area");
        },
        AbilityType::TankShell => {
            // Massive area damage
            create_explosion_effect_simple(commands, caster_position, 100.0, 80.0, enemy_data);
            play_tactical_sound("ability", "Tank shell fired! Devastating area damage");
        },
        AbilityType::StrafeRun => {
            // Linear area attack
            for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
                let distance = caster_position.distance(*enemy_position);
                if distance <= 150.0 && *enemy_health > 0.0 {
                    commands.entity(*enemy_entity).insert(AbilityEffect {
                        effect_type: EffectType::ArmorPiercing,
                        duration: Timer::from_seconds(0.1, TimerMode::Once),
                        strength: 60.0,
                    });
                }
            }
            play_tactical_sound("ability", "Helicopter strafe run! Multiple targets engaged");
        },
        AbilityType::DeployBarricade => {
            // Create defensive cover
            let barricade_pos = caster_position + Vec3::new(40.0, 0.0, 0.0);
            spawn_unit(commands, UnitType::Roadblock, Faction::Military, barricade_pos, game_assets);
            play_tactical_sound("ability", "Barricade deployed! Defensive cover established");
        },
        AbilityType::RepairVehicle => {
            // Heal nearby vehicles/allies
            commands.entity(caster_entity).insert(AbilityEffect {
                effect_type: EffectType::Healing(40.0),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                strength: 40.0,
            });
            play_tactical_sound("ability", "Repair tools active! Vehicle health restored");
        },
    }
}

pub fn create_explosion_effect_simple(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    damage: f32,
    enemy_data: &[(Entity, Vec3, UnitType, f32)],
) {
    // Visual explosion effect
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
        let offset = Vec3::new(angle.cos() * 20.0, angle.sin() * 20.0, 0.0);
        
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.5, 0.1),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_translation(position + offset + Vec3::new(0.0, 0.0, 0.8)),
                ..default()
            },
            ParticleEffect {
                lifetime: Timer::from_seconds(0.8, TimerMode::Once),
                velocity: offset.normalize() * 80.0,
            },
        ));
    }
    
    // Apply damage to enemies in range
    for (enemy_entity, enemy_position, _, enemy_health) in enemy_data.iter() {
        let distance = position.distance(*enemy_position);
        if distance <= radius && *enemy_health > 0.0 {
            let damage_multiplier = 1.0 - (distance / radius);
            let final_damage = damage * damage_multiplier;
            
            commands.entity(*enemy_entity).insert(AbilityEffect {
                effect_type: EffectType::Stunned,
                duration: Timer::from_seconds(0.1, TimerMode::Once),
                strength: final_damage,
            });
        }
    }
}