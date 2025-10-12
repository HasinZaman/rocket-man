use std::{cmp::min, collections::HashSet};

use bevy::{
    app::AppExit,
    camera::Camera3d,
    ecs::{
        entity::Entity,
        hierarchy::{ChildOf, Children},
        message::{MessageReader, MessageWriter},
        query::{With, Without},
        resource::Resource,
        system::{Commands, Query, Res, ResMut, Single},
    },
    input::{
        ButtonState,
        keyboard::{Key, KeyCode, KeyboardInput},
        mouse::{MouseButton, MouseButtonInput},
    },
    math::{Ray3d, Vec2},
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
    time::Time,
    transform::components::{GlobalTransform, Transform},
};

use crate::{
    cf104::{CanopyDoor, CanopyDoorHandle, Joystick, RotRange, RotRange2D, Throttle},
    player::{Focused, Player, Selectable, Selected, camera::OutlineCamera},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    None,
    Pressed,
    Held,
    Released,
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub state: KeyState,
}

impl KeyBinding {
    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            state: KeyState::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArmBinding {
    pub up: KeyBinding,
    pub down: KeyBinding,
    pub left: KeyBinding,
    pub right: KeyBinding,
    pub alt_1: KeyBinding,
    pub alt_2: KeyBinding,
}

impl ArmBinding {
    pub fn pressed(&mut self, key_code: KeyCode) {
        if self.up.key == key_code {
            self.up.state = KeyState::Pressed;
        }
        if self.down.key == key_code {
            self.down.state = KeyState::Pressed;
        }
        if self.left.key == key_code {
            self.left.state = KeyState::Pressed;
        }
        if self.right.key == key_code {
            self.right.state = KeyState::Pressed;
        }
        if self.alt_1.key == key_code {
            self.alt_1.state = KeyState::Pressed;
        }
        if self.alt_2.key == key_code {
            self.alt_2.state = KeyState::Pressed;
        }
    }
    pub fn released(&mut self, key_code: KeyCode) {
        if self.up.key == key_code {
            self.up.state = KeyState::Released;
        }
        if self.down.key == key_code {
            self.down.state = KeyState::Released;
        }
        if self.left.key == key_code {
            self.left.state = KeyState::Released;
        }
        if self.right.key == key_code {
            self.right.state = KeyState::Released;
        }
        if self.alt_1.key == key_code {
            self.alt_1.state = KeyState::Released;
        }
        if self.alt_2.key == key_code {
            self.alt_2.state = KeyState::Released;
        }
    }

    pub fn update(&mut self) {
        self.up.state = match self.up.state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Held => KeyState::Held,
            KeyState::Released => KeyState::None,
            KeyState::None => KeyState::None,
        };
        self.down.state = match self.down.state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Held => KeyState::Held,
            KeyState::Released => KeyState::None,
            KeyState::None => KeyState::None,
        };
        self.left.state = match self.left.state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Held => KeyState::Held,
            KeyState::Released => KeyState::None,
            KeyState::None => KeyState::None,
        };
        self.right.state = match self.right.state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Held => KeyState::Held,
            KeyState::Released => KeyState::None,
            KeyState::None => KeyState::None,
        };
        self.alt_1.state = match self.alt_1.state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Held => KeyState::Held,
            KeyState::Released => KeyState::None,
            KeyState::None => KeyState::None,
        };
        self.alt_2.state = match self.alt_2.state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Held => KeyState::Held,
            KeyState::Released => KeyState::None,
            KeyState::None => KeyState::None,
        };
    }
}

#[derive(Resource, Debug)]
pub struct KeyBindings {
    pub left: ArmBinding,
    pub right: ArmBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            left: ArmBinding {
                up: KeyBinding::new(KeyCode::KeyW),
                down: KeyBinding::new(KeyCode::KeyS),
                left: KeyBinding::new(KeyCode::KeyA),
                right: KeyBinding::new(KeyCode::KeyD),
                alt_1: KeyBinding::new(KeyCode::KeyQ),
                alt_2: KeyBinding::new(KeyCode::KeyE),
            },
            right: ArmBinding {
                up: KeyBinding::new(KeyCode::KeyI),
                down: KeyBinding::new(KeyCode::KeyK),
                left: KeyBinding::new(KeyCode::KeyJ),
                right: KeyBinding::new(KeyCode::KeyL),
                alt_1: KeyBinding::new(KeyCode::KeyU),
                alt_2: KeyBinding::new(KeyCode::KeyO),
            },
        }
    }
}

pub fn update_key_bindings(
    mut reader: MessageReader<KeyboardInput>,
    mut bindings: ResMut<KeyBindings>,
    mut exit: MessageWriter<AppExit>,
) {
    bindings.left.update();
    bindings.right.update();

    for event in reader.read() {
        if event.key_code == KeyCode::Escape {
            exit.write(AppExit::Success);
            return;
        }
        match event.state {
            ButtonState::Pressed => {
                bindings.left.pressed(event.key_code);
                bindings.right.pressed(event.key_code);
            }
            ButtonState::Released => {
                bindings.left.released(event.key_code);
                bindings.right.released(event.key_code);
            }
        }

        // println!("{event:?}");
    }
    // println!("{bindings:?}");
}

// Arm resource
#[derive(Resource, Default, Debug)]
pub struct Arms(Option<Entity>, Option<Entity>);

// select obj

pub fn select_tool(
    mut mouse_button_events: MessageReader<MouseButtonInput>,

    mut commands: Commands,

    mut arms: ResMut<Arms>,

    camera_transform: Single<
        &GlobalTransform,
        (With<Camera3d>, With<Player>, Without<OutlineCamera>),
    >,
    mut raycast: MeshRayCast,

    mut selectable_query: Query<(Entity, &ChildOf, Option<&Selected>), (With<Selectable>)>,

    remove_query: Query<&Children, (With<Selected>, Without<Selectable>)>,
) {
    // TODO - check if in play mode
    for event in mouse_button_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        let button = event.button;

        let ray: Ray3d = Ray3d::new(camera_transform.translation(), camera_transform.forward());

        if let Some((entity, hit)) = raycast
            .cast_ray(ray, &MeshRayCastSettings::default())
            .iter()
            .find_map(|(e, h)| {
                if selectable_query.get(*e).is_ok() {
                    Some((*e, h.clone()))
                } else {
                    None
                }
            })
        {
            if let Ok((entity, ChildOf(parent_entity), _selected)) =
                selectable_query.get_mut(entity)
            {
                commands.entity(entity).insert(Selected);
                commands.entity(*parent_entity).insert(Selected);

                match (button, &mut *arms) {
                    (MouseButton::Left, Arms(Some(holding), other))
                    | (MouseButton::Right, Arms(other, Some(holding))) => {
                        println!("button: {button:?}");
                        if Some(*holding) != *other {
                            commands.entity(*holding).remove::<Selected>();
                            if let Ok(children) = remove_query.get(*holding) {
                                println!("parent: {holding:?}");
                                for child in children {
                                    println!("child: {child:?}");
                                    commands.entity(*child).remove::<Selected>();
                                }
                            };
                        }
                        *holding = *parent_entity;
                    }
                    (MouseButton::Left, Arms(arm, _)) | (MouseButton::Right, Arms(_, arm)) => {
                        *arm = Some(*parent_entity);
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        println!("{arms:?}");
    }
}

pub fn throttle_controller(
    arms: Res<Arms>,
    keybindings: Res<KeyBindings>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Throttle, &RotRange, &mut Transform), With<Selected>>,
) {
    const DELTA: f32 = 50.0;

    let delta_time = time.delta_secs();

    for (entity, mut throttle, range, mut transform) in &mut query {
        let holding = (arms.0 == Some(entity), arms.1 == Some(entity));

        match (
            (
                keybindings.left.up.state,
                keybindings.left.down.state,
                holding.0,
            ),
            (
                keybindings.right.up.state,
                keybindings.right.down.state,
                holding.1,
            ),
        ) {
            ((KeyState::Held | KeyState::Pressed, _, true), _)
            | (_, (KeyState::Held | KeyState::Pressed, _, true)) => {
                throttle.0 = f32::min(100.0, throttle.0 + DELTA * delta_time);
            }
            ((_, KeyState::Held | KeyState::Pressed, true), _)
            | (_, (_, KeyState::Held | KeyState::Pressed, true)) => {
                throttle.0 = f32::max(0.0, throttle.0 - DELTA * delta_time);
            }
            _ => {}
        }

        let t = throttle.0 / 100.0;

        let target_rotation = range.min.slerp(range.max, t);

        transform.rotation = transform.rotation.slerp(target_rotation, 0.15);
    }
}

pub fn joystick_controller(
    arms: Res<Arms>,
    keybindings: Res<KeyBindings>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Joystick, &RotRange2D, &mut Transform), With<Selected>>,
) {
    const RETURN_SPEED: f32 = 0.9;
    const INPUT_SPEED: f32 = 1.0;

    for (entity, mut joystick, range, mut transform) in &mut query {
        let holding = (arms.0 == Some(entity), arms.1 == Some(entity));
        let mut input = Vec2::ZERO;
        match (
            (keybindings.left.up.state, holding.0),
            (keybindings.right.up.state, holding.1),
        ) {
            ((KeyState::Held | KeyState::Pressed, true), _)
            | (_, (KeyState::Held | KeyState::Pressed, true)) => {
                input.y -= 1.0;
            }
            _ => {}
        };
        match (
            (keybindings.left.down.state, holding.0),
            (keybindings.right.down.state, holding.1),
        ) {
            ((KeyState::Held | KeyState::Pressed, true), _)
            | (_, (KeyState::Held | KeyState::Pressed, true)) => {
                input.y += 1.0;
            }
            _ => {}
        };
        match (
            (keybindings.left.left.state, holding.0),
            (keybindings.right.left.state, holding.1),
        ) {
            ((KeyState::Held | KeyState::Pressed, true), _)
            | (_, (KeyState::Held | KeyState::Pressed, true)) => {
                input.x -= 1.0;
            }
            _ => {}
        };
        match (
            (keybindings.left.right.state, holding.0),
            (keybindings.right.right.state, holding.1),
        ) {
            ((KeyState::Held | KeyState::Pressed, true), _)
            | (_, (KeyState::Held | KeyState::Pressed, true)) => {
                input.x += 1.0;
            }
            _ => {}
        };

        // Normalize diagonal movement
        if input.length_squared() > 1.0 {
            input = input.normalize();
        }

        let delta_time = time.delta_secs();

        if input != Vec2::ZERO {
            joystick.0 = joystick.0.lerp(input, delta_time * INPUT_SPEED);
        } else {
            joystick.0 = joystick.0.lerp(Vec2::ZERO, delta_time * RETURN_SPEED);
        }

        let target_rotation = range.to_quat(joystick.0);
        transform.rotation = transform.rotation.slerp(target_rotation, 0.15);
    }
}

pub fn canopy_door_controller(
    time: Res<Time>,
    arms: Res<Arms>,
    keybindings: Res<KeyBindings>,
    mut doors: Query<(&mut CanopyDoor, &RotRange, &mut Transform)>,
    mut handles: Query<(Entity, &ChildOf), (With<CanopyDoorHandle>, With<Selected>)>,
    mut commands: Commands,
) {
    const DELTA: f32 = 75.0;

    let delta_time = time.delta_secs();

    for (entity, ChildOf(door_entity)) in &mut handles {
        let holding = (arms.0 == Some(entity), arms.1 == Some(entity));

        let (mut door, range, mut transform) = doors.get_mut(*door_entity).unwrap();

        match (
            (
                keybindings.left.up.state,
                keybindings.left.down.state,
                holding.0,
            ),
            (
                keybindings.right.up.state,
                keybindings.right.down.state,
                holding.1,
            ),
        ) {
            (
                (KeyState::Held | KeyState::Pressed, KeyState::None | KeyState::Released, true),
                _,
            )
            | (
                _,
                (KeyState::Held | KeyState::Pressed, KeyState::None | KeyState::Released, true),
            ) => {
                door.0 = f32::min(100.0, door.0 + DELTA * delta_time);
            }
            (
                (KeyState::None | KeyState::Released, KeyState::Held | KeyState::Pressed, true),
                _,
            )
            | (
                _,
                (KeyState::None | KeyState::Released, KeyState::Held | KeyState::Pressed, true),
            ) => {
                door.0 = f32::max(0.0, door.0 - DELTA * delta_time);
            }
            _ => {}
        };

        if door.0 <= 0.001 {
            door.0 = 0.;

            commands.entity(entity).remove::<CanopyDoorHandle>();
        }

        let t = door.0 / 100.0;

        let target_rotation = range.min.slerp(range.max, t);

        transform.rotation = transform.rotation.slerp(target_rotation, 0.15);
    }
}
