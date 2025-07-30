use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== POLITICAL SYSTEM PLUGIN ====================

pub struct PoliticalSystemPlugin;

impl Plugin for PoliticalSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PoliticalState>()
            .init_resource::<SocialMediaInfluence>()
            .add_systems(
                Update,
                (
                    political_pressure_system,
                    government_decision_system,
                    public_opinion_system,
                    media_coverage_system,
                    international_pressure_system,
                    political_ui_system,
                )
                    .run_if(not_in_menu_phase),
            );
    }
}

// ==================== POLITICAL STATE RESOURCE ====================

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct PoliticalState {
    pub government_stability: f32,      // 0.0 to 1.0
    pub public_support_cartel: f32,     // 0.0 to 1.0
    pub public_support_government: f32, // 0.0 to 1.0
    pub international_pressure: f32,    // 0.0 to 1.0
    pub media_attention: f32,           // 0.0 to 1.0
    pub political_will: f32,            // Government's will to continue operation
    pub casualties_civilian: u32,
    pub casualties_military: u32,
    pub casualties_cartel: u32,
    pub infrastructure_damage: f32, // Economic impact
    pub operation_duration: f32,    // Time elapsed in seconds
    pub decision_threshold: f32,    // Threshold for government capitulation
    pub active_politicians: Vec<Politician>,
    pub recent_events: Vec<PoliticalEvent>,
    pub government_response_level: GovernmentResponseLevel,
}

impl Default for PoliticalState {
    fn default() -> Self {
        Self {
            government_stability: 0.7,
            public_support_cartel: 0.3,
            public_support_government: 0.6,
            international_pressure: 0.2,
            media_attention: 0.1,
            political_will: 0.8,
            casualties_civilian: 0,
            casualties_military: 0,
            casualties_cartel: 0,
            infrastructure_damage: 0.0,
            operation_duration: 0.0,
            decision_threshold: 0.3,
            active_politicians: vec![
                Politician {
                    name: "President López Obrador".to_string(),
                    position: PoliticalPosition::President,
                    influence: 0.9,
                    support_for_operation: 0.8,
                    pressure_received: 0.0,
                },
                Politician {
                    name: "Defense Minister".to_string(),
                    position: PoliticalPosition::DefenseMinister,
                    influence: 0.7,
                    support_for_operation: 0.9,
                    pressure_received: 0.0,
                },
                Politician {
                    name: "Interior Minister".to_string(),
                    position: PoliticalPosition::InteriorMinister,
                    influence: 0.6,
                    support_for_operation: 0.7,
                    pressure_received: 0.0,
                },
                Politician {
                    name: "Sinaloa Governor".to_string(),
                    position: PoliticalPosition::StateGovernor,
                    influence: 0.5,
                    support_for_operation: 0.4,
                    pressure_received: 0.0,
                },
            ],
            recent_events: Vec::new(),
            government_response_level: GovernmentResponseLevel::Limited,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Politician {
    pub name: String,
    pub position: PoliticalPosition,
    pub influence: f32,
    pub support_for_operation: f32,
    pub pressure_received: f32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum PoliticalPosition {
    President,
    DefenseMinister,
    InteriorMinister,
    StateGovernor,
    LocalMayor,
    Opposition,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PoliticalEvent {
    pub event_type: EventType,
    pub timestamp: f32,
    pub impact_score: f32,
    pub description: String,
    pub media_coverage: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EventType {
    CivilianCasualty,
    MilitaryCasualty,
    CartelSurrender,
    InfrastructureDamage,
    PublicProtest,
    InternationalCriticism,
    MediaExposure,
    PoliticalStatement,
    OperationEscalation,
    Ceasefire,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum GovernmentResponseLevel {
    Limited,    // Minimal force, quick withdrawal
    Moderate,   // Standard military response
    Aggressive, // Full military engagement
    AllOut,     // No retreat, complete operation
}

// ==================== SOCIAL MEDIA INFLUENCE RESOURCE ====================

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct SocialMediaInfluence {
    pub twitter_sentiment: f32,   // -1.0 to 1.0
    pub facebook_engagement: f32, // 0.0 to 1.0
    pub viral_videos: Vec<ViralContent>,
    pub hashtag_trends: HashMap<String, f32>,
    pub international_coverage: f32, // 0.0 to 1.0
    pub journalist_presence: u32,
}

impl Default for SocialMediaInfluence {
    fn default() -> Self {
        let mut hashtag_trends = HashMap::new();
        hashtag_trends.insert("#Culiacan".to_string(), 0.1);
        hashtag_trends.insert("#OvidioGuzman".to_string(), 0.05);
        hashtag_trends.insert("#Mexico".to_string(), 0.03);

        Self {
            twitter_sentiment: -0.2,
            facebook_engagement: 0.1,
            viral_videos: Vec::new(),
            hashtag_trends,
            international_coverage: 0.05,
            journalist_presence: 3,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ViralContent {
    pub content_type: ContentType,
    pub reach: u32,
    pub sentiment: f32,
    pub timestamp: f32,
    pub impact_multiplier: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ContentType {
    CombatFootage,
    CivilianFleeing,
    PropertyDamage,
    PoliticalSpeech,
    ProtestFootage,
    CartelPropaganda,
}

// ==================== POLITICAL PRESSURE SYSTEM ====================

pub fn political_pressure_system(
    time: Res<Time>,
    mut political_state: ResMut<PoliticalState>,
    mut social_media: ResMut<SocialMediaInfluence>,
    game_state: Res<GameState>,
    unit_query: Query<&Unit>,
) {
    let dt = time.delta_seconds();
    political_state.operation_duration += dt;
    let mut rng = rand::thread_rng();

    // Calculate current situation metrics
    let cartel_units = unit_query
        .iter()
        .filter(|u| matches!(u.faction, Faction::Cartel))
        .count() as f32;
    let military_units = unit_query
        .iter()
        .filter(|u| matches!(u.faction, Faction::Military))
        .count() as f32;

    let intensity_factor = (cartel_units + military_units) / 50.0;
    let duration_pressure = (political_state.operation_duration / 3600.0).min(2.0); // Max 2 hours

    // Update political will based on various factors
    let casualty_pressure = (political_state.casualties_civilian as f32 * 0.05)
        + (political_state.casualties_military as f32 * 0.03);

    let media_pressure = political_state.media_attention * 0.02;
    let duration_fatigue = duration_pressure * 0.01;
    let international_pressure_effect = political_state.international_pressure * 0.015;

    political_state.political_will -=
        (casualty_pressure + media_pressure + duration_fatigue + international_pressure_effect)
            * dt;
    political_state.political_will = political_state.political_will.max(0.0);

    // Update government stability
    let stability_factors = (1.0 - political_state.public_support_government) * 0.5
        + political_state.infrastructure_damage * 0.3
        + intensity_factor * 0.2;

    political_state.government_stability -= stability_factors * dt * 0.1;
    political_state.government_stability = political_state.government_stability.clamp(0.0, 1.0);

    // Update media attention based on combat intensity
    let media_growth =
        intensity_factor * 0.02 + (political_state.casualties_civilian as f32 * 0.001);
    political_state.media_attention += media_growth * dt;
    political_state.media_attention = political_state.media_attention.clamp(0.0, 1.0);

    // Social media viral content generation
    if rng.gen::<f32>() < intensity_factor * dt * 0.1 {
        generate_viral_content(&mut social_media, &political_state, &mut rng);
    }

    // Update hashtag trends
    update_hashtag_trends(&mut social_media, &political_state, dt);

    // Update politician pressure levels
    for politician in &mut political_state.active_politicians {
        let position_multiplier = match politician.position {
            PoliticalPosition::President => 1.0,
            PoliticalPosition::DefenseMinister => 0.8,
            PoliticalPosition::InteriorMinister => 0.6,
            PoliticalPosition::StateGovernor => 0.9, // High local pressure
            _ => 0.4,
        };

        let pressure_increase =
            (media_pressure + international_pressure_effect) * position_multiplier * dt;
        politician.pressure_received += pressure_increase;

        // Pressure affects support for operation
        if politician.pressure_received > 0.5 {
            politician.support_for_operation -= 0.1 * dt;
            politician.support_for_operation = politician.support_for_operation.max(0.0);
        }
    }
}

fn generate_viral_content(
    social_media: &mut SocialMediaInfluence,
    political_state: &PoliticalState,
    rng: &mut rand::rngs::ThreadRng,
) {
    let content_types = [
        ContentType::CombatFootage,
        ContentType::CivilianFleeing,
        ContentType::PropertyDamage,
        ContentType::PoliticalSpeech,
        ContentType::ProtestFootage,
    ];

    let content_type = content_types[rng.gen_range(0..content_types.len())].clone();
    let reach = rng.gen_range(1000..100000);

    let sentiment = match content_type {
        ContentType::CombatFootage => rng.gen_range(-0.8..-0.2),
        ContentType::CivilianFleeing => rng.gen_range(-0.9..-0.5),
        ContentType::PropertyDamage => rng.gen_range(-0.7..-0.3),
        ContentType::PoliticalSpeech => rng.gen_range(-0.3..0.3),
        ContentType::ProtestFootage => rng.gen_range(-0.6..-0.1),
        ContentType::CartelPropaganda => rng.gen_range(0.2..0.6),
    };

    let viral_content = ViralContent {
        content_type,
        reach,
        sentiment,
        timestamp: political_state.operation_duration,
        impact_multiplier: rng.gen_range(1.0..3.0),
    };

    social_media.viral_videos.push(viral_content);

    // Limit viral content history
    if social_media.viral_videos.len() > 10 {
        social_media.viral_videos.remove(0);
    }
}

fn update_hashtag_trends(
    social_media: &mut SocialMediaInfluence,
    political_state: &PoliticalState,
    dt: f32,
) {
    let base_growth = political_state.media_attention * dt * 0.1;

    for (hashtag, trend_value) in social_media.hashtag_trends.iter_mut() {
        match hashtag.as_str() {
            "#Culiacan" => *trend_value += base_growth * 2.0,
            "#OvidioGuzman" => *trend_value += base_growth * 1.5,
            "#Mexico" => *trend_value += base_growth * 0.8,
            _ => *trend_value += base_growth * 0.5,
        }

        *trend_value = trend_value.clamp(0.0, 1.0);
    }

    // Add new trending hashtags based on events
    if political_state.casualties_civilian > 5 {
        social_media.hashtag_trends.insert(
            "#CivilianCasualties".to_string(),
            political_state.casualties_civilian as f32 * 0.02,
        );
    }

    if political_state.operation_duration > 7200.0 {
        // 2 hours
        social_media
            .hashtag_trends
            .insert("#EndTheViolence".to_string(), 0.3);
    }
}

// ==================== GOVERNMENT DECISION SYSTEM ====================

pub fn government_decision_system(
    mut political_state: ResMut<PoliticalState>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    // Calculate weighted decision factors
    let president = political_state
        .active_politicians
        .iter()
        .find(|p| p.position == PoliticalPosition::President)
        .cloned()
        .unwrap_or_else(|| political_state.active_politicians[0].clone());

    let decision_pressure = (1.0 - political_state.political_will) * 0.4
        + (1.0 - political_state.government_stability) * 0.3
        + (1.0 - president.support_for_operation) * 0.3;

    // Check for government capitulation
    if decision_pressure > political_state.decision_threshold {
        // Historical accuracy: Government decided to release Ovidio
        if !matches!(game_state.game_phase, GamePhase::Victory)
            && !matches!(game_state.game_phase, GamePhase::Defeat)
        {
            // Add historical decision event
            let event = PoliticalEvent {
                event_type: EventType::PoliticalStatement,
                timestamp: time.elapsed_seconds(),
                impact_score: 1.0,
                description: "Government orders cessation of operation and release of target"
                    .to_string(),
                media_coverage: 1.0,
            };

            political_state.recent_events.push(event);

            // Trigger victory condition (historically accurate outcome)
            game_state.game_phase = GamePhase::Victory;
        }
    }

    // Update government response level based on pressure and duration
    political_state.government_response_level = match decision_pressure {
        p if p < 0.2 => GovernmentResponseLevel::AllOut,
        p if p < 0.4 => GovernmentResponseLevel::Aggressive,
        p if p < 0.6 => GovernmentResponseLevel::Moderate,
        _ => GovernmentResponseLevel::Limited,
    };
}

// ==================== PUBLIC OPINION SYSTEM ====================

pub fn public_opinion_system(
    mut political_state: ResMut<PoliticalState>,
    social_media: Res<SocialMediaInfluence>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    // Social media influence on public opinion
    let social_impact = social_media
        .viral_videos
        .iter()
        .map(|v| v.sentiment * (v.reach as f32 / 100000.0))
        .sum::<f32>()
        * 0.1;

    let twitter_influence = social_media.twitter_sentiment * 0.05;
    let media_exposure_effect = political_state.media_attention * 0.03;

    // Casualties heavily influence public opinion
    let casualty_impact = (political_state.casualties_civilian as f32 * 0.02)
        + (political_state.casualties_military as f32 * 0.01);

    // Update public support
    political_state.public_support_government +=
        (social_impact + twitter_influence - casualty_impact - media_exposure_effect) * dt;
    political_state.public_support_government =
        political_state.public_support_government.clamp(0.0, 1.0);

    // Cartel support often inversely related but with different dynamics
    political_state.public_support_cartel +=
        (casualty_impact * 0.5 - social_impact * 0.3 + media_exposure_effect * 0.2) * dt;
    political_state.public_support_cartel = political_state.public_support_cartel.clamp(0.0, 1.0);
}

// ==================== MEDIA COVERAGE SYSTEM ====================

pub fn media_coverage_system(
    mut political_state: ResMut<PoliticalState>,
    mut social_media: ResMut<SocialMediaInfluence>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let mut rng = rand::thread_rng();

    // Media coverage increases with dramatic events
    let coverage_factors = political_state.infrastructure_damage * 0.2
        + (political_state.casualties_civilian as f32 * 0.05)
        + political_state.operation_duration / 7200.0; // 2-hour operation

    political_state.media_attention += coverage_factors * dt * 0.1;
    political_state.media_attention = political_state.media_attention.clamp(0.0, 1.0);

    // International media coverage
    if political_state.media_attention > 0.7 && rng.gen::<f32>() < dt * 0.1 {
        social_media.international_coverage += 0.1;
        social_media.journalist_presence += 1;

        // International coverage increases pressure
        political_state.international_pressure += 0.05;
    }

    social_media.international_coverage = social_media.international_coverage.clamp(0.0, 1.0);
    political_state.international_pressure = political_state.international_pressure.clamp(0.0, 1.0);
}

// ==================== INTERNATIONAL PRESSURE SYSTEM ====================

pub fn international_pressure_system(
    mut political_state: ResMut<PoliticalState>,
    social_media: Res<SocialMediaInfluence>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    // International attention increases pressure
    let international_factors = social_media.international_coverage * 0.5
        + political_state.media_attention * 0.3
        + (political_state.casualties_civilian as f32 * 0.02);

    political_state.international_pressure += international_factors * dt * 0.05;
    political_state.international_pressure = political_state.international_pressure.clamp(0.0, 1.0);

    // Generate international pressure events
    if political_state.international_pressure > 0.6 && rand::thread_rng().gen::<f32>() < dt * 0.05 {
        let event = PoliticalEvent {
            event_type: EventType::InternationalCriticism,
            timestamp: time.elapsed_seconds(),
            impact_score: 0.7,
            description: "International human rights organizations express concern".to_string(),
            media_coverage: 0.8,
        };

        political_state.recent_events.push(event);

        // Limit event history
        if political_state.recent_events.len() > 20 {
            political_state.recent_events.remove(0);
        }
    }
}

// ==================== POLITICAL UI SYSTEM ====================

pub fn political_ui_system(
    mut commands: Commands,
    political_state: Res<PoliticalState>,
    social_media: Res<SocialMediaInfluence>,
    existing_ui: Query<Entity, With<PoliticalUIPanel>>,
) {
    // Remove existing political UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Create political status panel
    spawn_political_ui_panel(&mut commands, &political_state, &social_media);
}

#[derive(Component)]
pub struct PoliticalUIPanel;

fn spawn_political_ui_panel(
    commands: &mut Commands,
    political_state: &PoliticalState,
    social_media: &SocialMediaInfluence,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    width: Val::Px(280.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.85)),
                ..default()
            },
            PoliticalUIPanel,
        ))
        .with_children(|parent| {
            // Political status title
            parent.spawn(TextBundle::from_section(
                "🏛️ POLITICAL STATUS",
                TextStyle {
                    font_size: 16.0,
                    color: Color::GOLD,
                    ..default()
                },
            ));

            // Government stability
            let stability_color = if political_state.government_stability > 0.7 {
                Color::GREEN
            } else if political_state.government_stability > 0.4 {
                Color::YELLOW
            } else {
                Color::RED
            };

            parent.spawn(TextBundle::from_section(
                format!(
                    "Stability: {:.1}%",
                    political_state.government_stability * 100.0
                ),
                TextStyle {
                    font_size: 12.0,
                    color: stability_color,
                    ..default()
                },
            ));

            // Political will
            let will_color = if political_state.political_will > 0.6 {
                Color::GREEN
            } else if political_state.political_will > 0.3 {
                Color::YELLOW
            } else {
                Color::RED
            };

            parent.spawn(TextBundle::from_section(
                format!(
                    "Political Will: {:.1}%",
                    political_state.political_will * 100.0
                ),
                TextStyle {
                    font_size: 12.0,
                    color: will_color,
                    ..default()
                },
            ));

            // Public support
            parent.spawn(TextBundle::from_section(
                format!(
                    "Public Support: {:.1}%",
                    political_state.public_support_government * 100.0
                ),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // Media attention
            let media_color = if political_state.media_attention > 0.7 {
                Color::RED
            } else if political_state.media_attention > 0.4 {
                Color::ORANGE
            } else {
                Color::WHITE
            };

            parent.spawn(TextBundle::from_section(
                format!(
                    "Media Attention: {:.1}%",
                    political_state.media_attention * 100.0
                ),
                TextStyle {
                    font_size: 12.0,
                    color: media_color,
                    ..default()
                },
            ));

            // International pressure
            parent.spawn(TextBundle::from_section(
                format!(
                    "Intl. Pressure: {:.1}%",
                    political_state.international_pressure * 100.0
                ),
                TextStyle {
                    font_size: 12.0,
                    color: Color::ORANGE,
                    ..default()
                },
            ));

            // Casualties
            if political_state.casualties_civilian > 0 || political_state.casualties_military > 0 {
                parent.spawn(TextBundle::from_section(
                    format!(
                        "Casualties: {}C {}M",
                        political_state.casualties_civilian, political_state.casualties_military
                    ),
                    TextStyle {
                        font_size: 12.0,
                        color: Color::RED,
                        ..default()
                    },
                ));
            }

            // Operation duration
            let hours = (political_state.operation_duration / 3600.0) as u32;
            let minutes = ((political_state.operation_duration % 3600.0) / 60.0) as u32;
            parent.spawn(TextBundle::from_section(
                format!("Duration: {}h {}m", hours, minutes),
                TextStyle {
                    font_size: 12.0,
                    color: Color::GRAY,
                    ..default()
                },
            ));

            // Top trending hashtag
            if let Some((hashtag, trend_value)) = social_media
                .hashtag_trends
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                parent.spawn(TextBundle::from_section(
                    format!("Trending: {} ({:.1}%)", hashtag, trend_value * 100.0),
                    TextStyle {
                        font_size: 10.0,
                        color: Color::CYAN,
                        ..default()
                    },
                ));
            }

            // Recent important events
            if let Some(event) = political_state.recent_events.last() {
                parent.spawn(TextBundle::from_section(
                    "📰 LATEST:",
                    TextStyle {
                        font_size: 11.0,
                        color: Color::YELLOW,
                        ..default()
                    },
                ));

                parent.spawn(TextBundle::from_section(
                    format!("• {}", event.description),
                    TextStyle {
                        font_size: 9.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ));
            }
        });
}
