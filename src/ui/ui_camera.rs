use crate::components::*;
use bevy::prelude::*;

// ==================== CAMERA CONTROL SYSTEM ====================

pub fn camera_control_system(
    mut camera_query: Query<(&mut Transform, &mut IsometricCamera), With<Camera>>,
    input: Res<Input<KeyCode>>,
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    time: Res<Time>,
    mut windows: Query<&mut Window>,
    mut stored_window_size: Local<Vec2>,
) {
    // Robust camera control with error handling
    let Ok((mut transform, camera)) = camera_query.get_single_mut() else {
        warn!("Camera system: No camera found or multiple cameras detected");
        return;
    };

    let mut movement = Vec3::ZERO;

    // WASD camera movement
    if input.pressed(KeyCode::W) {
        movement.y += 1.0;
    }
    if input.pressed(KeyCode::S) {
        movement.y -= 1.0;
    }
    if input.pressed(KeyCode::A) {
        movement.x -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        movement.x += 1.0;
    }

    // Apply movement
    if movement != Vec3::ZERO {
        transform.translation += movement.normalize() * camera.pan_speed * time.delta_seconds();
    }

    // Mouse wheel zoom
    for scroll in scroll_events.read() {
        let zoom_delta = -scroll.y * camera.zoom_speed;
        let new_scale = (transform.scale.x + zoom_delta).clamp(camera.min_zoom, camera.max_zoom);
        transform.scale = Vec3::splat(new_scale);
    }
}
