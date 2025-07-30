use bevy::prelude::*;

// ==================== UI BUILDER UTILITIES ====================

/// Create a standard menu container with title
pub fn create_menu_container(title: &str) -> (NodeBundle, TextBundle) {
    let container = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.8).into(),
        ..default()
    };

    let title_text = TextBundle::from_section(
        title,
        TextStyle {
            font_size: 48.0,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_style(Style {
        margin: UiRect::all(Val::Px(20.0)),
        ..default()
    });

    (container, title_text)
}

/// Create a standard button with text
pub fn create_button_with_text(text: &str, color: Color) -> (NodeBundle, TextBundle) {
    let button = NodeBundle {
        style: Style {
            width: Val::Px(200.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        background_color: color.into(),
        border_color: Color::WHITE.into(),
        ..default()
    };

    let button_text = TextBundle::from_section(
        text,
        TextStyle {
            font_size: 24.0,
            color: Color::WHITE,
            ..default()
        },
    );

    (button, button_text)
}

/// Create a text section with specified properties
pub fn create_text_section(text: &str, size: f32, color: Color) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font_size: size,
            color,
            ..default()
        },
    )
    .with_style(Style {
        margin: UiRect::all(Val::Px(5.0)),
        ..default()
    })
}

/// Create an info panel with title and content
pub fn create_info_panel(title: &str, content: &str) -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Px(300.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            margin: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgba(0.1, 0.1, 0.1, 0.9).into(),
        border_color: Color::rgb(0.4, 0.4, 0.4).into(),
        ..default()
    }
}

/// Create a progress bar
pub fn create_progress_bar(
    current: f32,
    max: f32,
    width: f32,
    height: f32,
) -> (NodeBundle, NodeBundle) {
    let progress_ratio = if max > 0.0 { current / max } else { 0.0 };

    let background = NodeBundle {
        style: Style {
            width: Val::Px(width),
            height: Val::Px(height),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgb(0.2, 0.2, 0.2).into(),
        border_color: Color::WHITE.into(),
        ..default()
    };

    let fill = NodeBundle {
        style: Style {
            width: Val::Px(width * progress_ratio),
            height: Val::Percent(100.0),
            ..default()
        },
        background_color: Color::rgb(0.2, 0.8, 0.2).into(),
        ..default()
    };

    (background, fill)
}

/// Create a health bar with color coding
pub fn create_health_bar(
    current_health: f32,
    max_health: f32,
    width: f32,
    height: f32,
) -> (NodeBundle, NodeBundle) {
    let health_ratio = if max_health > 0.0 {
        current_health / max_health
    } else {
        0.0
    };

    let health_color = if health_ratio > 0.6 {
        Color::rgb(0.2, 0.8, 0.2) // Green
    } else if health_ratio > 0.3 {
        Color::rgb(0.8, 0.8, 0.2) // Yellow
    } else {
        Color::rgb(0.8, 0.2, 0.2) // Red
    };

    let background = NodeBundle {
        style: Style {
            width: Val::Px(width),
            height: Val::Px(height),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgb(0.3, 0.1, 0.1).into(),
        border_color: Color::rgb(0.5, 0.5, 0.5).into(),
        ..default()
    };

    let fill = NodeBundle {
        style: Style {
            width: Val::Px(width * health_ratio),
            height: Val::Percent(100.0),
            ..default()
        },
        background_color: health_color.into(),
        ..default()
    };

    (background, fill)
}

/// Create a list container
pub fn create_list_container() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.1).into(),
        ..default()
    }
}

/// Create a horizontal container
pub fn create_horizontal_container() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Auto,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        ..default()
    }
}

/// Create a centered container
pub fn create_centered_container() -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    }
}

/// Create a status indicator (colored dot with text)
pub fn create_status_indicator(status: &str, color: Color) -> (NodeBundle, TextBundle) {
    let dot = NodeBundle {
        style: Style {
            width: Val::Px(12.0),
            height: Val::Px(12.0),
            margin: UiRect::right(Val::Px(8.0)),
            ..default()
        },
        background_color: color.into(),
        ..default()
    };

    let text = TextBundle::from_section(
        status,
        TextStyle {
            font_size: 16.0,
            color: Color::WHITE,
            ..default()
        },
    );

    (dot, text)
}

/// Create a tooltip container
pub fn create_tooltip(text: &str) -> NodeBundle {
    NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.9).into(),
        border_color: Color::rgb(0.6, 0.6, 0.6).into(),
        visibility: Visibility::Hidden,
        ..default()
    }
}

/// Create a minimap container
pub fn create_minimap_container(size: f32) -> NodeBundle {
    NodeBundle {
        style: Style {
            width: Val::Px(size),
            height: Val::Px(size),
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
        border_color: Color::WHITE.into(),
        ..default()
    }
}

/// Create a mission objective item
pub fn create_objective_item(text: &str, completed: bool) -> (NodeBundle, TextBundle) {
    let color = if completed {
        Color::rgb(0.2, 0.8, 0.2) // Green for completed
    } else {
        Color::rgb(0.8, 0.8, 0.2) // Yellow for active
    };

    let checkbox = NodeBundle {
        style: Style {
            width: Val::Px(16.0),
            height: Val::Px(16.0),
            margin: UiRect::right(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: if completed {
            Color::rgb(0.2, 0.8, 0.2).into()
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.0).into()
        },
        border_color: color.into(),
        ..default()
    };

    let objective_text = TextBundle::from_section(
        text,
        TextStyle {
            font_size: 18.0,
            color,
            ..default()
        },
    );

    (checkbox, objective_text)
}
