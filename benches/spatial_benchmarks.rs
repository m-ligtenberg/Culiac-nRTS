use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bevy::prelude::*;
use culiacan_rts::utils::spatial::{SpatialGrid, GridCell};

fn spatial_grid_benchmark(c: &mut Criterion) {
    let mut grid = SpatialGrid::new(50.0);
    
    // Vul de grid met 10k test units
    for i in 0..10000 {
        let pos = Vec3::new(i as f32 % 1000.0, 0.0, (i / 1000) as f32);
        grid.insert_unit(Entity::from_raw_id(i), pos, 100.0);
    }
    
    let mut group = c.benchmark_group("Spatial Grid");
    
    group.bench_function("find_nearby_units (50m radius)", |b| b.iter(|| {
        grid.find_nearby_units(black_box(Vec3::new(500.0, 0.0, 500.0)), black_box(50.0))
    }));
    
    group.bench_function("find_nearby_units (150m radius)", |b| b.iter(|| {
        grid.find_nearby_units(black_box(Vec3::new(500.0, 0.0, 500.0)), black_box(150.0))
    }));
    
    group.bench_function("find_nearby_units (500m radius)", |b| b.iter(|| {
        grid.find_nearby_units(black_box(Vec3::new(500.0, 0.0, 500.0)), black_box(500.0))
    }));
    
    group.finish();
}

criterion_group!(benches, spatial_grid_benchmark);
criterion_main!(benches);
