use bevy::prelude::*;
use crate::components::*;
use std::collections::HashMap;

// ==================== SPATIAL PARTITIONING SYSTEM ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
}

impl GridCell {
    pub fn from_position(position: Vec3, cell_size: f32) -> Self {
        Self {
            x: (position.x / cell_size).floor() as i32,
            y: (position.z / cell_size).floor() as i32,
        }
    }

    pub fn get_neighbors(&self) -> Vec<GridCell> {
        vec![
            *self,
            GridCell { x: self.x - 1, y: self.y - 1 },
            GridCell { x: self.x, y: self.y - 1 },
            GridCell { x: self.x + 1, y: self.y - 1 },
            GridCell { x: self.x - 1, y: self.y },
            GridCell { x: self.x + 1, y: self.y },
            GridCell { x: self.x - 1, y: self.y + 1 },
            GridCell { x: self.x, y: self.y + 1 },
            GridCell { x: self.x + 1, y: self.y + 1 },
        ]
    }
}

pub struct SpatialGrid {
    pub cell_size: f32,
    pub units: HashMap<GridCell, Vec<(Entity, Vec3, f32)>>, // Entity, position, max_range
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            units: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.units.clear();
    }

    pub fn insert_unit(&mut self, entity: Entity, position: Vec3, max_range: f32) {
        let cell = GridCell::from_position(position, self.cell_size);
        self.units
            .entry(cell)
            .or_insert_with(Vec::new)
            .push((entity, position, max_range));
    }

    pub fn find_nearby_units(&self, position: Vec3, search_range: f32) -> Vec<(Entity, Vec3, f32)> {
        let center_cell = GridCell::from_position(position, self.cell_size);
        let mut nearby_units = Vec::new();

        // Check center cell and all neighboring cells
        for cell in center_cell.get_neighbors() {
            if let Some(units_in_cell) = self.units.get(&cell) {
                for &(entity, unit_pos, max_range) in units_in_cell {
                    let distance = position.distance(unit_pos);
                    
                    // Include unit if it's within search range or if the unit's own range reaches us
                    if distance <= search_range + max_range {
                        nearby_units.push((entity, unit_pos, max_range));
                    }
                }
            }
        }

        nearby_units
    }
}

pub fn find_combat_pairs_optimized(
    units: &[(Entity, &Unit, &Transform)], 
    visibility_modifier: f32
) -> Vec<(Entity, Entity, f32)> {
    if units.len() < 10 {
        // For small unit counts, use the original O(n²) algorithm (it's faster due to less overhead)
        return find_combat_pairs_simple(units, visibility_modifier);
    }

    let mut combat_events = Vec::new();
    let cell_size = 50.0; // Cells of 50x50 units
    let mut spatial_grid = SpatialGrid::new(cell_size);

    // Phase 1: Populate spatial grid
    for (entity, unit, transform) in units.iter() {
        if unit.health > 0.0 {
            let effective_range = unit.range * visibility_modifier;
            spatial_grid.insert_unit(*entity, transform.translation, effective_range);
        }
    }

    // Phase 2: Find combat pairs using spatial queries
    for (entity_a, unit_a, transform_a) in units.iter() {
        if unit_a.health <= 0.0 || !unit_a.attack_cooldown.finished() {
            continue;
        }

        let effective_range_a = unit_a.range * visibility_modifier;
        
        // Try to attack assigned target first (same as original)
        if let Some(target_entity) = unit_a.target {
            if let Some((_, target_unit, target_transform)) = units.iter()
                .find(|(entity, _, _)| *entity == target_entity) {
                
                if target_unit.health > 0.0 
                    && target_unit.faction != unit_a.faction 
                    && transform_a.translation.distance(target_transform.translation) <= effective_range_a {
                    combat_events.push((*entity_a, target_entity, unit_a.damage));
                    continue;
                }
            }
        }

        // Find nearby enemies using spatial grid
        let nearby_units = spatial_grid.find_nearby_units(transform_a.translation, effective_range_a);
        
        for (entity_b, pos_b, _) in nearby_units {
            if entity_b == *entity_a {
                continue; // Skip self
            }

            // Find the actual unit data
            if let Some((_, unit_b, _)) = units.iter().find(|(e, _, _)| *e == entity_b) {
                if unit_a.faction == unit_b.faction || unit_b.health <= 0.0 {
                    continue;
                }

                let distance = transform_a.translation.distance(pos_b);
                let effective_range_b = unit_b.range * visibility_modifier;

                // Check if units are in range to attack each other
                if distance <= effective_range_a {
                    combat_events.push((*entity_a, entity_b, unit_a.damage));
                }
                if distance <= effective_range_b && unit_b.attack_cooldown.finished() {
                    combat_events.push((entity_b, *entity_a, unit_b.damage));
                }
            }
        }
    }

    combat_events
}

// Fallback simple algorithm for small unit counts
fn find_combat_pairs_simple(
    units: &[(Entity, &Unit, &Transform)], 
    visibility_modifier: f32
) -> Vec<(Entity, Entity, f32)> {
    let mut combat_events = Vec::new();
    
    for (i, (entity_a, unit_a, transform_a)) in units.iter().enumerate() {
        if unit_a.health <= 0.0 || !unit_a.attack_cooldown.finished() {
            continue;
        }
        
        let effective_range_a = unit_a.range * visibility_modifier;
        
        // Try to attack assigned target first
        if let Some(target_entity) = unit_a.target {
            if let Some((_, target_unit, target_transform)) = units.iter()
                .find(|(entity, _, _)| *entity == target_entity) {
                
                if target_unit.health > 0.0 
                    && target_unit.faction != unit_a.faction 
                    && transform_a.translation.distance(target_transform.translation) <= effective_range_a {
                    combat_events.push((*entity_a, target_entity, unit_a.damage));
                    continue;
                }
            }
        }
        
        // General combat - attack nearest enemy if no specific target
        for (entity_b, unit_b, transform_b) in units.iter().skip(i + 1) {
            if unit_a.faction == unit_b.faction || unit_b.health <= 0.0 {
                continue;
            }
            
            let distance = transform_a.translation.distance(transform_b.translation);
            let effective_range_b = unit_b.range * visibility_modifier;
            
            if distance <= effective_range_a {
                combat_events.push((*entity_a, *entity_b, unit_a.damage));
            }
            if distance <= effective_range_b && unit_b.attack_cooldown.finished() {
                combat_events.push((*entity_b, *entity_a, unit_b.damage));
            }
        }
    }
    
    combat_events
}