use bevy::prelude::*;
use bevy::ui::{Node, PositionType};
use bevy::window::{CursorOptions, PrimaryWindow, WindowMode};

pub fn fullscreen_startup(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
}

#[derive(Component)]
pub struct BlackoutRedout;

pub fn set_up_ui(mut commands: Commands, mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = false;

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        ZIndex(0),
        BackgroundGradient::from(RadialGradient {
            stops: vec![
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(25.0)),
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(50.0)),
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 1.0), Val::Percent(75.0)),
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 1.0), Val::Percent(100.0)),
            ],
            ..Default::default()
        }),
        BlackoutRedout,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(10.0),
            height: Val::Px(10.0),
            ..Default::default()
        },
        ZIndex(1),
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
