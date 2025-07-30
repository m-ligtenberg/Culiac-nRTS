use crate::auth::models::User;
use crate::campaign::VictoryType;
use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

// ==================== MULTIPLAYER SYSTEM PLUGIN ====================

pub struct MultiplayerSystemPlugin;

impl Plugin for MultiplayerSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiplayerState>()
            .init_resource::<NetworkManager>()
            .add_systems(
                Update,
                (
                    multiplayer_lobby_system,
                    player_connection_system,
                    game_sync_system,
                    player_input_sync_system,
                    multiplayer_ui_system,
                )
                    .run_if(resource_exists::<MultiplayerState>()),
            );
    }
}

// ==================== MULTIPLAYER RESOURCES ====================

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct MultiplayerState {
    pub session_id: Uuid,
    pub is_host: bool,
    pub game_mode: MultiplayerGameMode,
    pub connected_players: HashMap<Uuid, PlayerInfo>,
    pub max_players: u8,
    pub game_started: bool,
    pub scenario: MultiplayerScenario,
    pub player_assignments: HashMap<Uuid, PlayerRole>,
    #[serde(skip)]
    pub sync_interval: Timer,
    pub connection_status: ConnectionStatus,
}

impl Default for MultiplayerState {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            is_host: false,
            game_mode: MultiplayerGameMode::Asymmetric,
            connected_players: HashMap::new(),
            max_players: 4,
            game_started: false,
            scenario: MultiplayerScenario::HistoricalOctober17,
            player_assignments: HashMap::new(),
            sync_interval: Timer::from_seconds(0.1, TimerMode::Repeating), // 10 FPS sync
            connection_status: ConnectionStatus::Disconnected,
        }
    }
}

#[derive(Resource)]
pub struct NetworkManager {
    pub message_sender: Option<mpsc::UnboundedSender<NetworkMessage>>,
    pub message_receiver: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
    pub player_id: Uuid,
    pub auth_token: Option<String>,
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self {
            message_sender: None,
            message_receiver: None,
            player_id: Uuid::new_v4(),
            auth_token: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum MultiplayerGameMode {
    Asymmetric,  // Cartel vs Military (2v2)
    Historical,  // One player as government, others as advisors
    Cooperative, // All players as cartel coordination
    Competitive, // Multiple cartel factions
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MultiplayerScenario {
    HistoricalOctober17,    // Exact historical recreation
    AlternateHistory,       // What-if scenarios
    ModernDay,              // Updated to current day
    CustomScenario(String), // User-defined scenarios
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub user_id: Uuid,
    pub username: String,
    pub role: PlayerRole,
    pub connection_status: PlayerConnectionStatus,
    pub ping: u32,
    pub ready: bool,
    pub faction_preference: Option<Faction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PlayerRole {
    CartelCommander,     // Controls cartel units
    MilitaryCommander,   // Controls military units
    GovernmentAdvisor,   // Political decisions and resource allocation
    IntelligenceOfficer, // Intelligence operations and coordination
    Observer,            // Watch-only mode
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum PlayerConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    TimedOut,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Hosting,
    Error(String),
}

// ==================== NETWORK MESSAGES ====================

#[derive(Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    // Connection management
    PlayerJoin {
        player_info: PlayerInfo,
    },
    PlayerLeave {
        player_id: Uuid,
    },
    PlayerReady {
        player_id: Uuid,
        ready: bool,
    },

    // Game synchronization
    GameStateSync {
        game_state: GameStateSyncData,
    },
    UnitCommand {
        player_id: Uuid,
        command: UnitCommand,
    },
    PoliticalDecision {
        player_id: Uuid,
        decision: PoliticalDecision,
    },

    // Communication
    ChatMessage {
        player_id: Uuid,
        message: String,
        channel: ChatChannel,
    },
    VoiceChat {
        player_id: Uuid,
        audio_data: Vec<u8>,
    },

    // Game events
    GameStart {
        scenario: MultiplayerScenario,
    },
    GamePause {
        player_id: Uuid,
    },
    GameEnd {
        result: GameResult,
    },

    // Authentication
    AuthRequest {
        token: String,
    },
    AuthResponse {
        success: bool,
        player_id: Uuid,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameStateSyncData {
    pub timestamp: f64,
    pub unit_positions: HashMap<Entity, Vec3>,
    pub unit_health: HashMap<Entity, f32>,
    pub political_state: Option<crate::political_system::PoliticalState>,
    pub game_phase: GamePhase,
    pub resources: HashMap<Faction, u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UnitCommand {
    pub unit_id: Entity,
    pub command_type: CommandType,
    pub target_position: Option<Vec3>,
    pub target_entity: Option<Entity>,
    pub formation: Option<Formation>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CommandType {
    Move,
    Attack,
    Defend,
    Retreat,
    UseAbility(String),
    ChangeFormation,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PoliticalDecision {
    pub decision_type: PoliticalDecisionType,
    pub parameters: HashMap<String, f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PoliticalDecisionType {
    EscalateForce,
    WithdrawTroops,
    NegotiateCeasefire,
    RequestInternationalSupport,
    MediaStatement,
    ChangeOperationScope,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ChatChannel {
    All,
    Team,
    Private(Uuid),
    Command, // For role-based strategic communication
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub winner: Option<Faction>,
    pub end_condition: VictoryType,
    pub player_stats: HashMap<Uuid, PlayerStats>,
    pub duration: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub units_controlled: u32,
    pub units_lost: u32,
    pub damage_dealt: f32,
    pub objectives_completed: u32,
    pub political_influence_used: f32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Formation {
    Line,
    Column,
    Wedge,
    Box,
    Scattered,
}

// ==================== MULTIPLAYER LOBBY SYSTEM ====================

pub fn multiplayer_lobby_system(
    mut multiplayer_state: ResMut<MultiplayerState>,
    mut network_manager: ResMut<NetworkManager>,
    time: Res<Time>,
) {
    multiplayer_state.sync_interval.tick(time.delta());

    // Process incoming network messages
    if let Some(receiver) = &mut network_manager.message_receiver {
        while let Ok(message) = receiver.try_recv() {
            process_network_message(&mut multiplayer_state, &message);
        }
    }

    // Auto-start game when all players ready
    if !multiplayer_state.game_started
        && multiplayer_state.connected_players.len() >= 2
        && multiplayer_state
            .connected_players
            .values()
            .all(|p| p.ready)
    {
        start_multiplayer_game(&mut multiplayer_state, &mut network_manager);
    }
}

fn process_network_message(multiplayer_state: &mut MultiplayerState, message: &NetworkMessage) {
    match message {
        NetworkMessage::PlayerJoin { player_info } => {
            multiplayer_state
                .connected_players
                .insert(player_info.user_id, player_info.clone());
            assign_player_role(multiplayer_state, player_info.user_id);
        }

        NetworkMessage::PlayerLeave { player_id } => {
            multiplayer_state.connected_players.remove(player_id);
            multiplayer_state.player_assignments.remove(player_id);
        }

        NetworkMessage::PlayerReady { player_id, ready } => {
            if let Some(player) = multiplayer_state.connected_players.get_mut(player_id) {
                player.ready = *ready;
            }
        }

        NetworkMessage::AuthResponse { success, player_id } => {
            if *success {
                multiplayer_state.connection_status = ConnectionStatus::Connected;
            } else {
                multiplayer_state.connection_status =
                    ConnectionStatus::Error("Authentication failed".to_string());
            }
        }

        _ => {} // Handle other message types in respective systems
    }
}

fn assign_player_role(multiplayer_state: &mut MultiplayerState, player_id: Uuid) {
    let existing_roles: Vec<PlayerRole> = multiplayer_state
        .player_assignments
        .values()
        .cloned()
        .collect();

    let available_roles = match multiplayer_state.game_mode {
        MultiplayerGameMode::Asymmetric => {
            vec![
                PlayerRole::CartelCommander,
                PlayerRole::MilitaryCommander,
                PlayerRole::GovernmentAdvisor,
                PlayerRole::IntelligenceOfficer,
            ]
        }
        MultiplayerGameMode::Historical => {
            vec![
                PlayerRole::GovernmentAdvisor,
                PlayerRole::MilitaryCommander,
                PlayerRole::IntelligenceOfficer,
                PlayerRole::Observer,
            ]
        }
        MultiplayerGameMode::Cooperative => {
            vec![
                PlayerRole::CartelCommander,
                PlayerRole::CartelCommander, // Multiple cartel commanders
                PlayerRole::IntelligenceOfficer,
                PlayerRole::Observer,
            ]
        }
        MultiplayerGameMode::Competitive => {
            vec![
                PlayerRole::CartelCommander,
                PlayerRole::CartelCommander,
                PlayerRole::CartelCommander,
                PlayerRole::CartelCommander,
            ]
        }
    };

    // Assign first available role
    for role in available_roles {
        if !existing_roles.contains(&role)
            || matches!(role, PlayerRole::CartelCommander | PlayerRole::Observer)
        {
            multiplayer_state.player_assignments.insert(player_id, role);
            break;
        }
    }
}

fn start_multiplayer_game(
    multiplayer_state: &mut MultiplayerState,
    network_manager: &mut NetworkManager,
) {
    multiplayer_state.game_started = true;

    // Send game start message to all players
    if let Some(sender) = &network_manager.message_sender {
        let _ = sender.send(NetworkMessage::GameStart {
            scenario: multiplayer_state.scenario.clone(),
        });
    }
}

// ==================== PLAYER CONNECTION SYSTEM ====================

pub fn player_connection_system(
    mut multiplayer_state: ResMut<MultiplayerState>,
    mut network_manager: ResMut<NetworkManager>,
    time: Res<Time>,
) {
    // Monitor connection health
    let mut disconnected_players = Vec::new();

    for (player_id, player_info) in &mut multiplayer_state.connected_players {
        // Simulate ping monitoring (would be real network latency in production)
        player_info.ping = calculate_player_ping(*player_id);

        // Mark players as timed out if ping is too high
        if player_info.ping > 5000 {
            // 5 second timeout
            player_info.connection_status = PlayerConnectionStatus::TimedOut;
            disconnected_players.push(*player_id);
        }
    }

    // Remove timed out players
    for player_id in disconnected_players {
        multiplayer_state.connected_players.remove(&player_id);
        multiplayer_state.player_assignments.remove(&player_id);
    }
}

fn calculate_player_ping(player_id: Uuid) -> u32 {
    // Simplified ping calculation - in real implementation would measure actual network latency
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    player_id.hash(&mut hasher);
    let hash = hasher.finish();

    // Simulate realistic ping values (20-300ms)
    20 + ((hash % 280) as u32)
}

// ==================== GAME SYNC SYSTEM ====================

pub fn game_sync_system(
    mut multiplayer_state: ResMut<MultiplayerState>,
    network_manager: Res<NetworkManager>,
    game_state: Res<GameState>,
    political_state: Option<Res<crate::political_system::PoliticalState>>,
    unit_query: Query<(Entity, &Transform, &Unit)>,
    time: Res<Time>,
) {
    if !multiplayer_state.sync_interval.finished() {
        return;
    }

    if multiplayer_state.is_host && multiplayer_state.game_started {
        // Collect game state data
        let mut unit_positions = HashMap::new();
        let mut unit_health = HashMap::new();

        for (entity, transform, unit) in unit_query.iter() {
            unit_positions.insert(entity, transform.translation);
            unit_health.insert(entity, unit.health);
        }

        let sync_data = GameStateSyncData {
            timestamp: time.elapsed_seconds_f64(),
            unit_positions,
            unit_health,
            political_state: political_state.map(|ps| ps.clone()),
            game_phase: game_state.game_phase.clone(),
            resources: HashMap::new(), // Would include faction resources
        };

        // Send sync message to all clients
        if let Some(sender) = &network_manager.message_sender {
            let _ = sender.send(NetworkMessage::GameStateSync {
                game_state: sync_data,
            });
        }
    }
}

// ==================== PLAYER INPUT SYNC SYSTEM ====================

pub fn player_input_sync_system(
    multiplayer_state: Res<MultiplayerState>,
    network_manager: Res<NetworkManager>,
    mut unit_query: Query<&mut Unit>,
    keys: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if !multiplayer_state.game_started {
        return;
    }

    // Capture player input and send to other players
    if keys.just_pressed(KeyCode::Space) && multiplayer_state.is_host {
        // Example: Host can make political decisions
        let decision = PoliticalDecision {
            decision_type: PoliticalDecisionType::EscalateForce,
            parameters: HashMap::new(),
        };

        if let Some(sender) = &network_manager.message_sender {
            let _ = sender.send(NetworkMessage::PoliticalDecision {
                player_id: network_manager.player_id,
                decision,
            });
        }
    }
}

// ==================== MULTIPLAYER UI SYSTEM ====================

pub fn multiplayer_ui_system(
    mut commands: Commands,
    multiplayer_state: Res<MultiplayerState>,
    existing_ui: Query<Entity, With<MultiplayerUIPanel>>,
) {
    // Remove existing multiplayer UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Create multiplayer status panel
    if multiplayer_state.connected_players.len() > 1
        || !matches!(
            multiplayer_state.connection_status,
            ConnectionStatus::Disconnected
        )
    {
        spawn_multiplayer_ui_panel(&mut commands, &multiplayer_state);
    }
}

#[derive(Component)]
pub struct MultiplayerUIPanel;

fn spawn_multiplayer_ui_panel(commands: &mut Commands, multiplayer_state: &MultiplayerState) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    width: Val::Px(300.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
                ..default()
            },
            MultiplayerUIPanel,
        ))
        .with_children(|parent| {
            // Multiplayer status title
            parent.spawn(TextBundle::from_section(
                "🌐 MULTIPLAYER",
                TextStyle {
                    font_size: 16.0,
                    color: Color::CYAN,
                    ..default()
                },
            ));

            // Connection status
            let status_color = match multiplayer_state.connection_status {
                ConnectionStatus::Connected | ConnectionStatus::Hosting => Color::GREEN,
                ConnectionStatus::Connecting => Color::YELLOW,
                ConnectionStatus::Error(_) => Color::RED,
                ConnectionStatus::Disconnected => Color::GRAY,
            };

            let status_text = match &multiplayer_state.connection_status {
                ConnectionStatus::Disconnected => "Offline",
                ConnectionStatus::Connecting => "Connecting...",
                ConnectionStatus::Connected => "Connected",
                ConnectionStatus::Hosting => "Hosting",
                ConnectionStatus::Error(msg) => msg,
            };

            parent.spawn(TextBundle::from_section(
                format!("Status: {}", status_text),
                TextStyle {
                    font_size: 12.0,
                    color: status_color,
                    ..default()
                },
            ));

            // Game mode
            let mode_text = match multiplayer_state.game_mode {
                MultiplayerGameMode::Asymmetric => "Asymmetric (2v2)",
                MultiplayerGameMode::Historical => "Historical",
                MultiplayerGameMode::Cooperative => "Cooperative",
                MultiplayerGameMode::Competitive => "Competitive",
            };

            parent.spawn(TextBundle::from_section(
                format!("Mode: {}", mode_text),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // Connected players
            parent.spawn(TextBundle::from_section(
                format!(
                    "Players: {}/{}",
                    multiplayer_state.connected_players.len(),
                    multiplayer_state.max_players
                ),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // Player list
            for (player_id, player_info) in &multiplayer_state.connected_players {
                let role = multiplayer_state
                    .player_assignments
                    .get(player_id)
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_else(|| "Unassigned".to_string());

                let ping_color = if player_info.ping < 100 {
                    Color::GREEN
                } else if player_info.ping < 300 {
                    Color::YELLOW
                } else {
                    Color::RED
                };

                let ready_indicator = if player_info.ready { "✓" } else { "○" };

                parent.spawn(TextBundle::from_section(
                    format!(
                        "{} {} ({}ms) - {}",
                        ready_indicator, player_info.username, player_info.ping, role
                    ),
                    TextStyle {
                        font_size: 10.0,
                        color: ping_color,
                        ..default()
                    },
                ));
            }

            // Session info
            if multiplayer_state.is_host {
                parent.spawn(TextBundle::from_section(
                    format!(
                        "Session: {}",
                        multiplayer_state.session_id.to_string()[..8].to_uppercase()
                    ),
                    TextStyle {
                        font_size: 9.0,
                        color: Color::GRAY,
                        ..default()
                    },
                ));
            }
        });
}

// ==================== AUTHENTICATION INTEGRATION ====================

pub fn authenticate_multiplayer_session(
    network_manager: &mut NetworkManager,
    user: &User,
) -> Result<(), String> {
    // Generate session token using existing auth system
    let token = format!("mp_{}_{}", user.id, Uuid::new_v4());
    network_manager.auth_token = Some(token.clone());
    network_manager.player_id = user.id;

    // Send authentication request
    if let Some(sender) = &network_manager.message_sender {
        sender
            .send(NetworkMessage::AuthRequest { token })
            .map_err(|e| format!("Failed to send auth request: {}", e))?;
    }

    Ok(())
}

// ==================== HISTORICAL ACCURACY FEATURES ====================

pub fn setup_historical_multiplayer_scenario() -> MultiplayerScenario {
    // Configure the exact historical scenario for educational multiplayer
    MultiplayerScenario::HistoricalOctober17
}

pub fn get_scenario_player_roles(scenario: &MultiplayerScenario) -> Vec<PlayerRole> {
    match scenario {
        MultiplayerScenario::HistoricalOctober17 => vec![
            PlayerRole::GovernmentAdvisor, // Player advises on political decisions
            PlayerRole::MilitaryCommander, // Controls military response
            PlayerRole::IntelligenceOfficer, // Manages intel operations
            PlayerRole::Observer,          // Educational observer role
        ],
        _ => vec![
            PlayerRole::CartelCommander,
            PlayerRole::MilitaryCommander,
            PlayerRole::GovernmentAdvisor,
            PlayerRole::Observer,
        ],
    }
}
