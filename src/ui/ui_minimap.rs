use crate::components::*;
use bevy::prelude::*;

// Type aliases to reduce complexity
type MiniMapIconQuery<'a> = Query<
    'a,
    'a,
    (Entity, &'a mut Style, &'a MiniMapIcon),
    (With<MiniMapIcon>, Without<Transform>),
>;

// ==================== MINIMAP SYSTEM ====================

pub fn minimap_system(
    mut commands: Commands,
    unit_query: Query<(&Transform, &Unit), Without<MiniMapIcon>>,
    minimap_icon_query: MiniMapIconQuery,
    minimap_query: Query<Entity, With<MiniMap>>,
) {
    if let Ok(minimap_entity) = minimap_query.get_single() {
        // Clear old icons
        for (entity, _, _) in minimap_icon_query.iter() {
            commands.entity(entity).despawn();
        }

        // Create new icons for all living units
        for (transform, unit) in unit_query.iter() {
            if unit.health <= 0.0 {
                continue;
            }

            // Scale world position to minimap coordinates (200x150 minimap)
            let minimap_x = (transform.translation.x / 1000.0) * 100.0 + 100.0; // Center at 100
            let minimap_y = (transform.translation.y / 750.0) * 75.0 + 75.0; // Center at 75

            let icon_color = match unit.faction {
                Faction::Cartel => Color::RED,
                Faction::Military => Color::GREEN,
                _ => Color::WHITE,
            };

            commands.entity(minimap_entity).with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(minimap_x),
                            top: Val::Px(minimap_y),
                            width: Val::Px(4.0),
                            height: Val::Px(4.0),
                            ..default()
                        },
                        background_color: BackgroundColor(icon_color),
                        ..default()
                    },
                    MiniMapIcon {
                        unit_type: unit.unit_type.clone(),
                        faction: unit.faction.clone(),
                    },
                ));
            });
        }
    }
}
