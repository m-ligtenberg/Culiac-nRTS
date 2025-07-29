use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ==================== DIFFICULTY COMPONENTS ====================

#[derive(Component)]
pub struct DifficultySettings;

#[derive(Component)]
pub struct DifficultyToggleButton;

#[derive(Component)]
pub struct DifficultyDisplay;

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
    CartelSniperRifle,  // High precision, long range
    LMG,                // Light machine gun for suppression
    MedicBag,           // Healing equipment
    // Military weapons  
    StandardIssue,
    TacticalRifle,
    MilitarySniperRifle,
    VehicleWeapons,
    TankCannon,         // Heavy artillery
    HelicopterWeapons,  // Air-to-ground systems
    EngineerTools,      // Construction/repair equipment
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
    pub squad_id: u32,
    pub formation_center: Vec3,
    pub formation_facing: f32, // Rotation in radians
}

#[derive(Clone, PartialEq, Debug)]
pub enum FormationType {
    Line,           // Linear formation for defensive positions
    Circle,         // Defensive circle around high-value target
    Wedge,          // Assault formation for advancing
    Flanking,       // Split formation for flanking maneuvers
    Overwatch,      // Supporting fire positions
    Retreat,        // Tactical withdrawal formation
}

// ==================== COORDINATION COMPONENTS ====================

#[derive(Component)]
pub struct Squad {
    pub id: u32,
    pub leader: Option<Entity>,
    pub members: Vec<Entity>,
    pub squad_type: SquadType,
    pub current_objective: SquadObjective,
    pub rally_point: Option<Vec3>,
    pub cohesion_radius: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SquadType {
    AssaultTeam,     // Aggressive front-line units
    SupportTeam,     // Covering fire and overwatch
    ReconTeam,       // Scouting and intelligence
    SecurityTeam,    // Defensive perimeter units
}

#[derive(Clone, PartialEq, Debug)]
pub enum SquadObjective {
    Advance(Vec3),           // Move to position
    Flank(Vec3, Vec3),       // Flank target from position
    Defend(Vec3),            // Hold defensive position
    Retreat(Vec3),           // Tactical withdrawal to point
    Support(Entity),         // Support another unit/squad
    Suppress(Vec3),          // Suppressive fire on area
    Regroup(Vec3),          // Rally at location
}

#[derive(Component)]
pub struct Communication {
    pub radio_range: f32,
    pub last_report_time: f32,
    pub known_enemies: Vec<EnemyContact>,
    pub received_orders: Vec<TacticalOrder>,
}

#[derive(Clone, Debug)]
pub struct EnemyContact {
    pub position: Vec3,
    pub enemy_type: UnitType,
    pub confidence: f32,     // 0.0 to 1.0
    pub last_seen: f32,      // Time since last spotted
}

#[derive(Clone, Debug)]
pub struct TacticalOrder {
    pub order_type: OrderType,
    pub target_position: Option<Vec3>,
    pub target_entity: Option<Entity>,
    pub priority: u8,        // 1-10, higher is more urgent
    pub issued_time: f32,
}

#[derive(Clone, PartialEq, Debug)]
pub enum OrderType {
    Attack(Vec3),
    Defend(Vec3),
    Retreat(Vec3),
    Flank(Vec3),
    Support(Entity),
    ReportPosition,
    SuppressArea(Vec3),
    TakeCover,
}

#[derive(Component)]
pub struct TacticalState {
    pub current_state: TacticalMode,
    pub state_timer: f32,
    pub last_state_change: f32,
    pub suppression_level: f32,  // 0.0 to 1.0, affects accuracy and movement
    pub morale: f32,             // 0.0 to 1.0, affects decision making
}

#[derive(Clone, PartialEq, Debug)]
pub enum TacticalMode {
    Advancing,      // Moving toward objective
    Engaging,       // In active combat
    Retreating,     // Tactical withdrawal
    Suppressed,     // Pinned down by enemy fire
    Flanking,       // Executing flanking maneuver
    Overwatch,      // Providing covering fire
    Regrouping,     // Moving to rally point
    HoldPosition,   // Maintaining defensive stance
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

// Victory/Defeat Components
#[derive(Component)]
pub struct VictoryScreen;

#[derive(Component)]
pub struct DefeatScreen;

#[derive(Component)]
pub struct MissionResultText;

#[derive(Component)]
pub struct ContinueButton;

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
    PrecisionShot,   // Sniper's high-damage single shot
    SuppressiveFire, // Heavy gunner area suppression
    FieldMedic,      // Heal nearby allies
    // Military abilities  
    FragGrenade,     // Area damage
    AirStrike,       // Long range bombardment
    TacticalRetreat, // Temporary speed boost + damage reduction
    TankShell,       // Heavy artillery strike
    StrafeRun,       // Helicopter attack run
    DeployBarricade, // Engineer deploys cover
    RepairVehicle,   // Engineer repairs damaged units
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
    Healing(f32),        // Health regeneration over time
    Suppressed,          // Reduced accuracy and movement
    ArmorPiercing,       // Bypass armor bonuses
    AerialView,          // Helicopter spotting bonus
    Fortified,           // Engineer cover bonus
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
    Sniper,      // Long-range precision unit
    HeavyGunner, // High damage, slow movement
    Medic,       // Healing and support unit
    // Military units  
    Soldier,
    SpecialForces,
    Vehicle,
    Tank,        // Heavy armor and firepower
    Helicopter,  // Air support unit
    Engineer,    // Deployable structures and repairs
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
    Victory,       // Mission completed successfully
    Defeat,        // Mission failed
    GameOver,      // Final game over state
}