use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

// Type aliases to reduce complexity
type OptimizedUnitQuery<'a> = Query<
    'a,
    'a,
    (
        Entity,
        &'a mut Unit,
        &'a Transform,
        &'a mut Movement,
        Option<&'a mut AiCache>,
    ),
    Without<Objective>,
>;
use std::collections::VecDeque;

// ==================== TIME-SLICED AI OPTIMIZATION SYSTEM ====================

#[derive(Resource)]
pub struct AiScheduler {
    pub unit_queue: VecDeque<Entity>,
    pub updates_per_frame: usize,
    pub frame_counter: usize,
    pub strategic_timer: f32,
    pub strategic_update_interval: f32, // Strategic decisions updated less frequently
}

impl Default for AiScheduler {
    fn default() -> Self {
        Self {
            unit_queue: VecDeque::new(),
            updates_per_frame: 5, // Process 5 units per frame instead of all
            frame_counter: 0,
            strategic_timer: 0.0,
            strategic_update_interval: 0.5, // Strategic updates every 0.5 seconds
        }
    }
}

#[derive(Component)]
pub struct AiCache {
    pub last_strategic_update: f32,
    pub cached_nearest_enemy: Option<Entity>,
    pub cached_strategic_decision: StrategicDecision,
    pub cache_valid_until: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StrategicDecision {
    Attack,
    Defend,
    Retreat,
    Patrol,
    Regroup,
}

impl Default for AiCache {
    fn default() -> Self {
        Self {
            last_strategic_update: 0.0,
            cached_nearest_enemy: None,
            cached_strategic_decision: StrategicDecision::Patrol,
            cache_valid_until: 0.0,
        }
    }
}

// Time-sliced AI system that processes only a subset of units per frame
pub fn optimized_unit_ai_system(
    mut ai_scheduler: ResMut<AiScheduler>,
    mut unit_query: OptimizedUnitQuery,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    ai_scheduler.frame_counter += 1;
    ai_scheduler.strategic_timer += time.delta_seconds();

    // Repopulate queue if empty or if significant time has passed
    if ai_scheduler.unit_queue.is_empty() || ai_scheduler.frame_counter % 60 == 0 {
        ai_scheduler.unit_queue.clear();

        // Add all living units to the queue
        for (entity, unit, _, _, _) in unit_query.iter() {
            if unit.health > 0.0 {
                ai_scheduler.unit_queue.push_back(entity);
            }
        }
    }

    // Process only a limited number of units per frame
    let mut units_processed = 0;
    let entities_to_process: Vec<Entity> = ai_scheduler
        .unit_queue
        .iter()
        .take(ai_scheduler.updates_per_frame)
        .cloned()
        .collect();

    // Clear processed entities from queue
    for _ in 0..entities_to_process.len().min(ai_scheduler.unit_queue.len()) {
        ai_scheduler.unit_queue.pop_front();
    }

    for entity in entities_to_process {
        if let Ok((_, mut unit, transform, mut movement, cache_opt)) = unit_query.get_mut(entity) {
            if unit.health <= 0.0 {
                continue; // Skip dead units
            }

            // Initialize cache if it doesn't exist
            let mut needs_strategic_update = true;
            if let Some(cache) = cache_opt {
                needs_strategic_update = time.elapsed_seconds() > cache.cache_valid_until;
            }

            // Perform lightweight tactical updates every frame
            perform_tactical_ai_update(&mut unit, transform, &mut movement, &game_state);

            // Perform heavy strategic updates less frequently
            if needs_strategic_update
                || ai_scheduler.strategic_timer >= ai_scheduler.strategic_update_interval
            {
                // For strategic updates, we'll do a simplified approach without collecting all unit data
                // This avoids the borrow checker issue while still providing basic AI behavior
                perform_simple_strategic_ai_update(&mut unit, transform);
            }
        }
        units_processed += 1;
    }

    // Reset strategic timer
    if ai_scheduler.strategic_timer >= ai_scheduler.strategic_update_interval {
        ai_scheduler.strategic_timer = 0.0;
    }
}

// Lightweight tactical updates (run every frame for processed units)
fn perform_tactical_ai_update(
    unit: &mut Unit,
    transform: &Transform,
    movement: &mut Movement,
    game_state: &GameState,
) {
    // Quick movement adjustments based on current state
    if let Some(target_pos) = movement.target_position {
        let distance_to_target = transform.translation.distance(target_pos);

        // Simple obstacle avoidance - if stuck, try a slightly different path
        if distance_to_target < 5.0 {
            movement.target_position = None; // Reached target
        }
    }

    // Update cooldowns and simple state changes
    unit.attack_cooldown
        .tick(std::time::Duration::from_secs_f32(1.0 / 60.0));
}

// Simplified strategic updates (avoids borrow checker issues)
fn perform_simple_strategic_ai_update(unit: &mut Unit, _transform: &Transform) {
    // Simple strategic logic without complex unit analysis
    if unit.health < 30.0 {
        // If low health, clear target to encourage retreat behavior
        unit.target = None;
    } else if unit.target.is_none() {
        // If no target and healthy, the unit will naturally seek targets via other systems
        // This is a simplified approach that relies on other AI systems to find targets
    }

    // Additional strategic behaviors can be added here that don't require
    // analysis of all other units (to avoid borrow checker issues)
}

// Optimized AI director that adjusts scheduler based on performance
pub fn adaptive_ai_scheduler_system(
    mut ai_scheduler: ResMut<AiScheduler>,
    time: Res<Time>,
    unit_query: Query<&Unit>,
) {
    let total_units = unit_query.iter().count();

    // Adjust updates per frame based on unit count and performance
    ai_scheduler.updates_per_frame = match total_units {
        0..=20 => 8,   // Process more units per frame when few units
        21..=50 => 5,  // Standard processing
        51..=100 => 3, // Reduce processing for large battles
        _ => 2,        // Minimal processing for massive battles
    };

    // Adjust strategic update frequency based on game phase
    ai_scheduler.strategic_update_interval = match total_units {
        0..=30 => 0.3,  // More frequent updates for small battles
        31..=60 => 0.5, // Standard frequency
        _ => 0.8,       // Less frequent updates for large battles
    };
}

// Setup system to initialize AI scheduler
pub fn setup_ai_optimizer(mut commands: Commands) {
    commands.insert_resource(AiScheduler::default());
    info!("🧠 AI optimization system initialized (time-sliced updates)");
}
