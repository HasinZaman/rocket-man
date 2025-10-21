use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Query, Res, Single},
    },
    time::Time,
};

use crate::cf104::console::throttle::Throttle;

#[derive(Component, Default)]
#[relationship_target(relationship = MassComponent, linked_spawn)]
pub struct Mass(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Mass)]
pub struct MassComponent(pub Entity);

#[derive(Component)]
pub struct MassData(f32);

impl MassData {
    pub fn new(weight: f32) -> Self {
        MassData(weight)
    }
}

#[derive(Bundle)]
pub struct MassBundle {
    mass_component: MassComponent,
    mass_data: MassData,
}

impl MassBundle {
    pub fn empty_cf_104(parent_mass: Entity) -> Self {
        Self {
            mass_component: MassComponent(parent_mass),
            mass_data: MassData::new(6_300.),
        }
    }
    pub fn nuke(parent_mass: Entity) -> Self {
        Self {
            mass_component: MassComponent(parent_mass),
            mass_data: MassData::new(226.7962),
        }
    }
}

#[derive(Component)]
pub struct ExternalTank;

#[derive(Component)]
pub struct Tank {
    max_capacity: f32,
    pub active: bool,
    pub flow_rates: (f32, f32),
    pub flow_rate: f32,

    target: Option<Entity>, // None -> consumed by Engine, else feeds to another tank
}

#[derive(Bundle)]
pub struct InternalFuelTankBundle {
    mass_component: MassComponent,
    mass_data: MassData,
    tank: Tank,
}

impl InternalFuelTankBundle {
    pub fn new(max_capacity: f32, mass: Entity) -> Self {
        Self {
            mass_component: MassComponent(mass),
            mass_data: MassData::new(max_capacity),
            tank: Tank {
                max_capacity: max_capacity,
                active: false,
                flow_rates: (0.1, 5.0),
                flow_rate: 0.,
                target: None,
            },
        }
    }
}

#[derive(Bundle)]
pub struct ExternalFuelTankBundle {
    mass_component: MassComponent,
    mass_data: MassData,
    tank: Tank,
    external: ExternalTank,
}

impl ExternalFuelTankBundle {
    pub fn new(max_capacity: f32, mass: Entity, internal_tank: Entity) -> Self {
        Self {
            mass_component: MassComponent(mass),
            mass_data: MassData::new(max_capacity),
            tank: Tank {
                max_capacity: max_capacity,
                active: false,
                flow_rates: (0.1, 2.5),
                flow_rate: 0.,
                target: Some(internal_tank),
            },
            external: ExternalTank,
        }
    }
}

pub fn update_fuel_mass_system(
    time: Res<Time>,
    mut internal_tanks: Query<(&Tank, &mut MassData), Without<ExternalTank>>,
    mut external_tanks: Query<(&Tank, &mut MassData), With<ExternalTank>>,
) {
    let dt: f32 = time.delta_secs();

    // empty internal tanks
    for (tank, mut mass_data) in internal_tanks.iter_mut() {
        if !tank.active || tank.flow_rate <= 0.00001 {
            continue;
        }

        let flow: f32 = tank.flow_rate * dt;

        let delta_weight: f32 = mass_data.0.min(flow);
        mass_data.0 -= delta_weight;
    }

    for (tank, mut mass_data) in external_tanks.iter_mut() {
        if !tank.active || tank.flow_rate <= 0.00001 {
            continue;
        }

        let target: Entity = tank.target.unwrap();
        let Ok((internal_tank, mut internal_mass_data)) = internal_tanks.get_mut(target) else {
            continue;
        };

        let flow: f32 = tank.flow_rate * dt;

        let delta_weight: f32 = flow
            .min(mass_data.0)
            .min(internal_tank.max_capacity - internal_mass_data.0);

        internal_mass_data.0 += delta_weight;
        mass_data.0 -= delta_weight;
    }
}

pub fn update_tank_flow_rate(
    throttle: Single<&Throttle>,
    mut internal_tanks: Query<&mut Tank, Without<ExternalTank>>,
    mut external_tanks: Query<&mut Tank, With<ExternalTank>>,
) {
    for mut tank in &mut internal_tanks {
        // ramping
        if throttle.0 > 0.01 {
            tank.active = true;
        }

        tank.flow_rate =
            tank.flow_rates.0 + throttle.0 / 100. * (tank.flow_rates.1 - tank.flow_rates.0);
    }

    for mut tank in &mut external_tanks {
        // place holder (Should be set with a switch)
        if throttle.0 > 0.01 {
            tank.active = true;
        }

        tank.flow_rate =
            tank.flow_rates.0 + throttle.0 / 100. * (tank.flow_rates.1 - tank.flow_rates.0);
    }
}

pub fn get_weight(masses: &Mass, mass_components: &Query<&MassData, With<MassComponent>>) -> f32 {
    let mut mass: f32 = 0.;

    for mass_entity in &masses.0 {
        let Ok(mass_component) = mass_components.get(*mass_entity) else {
            continue;
        };

        mass += mass_component.0;
    }

    mass
}
