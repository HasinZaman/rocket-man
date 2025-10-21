use bevy::math::Vec3;

const CF104_WING_AREA: f32 = 18.22; // m^2
pub const CF104_CL0: f32 = 0.8;
pub const CF104_CL_ALPHA: f32 = 5.7;
pub const CF104_STALL_ALPHA: f32 = 15.0_f32.to_radians();
pub const CF104_INCIDENT_OFFSET: f32 = -2.0_f32.to_radians();

#[inline]
fn angle_of_attack(forward: &Vec3, velocity: &Vec3, up: &Vec3) -> f32 {
    let rel_air = -velocity;
    let vel_proj = rel_air - rel_air.dot(*up) * up;
    let angle = forward.angle_between(vel_proj);
    let sign = forward.cross(vel_proj).dot(*up).signum();

    angle * sign + CF104_INCIDENT_OFFSET
}

#[inline]
fn cl(alpha: f32) -> f32 {
    if alpha.abs() < CF104_STALL_ALPHA {
        CF104_CL0 + CF104_CL_ALPHA * alpha
    } else {
        CF104_CL0 + CF104_CL_ALPHA * CF104_STALL_ALPHA * (alpha / CF104_STALL_ALPHA).cos()
    }
}

pub fn lift_force(forward: &Vec3, velocity: &Vec3, up: &Vec3, rho: f32) -> Vec3 {
    let vel_proj: Vec3 = velocity.project_onto(*forward);

    let v_mag: f32 = vel_proj.length();

    if v_mag < 1e-3 {
        return Vec3::ZERO;
    }

    let alpha = angle_of_attack(forward, velocity, up);
    let cl = cl(alpha);
    let lift_mag = 0.5 * rho * v_mag * v_mag * CF104_WING_AREA * cl;

    // println!("v_mag:{v_mag:?}\tAoA:{alpha:?}\tcl{cl:?}\tlift:{lift_mag:?}");

    up.normalize() * lift_mag
}
