use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::{
    app::{Plugin, Startup, Update}, asset::{AssetServer, Assets, Handle}, camera::visibility::Visibility, color::{Color, LinearRgba}, ecs::{
        component::Component, entity::Entity, hierarchy::ChildOf, message::{Message, MessageReader, MessageWriter}, query::With, resource::Resource, system::{Commands, Query, Res, ResMut}
    }, light::{NotShadowCaster, NotShadowReceiver}, math::{primitives::{Cone, Cylinder, Sphere}, EulerRot, Quat, Vec3}, mesh::{Mesh, Mesh3d}, pbr::{Material, MeshMaterial3d, StandardMaterial}, render::alpha::AlphaMode, transform::components::{GlobalTransform, Transform}, utils::default
};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    cf104::{MovePlayerMessage, Plane}, player::Player, world::{ground::{GroundChunk, LandCover}, props::lahr::spawn_lahr_airbase}
};

mod lahr;
mod trees;

const LAHRS_AIRBASE: (i32, i32) = (0, 0);

#[derive(Component)]
pub struct Prop;

#[derive(Message, Clone, Debug)]
pub struct SpawnPropsMessage {
    pub entity: Entity,
    pub u: Vec3,
    pub v: Vec3,
    pub chunk_id: (i32, i32),

    pub width: f32,
    pub length: f32,

    pub heights: [f32; 4],
    pub land_use: [LandCover; 4],
}

#[derive(Resource, Default)]
struct SpawnPropsMessageStack(Vec<SpawnPropsMessage>);

fn spawn_props(
    mut commands: Commands,

    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut spawn_prop_reader: MessageReader<SpawnPropsMessage>,
    mut message_stack: ResMut<SpawnPropsMessageStack>,
    mut move_player: MessageWriter<MovePlayerMessage>,
    
    tree_meshes: Res<TreeMeshes>,
) {
    for msg in spawn_prop_reader.read() {
        message_stack.0.push(msg.clone());
    };

    if message_stack.0.len() == 0 {
        return;
    }
    // println!("Chunks left: {}", message_stack.0.len());
    
    let mut i1 = 0;
    const LAHRS_AIRBASE: (i32, i32) = (0, 0);
    while let Some(msg) = message_stack.0.pop() {
        let SpawnPropsMessage {
            entity,
            u,
            v,
            chunk_id,
            land_use,
            width,
            length,
            heights,
        } = msg;

        println!(
            "chunk_data: {:?}",
            (
                entity,
                u,
                v,
                chunk_id,
                land_use,
                width,
                length,
                heights,
            )
        );
        match chunk_id {
            LAHRS_AIRBASE => {
                // lahrs airbase
                let mut transform = Transform::default();

                // transform.rotate_local_y(FRAC_PI_2);
                // transform.translation = Vec3 { x: 10., y: 5., z: 10. };
                // transform.scale = Vec3::splat(-1.);
                println!("{heights:?}");

                // transform.rotation = Quat::from_mat3(&Mat3::from_cols(u.normalize(), Vec3::Y, v.normalize()));

                let lahrs_air_base =
                    spawn_lahr_airbase(&mut commands, &asset_server, &mut materials, transform);

                commands.entity(lahrs_air_base).insert((ChildOf(entity),));

                move_player.write(MovePlayerMessage(
                    Vec3::new(39.25777053833008, 263.7556, 169.4016571044922),
                    Quat::from_euler(EulerRot::XYZ, 0., 3. * FRAC_PI_2, 0.),
                ));
            }

            (x, y) => {
                i1-=1;
                // {
                //     let tree_activation: Vec<bool> = land_use.iter()
                //         .map(|land_cover| 
                //             land_cover == &LandCover::ConiferousForest ||
                //             land_cover == &LandCover::BroadLeavedForest ||
                //             land_cover == &LandCover::MixedForest
                //         )
                //         .collect();

                //     spawn_trees_in_area(
                //         entity,
                //         &mut commands,
                //         &tree_meshes,
                //         x, y,
                //         &heights,
                //         tree_activation.as_slice(),
                //         width
                //     );
                // }
            }
        }

        if i1 >= 128 {
            break;
        }

        i1+=1;
    }
}

fn boundary(
    activation: &[bool],
    heights: &[f32],
    size: f32,
) -> Vec<[Vec3; 3]> {
    let state =
        (activation[0] as u8) | ((activation[1] as u8) << 1) | ((activation[2] as u8) << 2) | ((activation[3] as u8) << 3);

    // Base corner positions
    let p0 = Vec3::new(0.0, heights[0], 0.0);
    let p1 = Vec3::new(size, heights[1], 0.0);
    let p2 = Vec3::new(size, heights[2], size);
    let p3 = Vec3::new(0.0, heights[3], size);

    // Edge midpoints (interpolated)
    let e0 = Vec3::new(size * 0.5, (heights[0] + heights[1]) * 0.5, 0.0);      // bottom
    let e1 = Vec3::new(size, (heights[1] + heights[2]) * 0.5, size * 0.5);    // right
    let e2 = Vec3::new(size * 0.5, (heights[2] + heights[3]) * 0.5, size);    // top
    let e3 = Vec3::new(0.0, (heights[3] + heights[0]) * 0.5, size * 0.5);     // left


    //
    // p3, e2, p2
    // e3,   , e1
    // p0, e0, p1
    //

    match state {
        0 => vec![],
        1 => vec![[p0, e0, e3]],
        2 => vec![[e0, p1, e1]],
        3 => vec![[e3, p0, p1], [p1, e1, e3]],
        4 => vec![[e1, p2, e2]],
        5 => vec![[p0, e0, e3], [e1, p2, e2]],
        6 => vec![[e0, p1, p2], [p2, e2, e0]],
        7 => vec![[p0, e0, e3], [e0, p1, e1], [e1, p2, e2], [e2, e3, e0], [e2, e0, e1]],
        8 => vec![[e3, e2, p3]],
        9 => vec![[e2, p0, p3], [p0, e0, e2]],
        10 => vec![[e3, e2, p3], [e0, p1, e1]],
        11 => vec![[p0, e0, e3], [e0, p1, e1], [e3, e2, p3], [e2, e3, e0], [e2, e0, e1]],
        12 => vec![[p3, e3, e1], [e1, p3, p2]],
        13 => vec![[p0, e0, e3], [e1, p2, e2], [e3, e2, p3], [e2, e3, e0], [e2, e0, e1]],
        14 => vec![[e0, p1, e1], [e1, p2, e2], [e3, e2, p3], [e2, e3, e0], [e2, e0, e1]],
        15 => vec![[p3, p0, p1], [p3, p1, p2]],
        _ => panic!("Invalid state")
    }
}


fn spawn_trees_in_area(
    parent_id: Entity,
    commands: &mut Commands,
    trees: &Res<TreeMeshes>,
    base_x: i32,
    base_z: i32,
    heights: &[f32],
    activation: &[bool],
    size: f32,
) {
    let triangle_bounds: Vec<[Vec3; 3]> = boundary(activation, heights, size);
    let min_height: f32 = heights[0].min(heights[1]).min(heights[2]).min(heights[3]);

    if triangle_bounds.is_empty() {
        return;
    }

    let seed: i64 = ((base_x as i64) << 32) ^ (base_z as i64);
    let mut rng: StdRng = StdRng::seed_from_u64(seed as u64);

    const TREE_RADIUS: f32 = 3.;
    let mut placed_trees: Vec<(Vec3, f32)> = Vec::new();
    let mut consecutive_failures = 0;

    // println!("Bounds: {triangle_bounds:?}");
    const MAX_FAILURES: i32 = 3;
    while consecutive_failures < MAX_FAILURES {
        let tri: &[Vec3; 3] = &triangle_bounds[rng.random_range(0..triangle_bounds.len())];
       
        // Compute 3D position in world space
        let proposed_pos: Vec3 = random_point_in_triangle(tri, &mut rng);
        // println!("Proposed pos: {proposed_pos:?}");

        // Collision test
        let mut collides = false;
        for (pos, radius) in &placed_trees {
            if proposed_pos.distance(*pos) < (radius + TREE_RADIUS) {
                collides = true;
                break;
            }
        }

        if collides {
            consecutive_failures += 1;
            continue;
        }

        consecutive_failures = 0;
        placed_trees.push((
            proposed_pos,
            TREE_RADIUS,
        ));

        // Spawn the tree
        let mut transform: Transform = Transform::from_translation(proposed_pos-Vec3{x: 0., y: min_height, z: 0.})
                .with_scale(
                    Vec3 {
                        x: (trees.coniferous_scale[0].1.x - trees.coniferous_scale[0].0.x) * rng.random::<f32>() + trees.coniferous_scale[0].0.x,
                        y: (trees.coniferous_scale[0].1.y - trees.coniferous_scale[0].0.y) * rng.random::<f32>() + trees.coniferous_scale[0].0.y,
                        z: (trees.coniferous_scale[0].1.z - trees.coniferous_scale[0].0.z) * rng.random::<f32>() + trees.coniferous_scale[0].0.z,
                    }
                    .clone())
                .with_rotation(trees.coniferous_rotation[0]);

        transform.rotate_y(rng.random_range(0.0..TAU));
        commands.spawn((
            Visibility::Inherited,
            Mesh3d(trees.coniferous_mesh[0].clone()),
            MeshMaterial3d(trees.coniferous_materials[0].clone()),
            transform,
            Prop,
            NotShadowCaster,
            NotShadowReceiver,
            ChildOf(parent_id)
        ));
    }

    // println!("{placed_trees:?}");
}
pub fn random_point_in_triangle<R: Rng>(verts: &[Vec3; 3], rng: &mut R) -> Vec3 {
    // Generate random barycentric coordinates
    let mut u: f32 = rng.random();
    let mut v: f32 = rng.random();

    if u + v > 1.0 {
        u = 1.0 - u;
        v = 1.0 - v;
    }

    let a = verts[0];
    let b = verts[1];
    let c = verts[2];

    // Barycentric interpolation
    a + (b - a) * u + (c - a) * v
}

#[derive(Resource, Default)]
struct TreeMeshes{
    coniferous_mesh: Vec<Handle<Mesh>>,
    coniferous_rotation: Vec<Quat>,
    coniferous_scale: Vec<(Vec3, Vec3)>,
    coniferous_materials: Vec<Handle<StandardMaterial>>,

    broad_leaved_mesh: Vec<Handle<Mesh>>,
    broad_leaved_materials: Vec<Handle<StandardMaterial>>,
}

fn init_tree_meshes(
    asset_server: Res<AssetServer>,
    mut tree_meshes: ResMut<TreeMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    {
        let mesh: Handle<Mesh> = asset_server.load("world\\pine_trees\\large_pine_tree\\tree.gltf#Mesh0/Primitive0");

        let material: Handle<StandardMaterial> = materials.add(StandardMaterial {
            perceptual_roughness: 0.8553509712219238,
            metallic: 0.0,
            base_color_texture: Some(asset_server.load("world\\pine_trees\\large_pine_tree\\Image_13.png")),
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            // unlit: true,
            ..default()
        });
        tree_meshes.coniferous_mesh.push(mesh);
        tree_meshes.coniferous_rotation.push(Quat::from_xyzw(-0.70710688829422,0., 0., 0.7071066498756409));
        tree_meshes.coniferous_scale.push((Vec3::splat(0.75), Vec3::splat(1.25)));
        tree_meshes.coniferous_materials.push(material);
    }

    // Broad-leaved: sphere canopy + trunk combo meshes
    for i in 0..3 {
        let trunk: Handle<Mesh> = meshes.add(Mesh::from(Cylinder {
            radius: 0.3,
            half_height: 3.0,
            ..default()
        }));
        let canopy = meshes.add(Mesh::from(Sphere {
            radius: 2.0 + i as f32 * 0.2,
        }));

        let material: Handle<StandardMaterial> = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6 + 0.05 * i as f32, 0.2),
            ..default()
        });

        tree_meshes.broad_leaved_mesh.push(trunk);
        tree_meshes.broad_leaved_materials.push(material);
    }
}

fn update_prop_visibility(
    plane_query: Query<&GlobalTransform, (With<Plane>, With<Player>)>,
    mut prop_query: Query<(&GlobalTransform, &mut Visibility), With<Prop>>
) {
    let Ok(plane_transform) = plane_query.single() else { return; };

    let plane_pos = plane_transform.translation();
    
    const MAX_RENDER_DISTANCE: f32 = 1_000.0;
    for (prop_transform, mut visibility) in prop_query.iter_mut() {
        let prop_pos = prop_transform.translation();
        let distance = plane_pos.distance(prop_pos);

        if distance <= MAX_RENDER_DISTANCE {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub struct PropPlugin;
impl Plugin for PropPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<SpawnPropsMessage>()
            .init_resource::<TreeMeshes>()
            .init_resource::<SpawnPropsMessageStack>()
            .add_systems(Startup, init_tree_meshes)
            .add_systems(Update, (spawn_props, update_prop_visibility));
    }
}
