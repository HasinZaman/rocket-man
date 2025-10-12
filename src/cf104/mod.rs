use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::{player::{
    camera::{mask_mesh, set_up_player_camera, MaskMaterials}, Player, Selectable
}, projectile::PlaneBundle};

// CF104
#[derive(Component)]
pub struct Plane;

#[derive(Component)]
pub struct Throttle(pub f32);

impl Default for Throttle {
    fn default() -> Self {
        Self(0.)
    }
}

#[derive(Component)]
pub struct RotRange {
    pub min: Quat,
    pub max: Quat,
}

#[derive(Component)]
pub struct Joystick(pub Vec2);

impl Default for Joystick {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct RotRange2D {
    pub center: Quat,
    pub radius: Vec2,
}

impl RotRange2D {
    pub fn new(center: Quat, radius: Vec2) -> Self {
        Self { center, radius }
    }

    pub fn to_quat(&self, input: Vec2) -> Quat {
        let clamped = input.clamp(Vec2::splat(-1.0), Vec2::splat(1.0));

        let yaw = self.radius.x * clamped.x;
        let pitch = self.radius.y * clamped.y;

        let offset = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

        self.center * offset
    }
}

pub struct CF104Plugin;
impl Plugin for CF104Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_player);
        //    .add_systems(Update, move_cf104); // optional: to move/scale the root later
    }
}

fn load_cf104<const PLAYER: bool>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,

    mask_materials: &Res<MaskMaterials>,
    images: &mut ResMut<Assets<Image>>,

    tip_fuel_tanks: Option<f32>,
) -> Entity {
    let cf104_asset_path = "cf104\\meshes.gltf";

    let body_id = commands.spawn((
        Player,
        Plane,
        PlaneBundle::cf_104(),
        Transform::default()
    )).id();

    // load body
    let body_id = {
        let parent_mesh_handle: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 11));
        let parent_material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3::splat(0.);

        transform.rotation = Quat::from_rotation_z(FRAC_PI_2) * Quat::from_rotation_y(FRAC_PI_2);

        transform.scale = Vec3::splat(1.);

        commands.spawn((
            Mesh3d(parent_mesh_handle),
            MeshMaterial3d(parent_material_handle),
            transform,
            ChildOf(body_id)
        )).id()
    };


    // load canopy shell
    let canopy_bundle = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 9));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: -0.46190130710601807,
            z: 1.4541336297988892,
        };

        let canopy_window_bundle = {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 0));
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgba(0.8, 0.8, 1.0, 0.25),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });

            let transform = Transform::default();

            (Mesh3d(mesh), MeshMaterial3d(material_handle), transform)
        };

        (
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            transform,
            children![(canopy_window_bundle)],
            ChildOf(body_id),
        )
    };
    let canopy_id = commands.spawn(canopy_bundle).id(); //.set_parent_in_place(body_id);

    // load canopy door
    let canopy_door_bundle = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 8));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();
        transform.rotation = Quat::from_rotation_y(-FRAC_PI_2);

        transform.translation = Vec3 {
            x: -0.5362579822540283,
            y: 0.,
            z: -0.4400066137313843,
        };

        let canopy_window_bundle = {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 7));
            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgba(0.8, 0.8, 1.0, 0.25),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });

            let transform = Transform::default();

            (Mesh3d(mesh), MeshMaterial3d(material_handle), transform)
        };

        (
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            transform,
            children![(canopy_window_bundle)],
            ChildOf(canopy_id),
        )
    };
    commands.spawn(canopy_door_bundle);

    // load cockpit shell
    let shell_id = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 6));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: 0.,
            z: 0.0027964115142822266,
        };

        let shell_id = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material_handle),
                transform,
                ChildOf(canopy_id),
            ))
            .id();

        mask_mesh::<true>(mask_materials, mesh.clone(), shell_id, commands);

        shell_id
    };

    // load console
    {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 2));
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            cull_mode: None,
            ..default()
        });

        // let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: 2.062485694885254,
            z: -1.4541336297988892,
        };

        mask_mesh::<true>(
            mask_materials,
            mesh.clone(),
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material_handle),
                    transform,
                    ChildOf(shell_id),
                ))
                .id(),
            commands,
        )
    };

    // load seat
    {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 4));
        // let material_handle = materials.add(StandardMaterial::default());

        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.1, 0.1),
            cull_mode: None,
            ..default()
        });

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: -0.736794650554657,
            z: -1.4541336297988892,
        };

        mask_mesh::<true>(
            mask_materials,
            mesh.clone(),
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material_handle),
                    transform,
                    ChildOf(shell_id),
                ))
                .id(),
            commands,
        )
    };

    let seat_back_bundle = {
        let mesh: Handle<Mesh> =
            asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 1));
        let material_handle = materials.add(StandardMaterial::default());

        let mut transform = Transform::default();

        transform.translation = Vec3 {
            x: 0.,
            y: -2.4246456623077393,
            z: -1.4541336297988892,
        };

        (
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            transform,
            ChildOf(shell_id),
        )
    };
    commands.spawn(seat_back_bundle);

    if let Some(fuel_level) = tip_fuel_tanks {
        for i in 0..2 {
            let fuel_tank_bundle = {
                let mesh: Handle<Mesh> =
                    asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 10));
                let material_handle = materials.add(StandardMaterial::default());

                let mut transform = Transform::default();

                transform.translation = Vec3 {
                    x: (i * 2 - 1) as f32 * -7.403514862060547,
                    y: -6.743416786193848,
                    z: -0.3721950650215149,
                };

                (
                    Mesh3d(mesh),
                    MeshMaterial3d(material_handle),
                    transform,
                    ChildOf(body_id),
                )
            };
            commands.spawn(fuel_tank_bundle);
        }
    }

    if PLAYER {
        {
            let camera_parent = commands.spawn((
                {
                    let mut transform: Transform = Transform::default();

                    transform.translation = Vec3 {
                        x: 0.,
                        y: -0.65,
                        z: 0.,
                    };
                    transform.rotation = Quat::from_euler(EulerRot::XYZ, FRAC_PI_2, 0., 0.);

                    transform
                },
                ChildOf(shell_id)
            )).id();
            

            set_up_player_camera(commands, Transform::default(), images, Some(camera_parent));
        };

        {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 5));
            let material_handle = materials.add(StandardMaterial::default());

            let mut transform: Transform = Transform::default();

            transform.translation = Vec3 {
                x: -0.3804253339767456,
                y: 0.2708113491535187,
                z: -0.9076950550079346,
            };

            transform.rotation = Quat::from_xyzw(0.5193636417388916, 0., 0., 0.8545534610748291);
            transform.scale = Vec3::splat(1.2716500759124756);

            mask_mesh::<false>(
                mask_materials,
                mesh.clone(),
                commands
                    .spawn((
                        Throttle::default(),
                        RotRange{
                            min: Quat::from_xyzw(0.5193636417388916, 0., 0., 0.8545534610748291),
                            max: Quat::from_xyzw(-0.114098, 0., 0., 0.99347),
                        },
                        Name::new("Throttle"),
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material_handle),
                        transform,
                        ChildOf(shell_id),
                    ))
                    .id(),
                commands,
            )
        };

        // player
        let joystick_bundle = {
            let mesh: Handle<Mesh> =
                asset_server.load(&format!("{cf104_asset_path}#Mesh{}/Primitive0", 3));
            let material_handle = materials.add(StandardMaterial::default());

            let mut transform: Transform = Transform::default();

            transform.translation = Vec3 {
                x: 0.0,
                y: 0.5056626200675964,
                z: -1.7769677639007568,
            };
            transform.rotation = Quat::from_xyzw(0.1549355387687683, 0., 0., 0.9879246950149536);
            transform.scale = Vec3::splat(1.2716500759124756);

            mask_mesh::<false>(
                mask_materials,
                mesh.clone(),
                commands
                    .spawn((
                        Joystick::default(),
                        RotRange2D::new(Quat::from_xyzw(0.1549355387687683, 0., 0., 0.9879246950149536), Vec2::new(PI / 12., PI / 14.)),
                        Name::new("Joystick"),
                        Mesh3d(mesh),
                        MeshMaterial3d(material_handle),
                        transform,
                        ChildOf(shell_id),
                    ))
                    .id(),
                commands,
            )
        };
    }
    body_id
}

fn initialize_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mask_materials: Res<MaskMaterials>,
    mut images: ResMut<Assets<Image>>,
) {
    load_cf104::<true>(
        &mut commands,
        &asset_server,
        &mut materials,
        &mask_materials,
        &mut images,
        Some(100.),
    );
}
