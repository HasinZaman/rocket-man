use bevy::ui::{Node, PositionType};
use bevy::window::{CursorOptions, PrimaryWindow, WindowMode};
use bevy::{prelude::*};


pub fn fullscreen_startup(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
}

pub fn hide_cursor(mut commands: Commands, mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = false;

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(10.0),
            height: Val::Px(10.0),
            ..Default::default()
        },
        BackgroundColor(Color::WHITE),
    ));
}

pub fn center_cursor(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    // Calculate center
    let center_x = window.width() / 2.0;
    let center_y = window.height() / 2.0;

    // Move cursor to center
    window.set_cursor_position(Some(Vec2::new(center_x, center_y)));
}
