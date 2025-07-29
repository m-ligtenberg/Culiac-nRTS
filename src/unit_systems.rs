use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

// ==================== UNIT CONFIGURATION SYSTEM ====================

pub fn configure_unit_stats(unit: &mut Unit, unit_type: &UnitType, faction: &Faction) {
    match unit_type {
        // Cartel units
        UnitType::Sicario => {
            unit.health = 80.0;
            unit.max_health = 80.0;
            unit.damage = 25.0;
            unit.range = 120.0;
            unit.movement_speed = 45.0;
            unit.equipment = Equipment {
                weapon: WeaponType::AssaultRifle,
                armor: ArmorType::LightVest,
                upgrades: vec![],
            };
        },
        UnitType::Enforcer => {
            unit.health = 120.0;
            unit.max_health = 120.0;
            unit.damage = 35.0;
            unit.range = 100.0;
            unit.movement_speed = 35.0;
            unit.equipment = Equipment {
                weapon: WeaponType::HeavyMachineGun,
                armor: ArmorType::TacticalVest,
                upgrades: vec![],
            };
        },
        UnitType::Sniper => {
            unit.health = 60.0;
            unit.max_health = 60.0;
            unit.damage = 80.0;   // High damage
            unit.range = 250.0;   // Very long range
            unit.movement_speed = 25.0; // Slow movement
            unit.equipment = Equipment {
                weapon: WeaponType::CartelSniperRifle,
                armor: ArmorType::LightVest,
                upgrades: vec![UpgradeType::ScopedSight],
            };
        },
        UnitType::HeavyGunner => {
            unit.health = 150.0;
            unit.max_health = 150.0;
            unit.damage = 45.0;
            unit.range = 140.0;
            unit.movement_speed = 20.0; // Very slow
            unit.equipment = Equipment {
                weapon: WeaponType::LMG,
                armor: ArmorType::HeavyArmor,
                upgrades: vec![UpgradeType::ExtendedMag],
            };
        },
        UnitType::Medic => {
            unit.health = 70.0;
            unit.max_health = 70.0;
            unit.damage = 15.0;   // Low combat ability
            unit.range = 80.0;
            unit.movement_speed = 40.0;
            unit.equipment = Equipment {
                weapon: WeaponType::MedicBag,
                armor: ArmorType::LightVest,
                upgrades: vec![UpgradeType::RadioComms],
            };
        },
        UnitType::Ovidio => {
            unit.health = 200.0;
            unit.max_health = 200.0;
            unit.damage = 40.0;
            unit.range = 110.0;
            unit.movement_speed = 30.0;
            unit.equipment = Equipment {
                weapon: WeaponType::AssaultRifle,
                armor: ArmorType::HeavyArmor,
                upgrades: vec![UpgradeType::ReinforcedArmor, UpgradeType::RadioComms],
            };
        },
        
        // Military units
        UnitType::Soldier => {
            unit.health = 100.0;
            unit.max_health = 100.0;
            unit.damage = 30.0;
            unit.range = 130.0;
            unit.movement_speed = 40.0;
            unit.equipment = Equipment {
                weapon: WeaponType::StandardIssue,
                armor: ArmorType::TacticalVest,
                upgrades: vec![],
            };
        },
        UnitType::SpecialForces => {
            unit.health = 130.0;
            unit.max_health = 130.0;
            unit.damage = 45.0;
            unit.range = 150.0;
            unit.movement_speed = 50.0;
            unit.equipment = Equipment {
                weapon: WeaponType::TacticalRifle,
                armor: ArmorType::TacticalVest,
                upgrades: vec![UpgradeType::ScopedSight, UpgradeType::CombatStims],
            };
        },
        UnitType::Tank => {
            unit.health = 300.0;
            unit.max_health = 300.0;
            unit.damage = 100.0;  // Very high damage
            unit.range = 200.0;
            unit.movement_speed = 15.0; // Very slow
            unit.equipment = Equipment {
                weapon: WeaponType::TankCannon,
                armor: ArmorType::VehicleArmor,
                upgrades: vec![UpgradeType::ReinforcedArmor],
            };
        },
        UnitType::Helicopter => {
            unit.health = 80.0;   // Fragile but fast
            unit.max_health = 80.0;
            unit.damage = 60.0;
            unit.range = 180.0;
            unit.movement_speed = 80.0; // Very fast
            unit.equipment = Equipment {
                weapon: WeaponType::HelicopterWeapons,
                armor: ArmorType::None,
                upgrades: vec![UpgradeType::ScopedSight],
            };
        },
        UnitType::Engineer => {
            unit.health = 90.0;
            unit.max_health = 90.0;
            unit.damage = 20.0;   // Low combat ability
            unit.range = 100.0;
            unit.movement_speed = 35.0;
            unit.equipment = Equipment {
                weapon: WeaponType::EngineerTools,
                armor: ArmorType::TacticalVest,
                upgrades: vec![UpgradeType::RadioComms],
            };
        },
        UnitType::Vehicle => {
            unit.health = 180.0;
            unit.max_health = 180.0;
            unit.damage = 50.0;
            unit.range = 160.0;
            unit.movement_speed = 25.0;
            unit.equipment = Equipment {
                weapon: WeaponType::VehicleWeapons,
                armor: ArmorType::VehicleArmor,
                upgrades: vec![],
            };
        },
        UnitType::Roadblock => {
            unit.health = 50.0;
            unit.max_health = 50.0;
            unit.damage = 0.0;    // Defensive structure
            unit.range = 0.0;
            unit.movement_speed = 0.0; // Immobile
            unit.equipment = Equipment {
                weapon: WeaponType::BasicRifle,
                armor: ArmorType::None,
                upgrades: vec![],
            };
        },
    }
}

pub fn get_unit_abilities(unit_type: &UnitType) -> Vec<UnitAbility> {
    match unit_type {
        UnitType::Sniper => vec![
            UnitAbility {
                ability_type: AbilityType::PrecisionShot,
                cooldown: Timer::from_seconds(8.0, TimerMode::Once),
                range: 300.0,
                energy_cost: 40,
            }
        ],
        UnitType::HeavyGunner => vec![
            UnitAbility {
                ability_type: AbilityType::SuppressiveFire,
                cooldown: Timer::from_seconds(12.0, TimerMode::Once),
                range: 160.0,
                energy_cost: 50,
            }
        ],
        UnitType::Medic => vec![
            UnitAbility {
                ability_type: AbilityType::FieldMedic,
                cooldown: Timer::from_seconds(6.0, TimerMode::Once),
                range: 100.0,
                energy_cost: 30,
            }
        ],
        UnitType::Tank => vec![
            UnitAbility {
                ability_type: AbilityType::TankShell,
                cooldown: Timer::from_seconds(15.0, TimerMode::Once),
                range: 250.0,
                energy_cost: 60,
            }
        ],
        UnitType::Helicopter => vec![
            UnitAbility {
                ability_type: AbilityType::StrafeRun,
                cooldown: Timer::from_seconds(20.0, TimerMode::Once),
                range: 200.0,
                energy_cost: 70,
            }
        ],
        UnitType::Engineer => vec![
            UnitAbility {
                ability_type: AbilityType::DeployBarricade,
                cooldown: Timer::from_seconds(25.0, TimerMode::Once),
                range: 50.0,
                energy_cost: 40,
            },
            UnitAbility {
                ability_type: AbilityType::RepairVehicle,
                cooldown: Timer::from_seconds(10.0, TimerMode::Once),
                range: 80.0,
                energy_cost: 35,
            }
        ],
        UnitType::Enforcer => vec![
            UnitAbility {
                ability_type: AbilityType::BurstFire,
                cooldown: Timer::from_seconds(6.0, TimerMode::Once),
                range: 120.0,
                energy_cost: 25,
            }
        ],
        UnitType::SpecialForces => vec![
            UnitAbility {
                ability_type: AbilityType::FragGrenade,
                cooldown: Timer::from_seconds(10.0, TimerMode::Once),
                range: 140.0,
                energy_cost: 35,
            }
        ],
        _ => vec![], // Default units have no special abilities
    }
}

pub fn get_unit_emoji(unit_type: &UnitType) -> &'static str {
    match unit_type {
        UnitType::Sicario => "🔫",
        UnitType::Enforcer => "⚔️",
        UnitType::Sniper => "🎯",
        UnitType::HeavyGunner => "💥",
        UnitType::Medic => "🏥",
        UnitType::Ovidio => "👑",
        UnitType::Roadblock => "🚧",
        UnitType::Soldier => "🪖",
        UnitType::SpecialForces => "🎯",
        UnitType::Tank => "🚛",
        UnitType::Helicopter => "🚁",
        UnitType::Engineer => "🔧",
        UnitType::Vehicle => "🚗",
    }
}

pub fn get_unit_color(unit_type: &UnitType, faction: &Faction) -> Color {
    let base_color = match faction {
        Faction::Cartel => match unit_type {
            UnitType::Ovidio => Color::GOLD,
            UnitType::Sniper => Color::MAROON,
            UnitType::HeavyGunner => Color::rgb(0.5, 0.0, 0.0), // Dark red
            UnitType::Medic => Color::rgb(0.0, 0.8, 0.2), // Green cross
            _ => Color::RED,
        },
        Faction::Military => match unit_type {
            UnitType::Tank => Color::DARK_GREEN,
            UnitType::Helicopter => Color::rgb(0.0, 0.6, 0.0),
            UnitType::Engineer => Color::rgb(0.8, 0.8, 0.0), // Yellow
            UnitType::SpecialForces => Color::rgb(0.0, 1.0, 0.0), // Bright green
            _ => Color::GREEN,
        },
        _ => Color::WHITE,
    };
    base_color
}

// ==================== WEAPON UPGRADE SYSTEM ====================

pub fn apply_weapon_upgrades(unit: &mut Unit) {
    for upgrade in &unit.equipment.upgrades {
        match upgrade {
            UpgradeType::ScopedSight => {
                unit.range *= 1.25; // +25% range
            },
            UpgradeType::ExtendedMag => {
                unit.damage *= 1.33; // +33% damage per burst
            },
            UpgradeType::ReinforcedArmor => {
                unit.health *= 1.20; // +20% health
                unit.max_health *= 1.20;
            },
            UpgradeType::CombatStims => {
                unit.movement_speed *= 1.15; // +15% speed
            },
            UpgradeType::RadioComms => {
                // Communication bonuses applied elsewhere
            },
        }
    }
}

pub fn calculate_weapon_effectiveness(weapon: &WeaponType, target_armor: &ArmorType) -> f32 {
    let base_effectiveness = match weapon {
        WeaponType::CartelSniperRifle | WeaponType::MilitarySniperRifle => 1.5,  // High armor penetration
        WeaponType::TankCannon => 2.0,   // Extremely high penetration
        WeaponType::RPG => 1.8,          // Anti-armor weapon
        WeaponType::LMG => 1.2,          // Good suppression
        WeaponType::MedicBag => 0.0,     // No combat effectiveness
        WeaponType::EngineerTools => 0.5, // Minimal combat use
        WeaponType::HelicopterWeapons => 1.3, // Good effectiveness
        _ => 1.0,
    };
    
    let armor_modifier = match target_armor {
        ArmorType::None => 1.0,
        ArmorType::LightVest => 0.85,
        ArmorType::TacticalVest => 0.75,
        ArmorType::HeavyArmor => 0.60,
        ArmorType::VehicleArmor => 0.40,
    };
    
    base_effectiveness * armor_modifier
}

// ==================== UNIT ABILITY ACTIVATION ====================

pub fn can_activate_ability(ability: &UnitAbility, unit_energy: u32) -> bool {
    ability.cooldown.finished() && unit_energy >= ability.energy_cost
}

pub fn get_ability_description(ability_type: &AbilityType) -> &'static str {
    match ability_type {
        AbilityType::PrecisionShot => "Long-range high-damage shot that pierces armor",
        AbilityType::SuppressiveFire => "Area suppression that reduces enemy accuracy and movement",
        AbilityType::FieldMedic => "Heals nearby allies over time",
        AbilityType::TankShell => "Devastating area damage with massive range",
        AbilityType::StrafeRun => "Aerial attack run covering a large area",
        AbilityType::DeployBarricade => "Creates defensive cover for allies",
        AbilityType::RepairVehicle => "Restores health to damaged vehicles and structures",
        AbilityType::BurstFire => "Rapid succession of shots with increased damage",
        AbilityType::FragGrenade => "Explosive area damage",
        AbilityType::Intimidate => "Reduces enemy morale and combat effectiveness",
        AbilityType::CallBackup => "Summons reinforcement unit to the battlefield",
        AbilityType::AirStrike => "Long-range bombardment from air support",
        AbilityType::TacticalRetreat => "Temporary speed boost with damage reduction",
    }
}