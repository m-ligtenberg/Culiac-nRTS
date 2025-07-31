use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

// Type aliases to reduce complexity
type StatusTextQuery<'a> = Query<
    'a,
    'a,
    &'a mut Text,
    With<StatusText>,
>;

type WaveTextQuery<'a> = Query<'a, 'a, &'a mut Text, With<WaveText>>;

type ScoreTextQuery<'a> = Query<'a, 'a, &'a mut Text, With<ScoreText>>;

type DifficultyTextQuery<'a> = Query<'a, 'a, &'a mut Text, With<DifficultyDisplay>>;

type HealthBarQuery<'a> = Query<
    'a,
    'a,
    (Entity, &'a mut Transform, &'a mut Sprite, &'a HealthBar),
    (With<HealthBar>, Without<Unit>),
>;

// ==================== CORE UI UPDATE SYSTEMS ====================

pub fn ui_update_system(
    game_state: Res<GameState>,
    ai_director: Res<AiDirector>,
    unit_query: Query<&Unit, Changed<Unit>>,
    mut status_query: StatusTextQuery,
    mut wave_query: WaveTextQuery,
    mut score_query: ScoreTextQuery,
    mut difficulty_query: DifficultyTextQuery,
) {
    // Count units by faction
    let cartel_count = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Cartel && u.health > 0.0)
        .count();
    let military_count = unit_query
        .iter()
        .filter(|u| u.faction == Faction::Military && u.health > 0.0)
        .count();
    let ovidio_alive = unit_query
        .iter()
        .any(|u| u.unit_type == UnitType::Ovidio && u.health > 0.0);

    // Update status text
    if let Ok(mut text) = status_query.get_single_mut() {
        let status = if !ovidio_alive {
            "❌ MISSION FAILED: Ovidio captured!"
        } else if game_state.game_phase == GamePhase::GameOver {
            "✅ MISSION SUCCESS: Government retreats!"
        } else {
            match game_state.game_phase {
                GamePhase::MainMenu => "🎮 Main Menu",
                GamePhase::SaveMenu => "💾 Save Game",
                GamePhase::LoadMenu => "📂 Load Game",
                GamePhase::MissionBriefing => "📋 Mission Briefing",
                GamePhase::Preparation => "🔄 Phase: Preparation",
                GamePhase::InitialRaid => "⚔️ Phase: Initial Raid",
                GamePhase::BlockConvoy => "🚧 Phase: Block Convoy",
                GamePhase::ApplyPressure => "🔥 Phase: Apply Pressure",
                GamePhase::HoldTheLine => "🛡️ Phase: Hold The Line",
                GamePhase::Victory => "🏆 VICTORY!",
                GamePhase::Defeat => "💀 DEFEAT!",
                GamePhase::GameOver => "🏁 Mission Complete",
            }
        };
        text.sections[0].value = format!(
            "{}\nCartel: {} | Military: {}",
            status, cartel_count, military_count
        );
    } else {
        warn!("StatusText UI element not found");
    }

    // Update wave text
    if let Ok(mut text) = wave_query.get_single_mut() {
        text.sections[0].value = format!(
            "Wave: {} - Timer: {:.1}s",
            game_state.current_wave, game_state.mission_timer
        );
    }

    // Update score text
    if let Ok(mut text) = score_query.get_single_mut() {
        text.sections[0].value = format!(
            "Score: Cartel {} - Military {}",
            game_state.cartel_score, game_state.military_score
        );
    }

    // Update difficulty display
    if let Ok(mut text) = difficulty_query.get_single_mut() {
        let adaptive_status = if ai_director.adaptive_difficulty {
            "AUTO"
        } else {
            "MANUAL"
        };
        // Adaptive difficulty status determined based on system configuration
        text.sections[0].value = format!(
            "Difficulty: {:.1} ({}) | Performance: {:.0}%\nD=Toggle | F1-F4=Set Level",
            ai_director.intensity_level,
            adaptive_status,
            ai_director.player_performance * 100.0
        );
    }
    // Creative toevoeging: indien in debug mode, log extra informatie voor een beter overzicht
    if cfg!(debug_assertions) {
        info!(
            "DEBUG: Game Phase: {:?}, Unit Count: {}",
            game_state.game_phase,
            unit_query.iter().count()
        );
    }
}

pub fn health_bar_system(
    mut commands: Commands,
    unit_query: Query<(Entity, &Unit, &Transform), Changed<Unit>>,
    mut health_bar_query: HealthBarQuery,
) {
    // Update health bars when units change
    for (unit_entity, unit, unit_transform) in unit_query.iter() {
        for (bar_entity, mut bar_transform, mut bar_sprite, health_bar) in
            health_bar_query.iter_mut()
        {
            if health_bar.owner == unit_entity {
                // Update position
                bar_transform.translation = unit_transform.translation + health_bar.offset;

                // Update health bar color and width based on health percentage
                let health_percent = unit.health / unit.max_health;
                let bar_color = if health_percent > 0.6 {
                    Color::rgb(0.2, 0.8, 0.2) // Green
                } else if health_percent > 0.3 {
                    Color::rgb(0.8, 0.8, 0.2) // Yellow
                } else {
                    Color::rgb(0.8, 0.2, 0.2) // Red
                };

                bar_sprite.color = bar_color;

                // Adjust bar width based on health (only for foreground bars)
                if health_bar.offset.z > 0.15 {
                    // Foreground bar
                    if let Some(ref mut size) = bar_sprite.custom_size {
                        size.x = 50.0 * health_percent;
                    }
                }

                // Remove health bar if unit is dead
                if unit.health <= 0.0 {
                    commands.entity(bar_entity).despawn();
                }
            }
        }
    }

    // Clean up health bars for dead units
    let living_units: std::collections::HashSet<Entity> = unit_query
        .iter()
        .filter(|(_, unit, _)| unit.health > 0.0)
        .map(|(entity, _, _)| entity)
        .collect();

    for (bar_entity, _, _, health_bar) in health_bar_query.iter() {
        if !living_units.contains(&health_bar.owner) {
            commands.entity(bar_entity).despawn();
        }
    }
}

pub fn damage_indicator_system(
    mut commands: Commands,
    mut damage_query: Query<(
        Entity,
        &mut Transform,
        &mut DamageIndicator,
        Option<&ParticleEffect>,
    )>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut indicator, particle_effect) in damage_query.iter_mut() {
        indicator.lifetime.tick(time.delta());

        // Use particle effect velocity if available, otherwise default upward movement
        if let Some(particle) = particle_effect {
            transform.translation += particle.velocity * time.delta_seconds();
        } else {
            transform.translation.y += 30.0 * time.delta_seconds();
        }

        // Fade out over time for smooth disappearance (future enhancement)
        let _alpha =
            1.0 - (indicator.lifetime.elapsed_secs() / indicator.lifetime.duration().as_secs_f32());

        // Remove when expired
        if indicator.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn particle_system(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut Transform, &mut ParticleEffect)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in particle_query.iter_mut() {
        particle.lifetime.tick(time.delta());

        // Move particle
        transform.translation += particle.velocity * time.delta_seconds();

        // Remove when expired
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
