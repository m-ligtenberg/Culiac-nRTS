use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;

// ==================== INTEL SYSTEM SETUP ====================

pub struct IntelSystemPlugin;

impl Plugin for IntelSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<IntelSystem>()
            .add_systems(Update, (
                radio_intercept_system,
                informant_network_system,
                reconnaissance_system,
                counter_intel_system,
                intel_ui_system,
                process_intel_reports,
            ).run_if(not_in_menu_phase));
    }
}

// ==================== RADIO INTERCEPT SYSTEM ====================

pub fn radio_intercept_system(
    time: Res<Time>,
    mut intel_system: ResMut<IntelSystem>,
    mut intel_operators: Query<&mut IntelOperator>,
    military_units: Query<(&Transform, &Unit), (With<Unit>, Without<IntelOperator>)>,
) {
    let mut rng = rand::thread_rng();
    
    // Process radio intercept operators
    for mut operator in intel_operators.iter_mut() {
        if operator.intel_type == IntelType::RadioIntercept {
            operator.intel_cooldown.tick(time.delta());
            
            if operator.intel_cooldown.finished() {
                // Reset cooldown (5-15 seconds between intercepts)
                operator.intel_cooldown = Timer::from_seconds(
                    rng.gen_range(5.0..15.0), 
                    TimerMode::Once
                );
                
                // Attempt to intercept military communications
                let intercept_roll = rng.gen::<f32>();
                let jamming_penalty = if intel_system.jamming_active {
                    intel_system.jamming_strength * 0.5
                } else {
                    0.0
                };
                
                if intercept_roll < (intel_system.intercept_chance - jamming_penalty) {
                    // Generate realistic radio intercept
                    if let Some(intercept) = generate_radio_intercept(&military_units, &mut rng, time.elapsed_seconds()) {
                        intel_system.global_intel_network.active_intercepts.push(intercept);
                        
                        // Limit intercept history to prevent memory bloat
                        if intel_system.global_intel_network.active_intercepts.len() > 20 {
                            intel_system.global_intel_network.active_intercepts.remove(0);
                        }
                    }
                }
            }
        }
    }
}

fn generate_radio_intercept(
    military_units: &Query<(&Transform, &Unit), (With<Unit>, Without<IntelOperator>)>,
    rng: &mut rand::rngs::ThreadRng,
    current_time: f32,
) -> Option<RadioIntercept> {
    if military_units.is_empty() {
        return None;
    }
    
    // Pick a random military unit as message source
    let units: Vec<_> = military_units.iter().collect();
    let (transform, _unit) = units[rng.gen_range(0..units.len())];
    
    let message_types = [
        RadioMessageType::TroopMovement(transform.translation, rng.gen_range(2..8)),
        RadioMessageType::AirSupport(transform.translation + Vec3::new(
            rng.gen_range(-100.0..100.0), 
            0.0, 
            rng.gen_range(-100.0..100.0)
        )),
        RadioMessageType::Reinforcements(transform.translation, rng.gen_range(30.0..120.0)),
        RadioMessageType::StatusUpdate("Sector clear, continuing patrol".to_string()),
        RadioMessageType::SupplyDrop(transform.translation + Vec3::new(
            rng.gen_range(-200.0..200.0), 
            0.0, 
            rng.gen_range(-200.0..200.0)
        )),
    ];
    
    let message_type = message_types[rng.gen_range(0..message_types.len())].clone();
    let content = format_radio_message(&message_type);
    
    Some(RadioIntercept {
        message_type,
        source_position: transform.translation,
        intercept_time: current_time,
        reliability: rng.gen_range(0.6..0.95),
        content,
    })
}

fn format_radio_message(message_type: &RadioMessageType) -> String {
    match message_type {
        RadioMessageType::TroopMovement(pos, count) => {
            format!("Alpha team moving {} units to grid {:.0},{:.0}", count, pos.x, pos.z)
        },
        RadioMessageType::AirSupport(pos) => {
            format!("Requesting air support at coordinates {:.0},{:.0}", pos.x, pos.z)
        },
        RadioMessageType::Reinforcements(pos, eta) => {
            format!("Reinforcements ETA {:.0} minutes to grid {:.0},{:.0}", eta / 60.0, pos.x, pos.z)
        },
        RadioMessageType::StatusUpdate(msg) => msg.clone(),
        RadioMessageType::SupplyDrop(pos) => {
            format!("Supply drop scheduled at LZ {:.0},{:.0}", pos.x, pos.z)
        },
        RadioMessageType::Retreat(pos) => {
            format!("Falling back to rally point {:.0},{:.0}", pos.x, pos.z)
        },
    }
}

// ==================== INFORMANT NETWORK SYSTEM ====================

pub fn informant_network_system(
    time: Res<Time>,
    mut intel_system: ResMut<IntelSystem>,
    mut intel_operators: Query<&mut IntelOperator>,
    military_units: Query<(&Transform, &Unit), With<Unit>>,
) {
    let mut rng = rand::thread_rng();
    
    for mut operator in intel_operators.iter_mut() {
        if operator.intel_type == IntelType::Informant {
            operator.intel_cooldown.tick(time.delta());
            
            if operator.intel_cooldown.finished() {
                // Reset cooldown (20-60 seconds between tips)
                operator.intel_cooldown = Timer::from_seconds(
                    rng.gen_range(20.0..60.0), 
                    TimerMode::Once
                );
                
                // Generate informant tip
                if rng.gen::<f32>() < 0.4 { // 40% chance per check
                    if let Some(tip) = generate_informant_tip(&military_units, &mut rng, time.elapsed_seconds()) {
                        intel_system.global_intel_network.informant_reports.push(tip);
                        
                        // Limit tip history
                        if intel_system.global_intel_network.informant_reports.len() > 15 {
                            intel_system.global_intel_network.informant_reports.remove(0);
                        }
                    }
                }
            }
        }
    }
}

fn generate_informant_tip(
    military_units: &Query<(&Transform, &Unit), With<Unit>>,
    rng: &mut rand::rngs::ThreadRng,
    current_time: f32,
) -> Option<InformantTip> {
    if military_units.is_empty() {
        return None;
    }
    
    let units: Vec<_> = military_units.iter().collect();
    let (transform, unit) = units[rng.gen_range(0..units.len())];
    
    let tip_types = [
        TipType::EnemyPosition(unit.unit_type.clone(), rng.gen_range(1..5)),
        TipType::PlannedAttack(transform.translation, rng.gen_range(60.0..300.0)),
        TipType::WeakPoint(transform.translation + Vec3::new(
            rng.gen_range(-50.0..50.0), 
            0.0, 
            rng.gen_range(-50.0..50.0)
        )),
        TipType::CommandPost(transform.translation),
    ];
    
    let tip_type = tip_types[rng.gen_range(0..tip_types.len())].clone();
    let urgency = match rng.gen_range(0..4) {
        0 => TipUrgency::Low,
        1 => TipUrgency::Medium,
        2 => TipUrgency::High,
        _ => TipUrgency::Critical,
    };
    
    Some(InformantTip {
        tip_type,
        location: transform.translation,
        confidence: rng.gen_range(0.5..0.9),
        time_received: current_time,
        urgency,
    })
}

// ==================== RECONNAISSANCE SYSTEM ====================

pub fn reconnaissance_system(
    time: Res<Time>,
    mut intel_system: ResMut<IntelSystem>,
    mut intel_operators: Query<(&Transform, &mut IntelOperator)>,
    enemy_units: Query<(&Transform, &Unit), (With<Unit>, Without<IntelOperator>)>,
) {
    for (operator_transform, mut operator) in intel_operators.iter_mut() {
        if operator.intel_type == IntelType::Reconnaissance {
            operator.intel_cooldown.tick(time.delta());
            
            if operator.intel_cooldown.finished() {
                operator.intel_cooldown = Timer::from_seconds(15.0, TimerMode::Once);
                
                // Scan for enemies within detection range
                let mut enemies_spotted = Vec::new();
                
                for (enemy_transform, enemy_unit) in enemy_units.iter() {
                    let distance = operator_transform.translation.distance(enemy_transform.translation);
                    
                    if distance <= operator.detection_range {
                        enemies_spotted.push(EnemyContact {
                            position: enemy_transform.translation,
                            enemy_type: enemy_unit.unit_type.clone(),
                            confidence: calculate_detection_confidence(distance, operator.detection_range),
                            last_seen: time.elapsed_seconds(),
                        });
                    }
                }
                
                if !enemies_spotted.is_empty() {
                    let recon_report = ReconReport {
                        area_scanned: operator_transform.translation,
                        scan_radius: operator.detection_range,
                        enemies_spotted,
                        terrain_info: generate_terrain_intel(operator_transform.translation),
                        scan_time: time.elapsed_seconds(),
                    };
                    
                    intel_system.global_intel_network.reconnaissance_data.push(recon_report);
                    
                    // Limit recon history
                    if intel_system.global_intel_network.reconnaissance_data.len() > 25 {
                        intel_system.global_intel_network.reconnaissance_data.remove(0);
                    }
                }
            }
        }
    }
}

fn calculate_detection_confidence(distance: f32, max_range: f32) -> f32 {
    (1.0 - (distance / max_range)).max(0.3)
}

fn generate_terrain_intel(position: Vec3) -> TerrainIntel {
    let mut rng = rand::thread_rng();
    
    TerrainIntel {
        cover_points: (0..rng.gen_range(2..6))
            .map(|_| position + Vec3::new(
                rng.gen_range(-30.0..30.0), 
                0.0, 
                rng.gen_range(-30.0..30.0)
            ))
            .collect(),
        choke_points: (0..rng.gen_range(1..3))
            .map(|_| position + Vec3::new(
                rng.gen_range(-50.0..50.0), 
                0.0, 
                rng.gen_range(-50.0..50.0)
            ))
            .collect(),
        elevation_advantages: (0..rng.gen_range(1..4))
            .map(|_| position + Vec3::new(
                rng.gen_range(-40.0..40.0), 
                rng.gen_range(5.0..15.0), 
                rng.gen_range(-40.0..40.0)
            ))
            .collect(),
        escape_routes: (0..rng.gen_range(2..5))
            .map(|_| position + Vec3::new(
                rng.gen_range(-60.0..60.0), 
                0.0, 
                rng.gen_range(-60.0..60.0)
            ))
            .collect(),
    }
}

// ==================== COUNTER INTELLIGENCE SYSTEM ====================

pub fn counter_intel_system(
    time: Res<Time>,
    mut intel_system: ResMut<IntelSystem>,
    intel_operators: Query<(&Transform, &IntelOperator)>,
    military_units: Query<(&Transform, &Unit), With<Unit>>,
) {
    let mut rng = rand::thread_rng();
    
    // Military counter-intelligence tries to detect cartel intel operations
    for (military_transform, _military_unit) in military_units.iter() {
        for (intel_transform, intel_operator) in intel_operators.iter() {
            let distance = military_transform.translation.distance(intel_transform.translation);
            
            // Chance to detect based on distance and stealth level
            let detection_chance = (1.0 - intel_operator.stealth_level) * 
                                 intel_system.counter_intel_level * 
                                 (1.0 - (distance / 200.0)).max(0.0);
            
            if rng.gen::<f32>() < detection_chance * time.delta_seconds() * 0.1 {
                let alert = CounterIntelAlert {
                    threat_type: match intel_operator.intel_type {
                        IntelType::Reconnaissance => CounterIntelThreat::EnemyScout(intel_transform.translation.into()),
                        IntelType::RadioIntercept => CounterIntelThreat::RadioJamming(
                            military_transform.translation, 
                            100.0
                        ),
                        IntelType::Informant => CounterIntelThreat::InformantCompromised(intel_transform.translation),
                        IntelType::CounterIntel => CounterIntelThreat::SurveillanceDrone(intel_transform.translation.into()),
                    },
                    detected_position: intel_transform.translation,
                    alert_time: time.elapsed_seconds(),
                    threat_level: rng.gen_range(0.3..0.8),
                };
                
                intel_system.global_intel_network.counter_intel_alerts.push(alert);
                
                // Limit alert history
                if intel_system.global_intel_network.counter_intel_alerts.len() > 10 {
                    intel_system.global_intel_network.counter_intel_alerts.remove(0);
                }
                
                // Activate jamming if appropriate
                if matches!(intel_operator.intel_type, IntelType::RadioIntercept) {
                    intel_system.jamming_active = true;
                    intel_system.jamming_strength = rng.gen_range(0.3..0.8);
                }
            }
        }
    }
}

// ==================== INTEL PROCESSING SYSTEM ====================

pub fn process_intel_reports(
    mut commands: Commands,
    intel_system: Res<IntelSystem>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();
    
    // Process radio intercepts for actionable intelligence
    for intercept in &intel_system.global_intel_network.active_intercepts {
        match &intercept.message_type {
            RadioMessageType::AirSupport(position) => {
                // Spawn warning indicator for incoming airstrike
                spawn_intel_indicator(&mut commands, *position, "⚠️ AIR STRIKE INCOMING", Color::RED);
            },
            RadioMessageType::Reinforcements(position, eta) => {
                if *eta < 120.0 { // Less than 2 minutes
                    spawn_intel_indicator(&mut commands, *position, "🚁 REINFORCEMENTS NEAR", Color::ORANGE);
                }
            },
            _ => {}
        }
    }
    
    // Process high-urgency informant tips
    for tip in &intel_system.global_intel_network.informant_reports {
        if matches!(tip.urgency, TipUrgency::Critical | TipUrgency::High) && 
           current_time - tip.time_received < 60.0 { // Fresh intel within 1 minute
            match &tip.tip_type {
                TipType::PlannedAttack(position, eta) => {
                    if *eta < 90.0 {
                        spawn_intel_indicator(&mut commands, *position, "🎯 IMMINENT ATTACK", Color::RED);
                    }
                },
                TipType::CommandPost(position) => {
                    spawn_intel_indicator(&mut commands, *position, "🏢 HVT IDENTIFIED", Color::YELLOW);
                },
                _ => {}
            }
        }
    }
}

fn spawn_intel_indicator(
    commands: &mut Commands,
    position: Vec3,
    text: &str,
    color: Color,
) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 20.0,
                    color,
                    ..default()
                }
            ),
            transform: Transform::from_translation(position + Vec3::new(0.0, 30.0, 0.0)),
            ..default()
        },
        DamageIndicator {
            lifetime: Timer::from_seconds(5.0, TimerMode::Once),
        },
    ));
}

// ==================== INTEL UI SYSTEM ====================

pub fn intel_ui_system(
    mut commands: Commands,
    intel_system: Res<IntelSystem>,
    existing_ui: Query<Entity, With<IntelUIPanel>>,
) {
    // Remove existing intel UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Create intel panel
    let recent_intercepts = intel_system.global_intel_network.active_intercepts
        .iter()
        .rev()
        .take(3)
        .collect::<Vec<_>>();
    
    let recent_tips = intel_system.global_intel_network.informant_reports
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>();
    
    if !recent_intercepts.is_empty() || !recent_tips.is_empty() {
        spawn_intel_ui_panel(&mut commands, &recent_intercepts, &recent_tips);
    }
}

#[derive(Component)]
pub struct IntelUIPanel;

fn spawn_intel_ui_panel(
    commands: &mut Commands,
    intercepts: &[&RadioIntercept],
    tips: &[&InformantTip],
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(150.0),
                width: Val::Px(300.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
            ..default()
        },
        IntelUIPanel,
    )).with_children(|parent| {
        // Intel panel title
        parent.spawn(TextBundle::from_section(
            "📡 INTELLIGENCE",
            TextStyle {
                font_size: 16.0,
                color: Color::CYAN,
                ..default()
            }
        ));
        
        // Radio intercepts
        if !intercepts.is_empty() {
            parent.spawn(TextBundle::from_section(
                "📻 RADIO CHATTER:",
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                }
            ));
            
            for intercept in intercepts {
                let reliability_color = if intercept.reliability > 0.8 {
                    Color::GREEN
                } else if intercept.reliability > 0.6 {
                    Color::YELLOW
                } else {
                    Color::ORANGE
                };
                
                parent.spawn(TextBundle::from_section(
                    format!("• {}", intercept.content),
                    TextStyle {
                        font_size: 10.0,
                        color: reliability_color,
                        ..default()
                    }
                ));
            }
        }
        
        // Informant tips
        if !tips.is_empty() {
            parent.spawn(TextBundle::from_section(
                "👤 INFORMANTS:",
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                }
            ));
            
            for tip in tips {
                let urgency_color = match tip.urgency {
                    TipUrgency::Critical => Color::RED,
                    TipUrgency::High => Color::ORANGE,
                    TipUrgency::Medium => Color::YELLOW,
                    TipUrgency::Low => Color::WHITE,
                };
                
                let tip_text = match &tip.tip_type {
                    TipType::EnemyPosition(unit_type, count) => 
                        format!("• {} {:?} spotted", count, unit_type),
                    TipType::PlannedAttack(_, eta) => 
                        format!("• Attack planned in {:.0}s", eta),
                    TipType::WeakPoint(_) => 
                        "• Weak point identified".to_string(),
                    TipType::CommandPost(_) => 
                        "• Command post located".to_string(),
                    TipType::SupplyRoute(_, _) => 
                        "• Supply route discovered".to_string(),
                };
                
                parent.spawn(TextBundle::from_section(
                    tip_text,
                    TextStyle {
                        font_size: 10.0,
                        color: urgency_color,
                        ..default()
                    }
                ));
            }
        }
    });
}