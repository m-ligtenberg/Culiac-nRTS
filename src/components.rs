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
pub struct MissionBriefing;

#[derive(Component)]
pub struct MissionTitle;

#[derive(Component)]
pub struct MissionDescription;

#[derive(Component)]
pub struct MissionObjectives;

// Save/Load Menu Components
#[derive(Component)]
pub struct SaveLoadMenu;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct SaveSlot {
    pub slot_id: usize,
}

#[derive(Component)]
pub struct NewGameButton;

#[derive(Component)]
pub struct MainMenuButton;

#[derive(Component)]
pub struct SelectionIndicator;

#[derive(Component)]
pub struct TargetIndicator;

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

// Animation system components
#[derive(Component)]
pub struct AnimatedSprite {
    pub animation_timer: Timer,
    pub scale_amplitude: f32,
    pub rotation_speed: f32,
    pub base_scale: Vec3,
}

#[derive(Component)]
pub struct MovementAnimation {
    pub bob_timer: Timer,
    pub bob_amplitude: f32,
    pub base_y: f32,
}

// Pathfinding components
#[derive(Component)]
pub struct PathfindingAgent {
    pub path: Vec<Vec3>,
    pub current_waypoint: usize,
    pub avoidance_radius: f32,
    pub max_speed: f32,
    pub stuck_timer: f32,
}

#[derive(Component)]
pub struct Obstacle {
    pub radius: f32,
}

// Unit ability system
#[derive(Component)]
pub struct UnitAbility {
    pub ability_type: AbilityType,
    pub cooldown: Timer,
    pub range: f32,
    pub energy_cost: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AbilityType {
    // Cartel abilities
    BurstFire,       // Rapid fire attack
    Intimidate,      // Reduce enemy morale/damage
    CallBackup,      // Summon reinforcement unit
    // Military abilities  
    FragGrenade,     // Area damage
    AirStrike,       // Long range bombardment
    TacticalRetreat, // Temporary speed boost + damage reduction
}

#[derive(Component)]
pub struct AbilityEffect {
    pub effect_type: EffectType,
    pub duration: Timer,
    pub strength: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum EffectType {
    DamageBoost(f32),
    SpeedBoost(f32), 
    DamageReduction(f32),
    Stunned,
    Intimidated,
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
    MainMenu,       // Main menu with save/load options
    SaveMenu,       // Save game menu
    LoadMenu,       // Load game menu
    MissionBriefing, // Show mission briefing screen
    Preparation,    // Initial setup
    InitialRaid,   // Mission 1: Defend safehouse
    BlockConvoy,   // Mission 2: Block extraction
    ApplyPressure, // Mission 3: Escalate pressure
    HoldTheLine,   // Mission 4: Final showdown
    GameOver,
}