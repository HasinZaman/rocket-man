use std::array;

use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Query, Res},
    },
    math::{Vec3, primitives::Triangle3d},
    mesh::{Mesh, Mesh3d},
    transform::components::GlobalTransform,
};
use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::single::SingleFloatOverlay,
};

use crate::projectile::{
    Velocity,
    util::{air_density, speed_of_sound},
};

#[derive(Component)]
#[relationship(relationship_target = Drag)]
pub struct DragTarget(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = DragTarget, linked_spawn)]
pub struct Drag(Vec<Entity>);

impl Drag {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

#[derive(Debug)]
pub enum AreaCache {
    None,
    // Computer{
    //  angle: f32,
    //  triangles: Vec<Vec<[f32; 2]>>,
    // }
    Final { area: f32 },
}

impl Default for AreaCache {
    fn default() -> Self {
        AreaCache::None
    }
}

#[derive(Component, Default, Debug)]
pub struct CrossSectionArea {
    cache: AreaCache,
    pub area: f32,
}

pub fn update_cross_section(
    meshes: Res<Assets<Mesh>>,

    cross_section_components: Query<(&Mesh3d, &GlobalTransform), With<DragTarget>>,

    cross_section_query: Query<(&mut CrossSectionArea, &Drag, &Velocity, &GlobalTransform)>,
) {
    for (mut drag_data, Drag(children), Velocity(velocity), drag_transform) in cross_section_query {
        if velocity.length().abs() <= 0.0001 {
            drag_data.cache = AreaCache::None;
            drag_data.area = 0.;
            continue;
        }

        let angle = velocity.normalize().dot(*drag_transform.forward());

        let max_angle: f32 = 1. - 5.0_f32.to_radians().cos();

        match (angle >= max_angle, &mut drag_data.cache) {
            (false, _) | (true, AreaCache::None) => {
                // compute new cache
                let (u, v) = {
                    match velocity.dot(Vec3::X).abs() >= 1. - 0.00001 {
                        true => {
                            let u = velocity.cross(Vec3::Y).normalize();
                            let v = velocity.cross(u).normalize();

                            (u, v)
                        }
                        false => {
                            let u = velocity.cross(Vec3::X).normalize();
                            let v = velocity.cross(u).normalize();

                            (u, v)
                        }
                    }
                };
                let mut triangles: Vec<[Vec3; 3]> = vec![];
                for child in children {
                    let Ok((mesh_handle, mesh_transform)) = cross_section_components.get(*child)
                    else {
                        println!("No mesh handle");
                        continue;
                    };

                    let Some(mesh) = meshes.get(mesh_handle.id()) else {
                        println!("Mesh not loaded");
                        continue;
                    };

                    let Ok(iter) = mesh.triangles() else {
                        println!("Failed to get triangles");
                        continue;
                    };

                    let translation = mesh_transform.translation();
                    let scale = mesh_transform.scale();
                    let rotation = mesh_transform.rotation();

                    let global_translation = drag_transform.translation();

                    for Triangle3d { vertices } in iter {
                        let mut iter = vertices.into_iter().map(|vec| {
                            rotation * Vec3::new(vec.x * scale.x, vec.y * scale.y, vec.z * scale.z)
                                + translation
                                - global_translation
                        });

                        let vertices: [Vec3; 3] = array::from_fn(|_| iter.next().unwrap());

                        triangles.push(vertices);
                    }
                }
                let triangles: Vec<Vec<[f32; 2]>> = triangles
                    .into_iter()
                    .map(|triangles| {
                        triangles
                            .into_iter()
                            .map(|vec: Vec3| {
                                [
                                    {
                                        let vec = vec.project_onto(u);
                                        let length = vec.length();

                                        match (length * u - vec).length() <= 0.0001 {
                                            true => length,
                                            false => -length,
                                        }
                                    },
                                    {
                                        let vec = vec.project_onto(v);
                                        let length = vec.length();

                                        match (length * v - vec).length() <= 0.0001 {
                                            true => length,
                                            false => -length,
                                        }
                                    },
                                ]
                            })
                            .collect::<Vec<[f32; 2]>>()
                    })
                    .collect();
                let mut cross_section: Vec<Vec<Vec<[f32; 2]>>> = vec![];
                for triangle in triangles {
                    cross_section =
                        cross_section.overlay(&triangle, OverlayRule::Union, FillRule::EvenOdd);
                    break;
                }
                let mut total_area = 0.0;
                for shape in &cross_section {
                    for contour in shape {
                        let mut area = 0.0;
                        for i in 0..contour.len() {
                            let j = (i + 1) % contour.len();
                            area += contour[i][0] * contour[j][1] - contour[j][0] * contour[i][1];
                        }
                        area *= 0.5;
                        total_area += area;
                    }
                }
                total_area *= 148.6884931;
                println!("{:?}", cross_section);

                drag_data.cache = AreaCache::Final { area: total_area };

                drag_data.area = total_area;
            }
            (true, AreaCache::Final { area }) => {
                // no new cache
                drag_data.area = *area;
            }
        };
        println!("{:?}", drag_data.area);
    }
}

const C_D_SUBSONIC: f32 = 0.02;
const C_D_TRANSONIC_SPIKE: f32 = 0.10;
const C_D_SUPERSONIC_BASE: f32 = 0.04;
pub fn drag_force(
    cross_section_area: f32,
    velocity: &Velocity,
    temperature: f32,
    air_pressure: f32,
) -> Vec3 {
    let air_density: f32 = air_density(air_pressure, temperature);

    let speed = velocity.0.length();
    if speed < 1e-3 {
        return Vec3::ZERO;
    }
    let velocity_dir = velocity.0.normalize();

    let speed_of_sound = speed_of_sound(temperature);
    let mach_number = speed / speed_of_sound;

    let drag_coefficient = if mach_number < 0.8 {
        C_D_SUBSONIC
    } else if mach_number < 1.2 {
        let transition_factor = (mach_number - 0.8) / 0.4;
        C_D_SUBSONIC + transition_factor * (C_D_TRANSONIC_SPIKE - C_D_SUBSONIC)
    } else {
        C_D_SUPERSONIC_BASE + 0.02 * (mach_number - 1.2)
    };

    let dynamic_pressure = 0.5 * air_density * speed * speed;
    let drag_magnitude = dynamic_pressure * drag_coefficient * cross_section_area;

    -drag_magnitude * velocity_dir
}
