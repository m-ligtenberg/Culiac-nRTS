use crate::components::*;
use bevy::prelude::*;

// ==================== ANIMATION SYSTEMS ====================

pub fn sprite_animation_system(
    mut animated_query: Query<(&mut Transform, &mut AnimatedSprite)>,
    time: Res<Time>,
) {
    for (mut transform, mut animated_sprite) in animated_query.iter_mut() {
        animated_sprite.animation_timer.tick(time.delta());

        // Pulsing scale animation
        let time_ratio = animated_sprite.animation_timer.elapsed_secs()
            / animated_sprite.animation_timer.duration().as_secs_f32();
        let pulse = (time_ratio * std::f32::consts::PI * 2.0).sin();
        let scale_modifier = 1.0 + pulse * animated_sprite.scale_amplitude;

        transform.scale = animated_sprite.base_scale * scale_modifier;

        // Gentle rotation
        transform.rotation =
            Quat::from_rotation_z(animated_sprite.rotation_speed * time.delta_seconds())
                * transform.rotation;

        // Reset timer when finished
        if animated_sprite.animation_timer.finished() {
            animated_sprite.animation_timer.reset();
        }
    }
}

pub fn movement_animation_system(
    mut movement_anim_query: Query<(&mut Transform, &mut MovementAnimation, &Movement)>,
    time: Res<Time>,
) {
    for (mut transform, mut movement_anim, movement) in movement_anim_query.iter_mut() {
        movement_anim.bob_timer.tick(time.delta());

        // Only animate when moving
        if movement.target_position.is_some() {
            let bob_time = movement_anim.bob_timer.elapsed_secs();
            let bob_offset = (bob_time * 8.0).sin() * movement_anim.bob_amplitude;
            transform.translation.y = movement_anim.base_y + bob_offset;
        } else {
            // Return to base position when not moving
            transform.translation.y = movement_anim.base_y;
        }

        // Reset timer periodically
        if movement_anim.bob_timer.finished() {
            movement_anim.bob_timer.reset();
        }
    }
}
