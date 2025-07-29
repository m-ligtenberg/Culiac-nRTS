use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ==================== CAMERA COMPONENTS ====================

#[derive(Component)]
pub struct IsometricCamera {
    pub pan_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

// ==================== UNIT COMPONENTS ====================

#[derive(Component, Clone)]
pub struct Unit {
    pub health: f32,
    pub max_health: f32,
    pub faction: Faction,
    pub unit_type: UnitType,
    pub damage: f32,
    pub range: f32,
    pub movement_speed: f32,
    pub target: Option<Entity>,
    pub attack_cooldown: Timer,
    pub experience: u32,
    pub kills: u32,
    pub veterancy_level: VeterancyLevel,
    pub equipment: Equipment,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum VeterancyLevel {
    Recruit,    // 0-2 kills
    Veteran,    // 3-5 kills  
    Elite,      // 6+ kills
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: WeaponType,
    pub armor: ArmorType,
    pub upgrades: Vec<UpgradeType>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum WeaponType {
    // Cartel weapons
    BasicRifle,
    AssaultRifle,
    HeavyMachineGun,
    RPG,
    // Military weapons  
    StandardIssue,
    TacticalRifle,
    SniperRifle,
    VehicleWeapons,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ArmorType {
    None,
    LightVest,
    TacticalVest,
    HeavyArmor,
    VehicleArmor,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum UpgradeType {
    ScopedSight,      // +25% range
    ExtendedMag,      // +33% damage per burst
    ReinforcedArmor,  // +20% health
    CombatStims,      // +15% speed
    RadioComms,       // Coordination bonuses
}

// ==================== MOVEMENT COMPONENTS ====================

#[derive(Component)]
pub struct Movement {
    pub target_position: Option<Vec3>,
    pub speed: f32,
}

#[derive(Component)]
pub struct Formation {
    pub formation_type: FormationType,
    pub position_in_formation: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FormationType {
    Line,
    Circle,
    Wedge,
}

// ==================== UI COMPONENTS ====================

#[derive(Component)]
pub struct HealthBar {
    pub owner: Entity,
    pub offset: Vec3,
}

#[derive(Component)]
pub struct DamageIndicator {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct Selected {
    pub selection_color: Color,
}

#[derive(Component)]
pub struct UIElement;

#[derive(Component)]
pub struct StatusText;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct SelectionIndicator;

// ==================== MINIMAP COMPONENTS ====================

#[derive(Component)]
pub struct MiniMap;

#[derive(Component)]
pub struct MiniMapIcon {
    pub unit_type: UnitType,
    pub faction: Faction,
}

// ==================== VISUAL EFFECTS COMPONENTS ====================

#[derive(Component)]
pub struct ParticleEffect {
    pub lifetime: Timer,
    pub velocity: Vec3,
}

// ==================== SPAWNING COMPONENTS ====================

#[derive(Component)]
pub struct WaveSpawner {
    pub next_wave_timer: Timer,
    pub wave_number: u32,
    pub units_in_wave: u32,
}

#[derive(Component)]
pub struct Objective {
    pub objective_type: ObjectiveType,
    pub _position: Vec3,
    pub _radius: f32,
    pub _health: f32,
}

// ==================== ENUMS & TYPES ====================

#[derive(Clone, PartialEq, Debug)]
#[allow(dead_code)]
pub enum Faction {
    Cartel,
    Military,
    Civilian,
}

#[derive(Clone, PartialEq, Debug)]
pub enum UnitType {
    // Cartel units
    Sicario,
    Enforcer,
    Roadblock,
    // Military units  
    Soldier,
    SpecialForces,
    Vehicle,
    // Special
    Ovidio, // High value target
}

#[derive(Clone, PartialEq, Debug)]
#[allow(dead_code)]
pub enum ObjectiveType {
    Safehouse,      // Cartel must defend
    ExtractionPoint, // Military tries to reach
    Checkpoint,     // Control points
}

// ==================== GAME PHASE ====================

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    Preparation,    // Initial setup
    InitialRaid,   // Mission 1: Defend safehouse
    BlockConvoy,   // Mission 2: Block extraction
    ApplyPressure, // Mission 3: Escalate pressure
    HoldTheLine,   // Mission 4: Final showdown
    GameOver,
}