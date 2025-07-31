use crate::components::*;
use bevy::prelude::*;
use std::collections::HashMap;

// Implementatie van spatial grid zoals gedefinieerd in je code
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
            GridCell {
                x: self.x - 1,
                y: self.y - 1,
            },
            GridCell {
                x: self.x,
                y: self.y - 1,
            },
            GridCell {
                x: self.x + 1,
                y: self.y - 1,
            },
            GridCell {
                x: self.x - 1,
                y: self.y,
            },
            GridCell {
                x: self.x + 1,
                y: self.y,
            },
            GridCell {
                x: self.x - 1,
                y: self.y + 1,
            },
            GridCell {
                x: self.x,
                y: self.y + 1,
            },
            GridCell {
                x: self.x + 1,
                y: self.y + 1,
            },
        ]
    }
}

pub struct SpatialGrid {
    pub cell_size: f32,
    pub units: HashMap<GridCell, Vec<(Entity, Vec3, f32)>>,
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
            .or_default()
            .push((entity, position, max_range));
    }
}
