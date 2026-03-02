// =============================================================================
// physics/ray.rs — Photon Ray & RK4 Geodesic Integrator
//
// This is the mathematical heart of the raytracer. Each "ray" represents a
// photon being traced *backward* in time from the camera into the scene.
//
// We numerically integrate the photon's trajectory through curved spacetime
// using a Runge-Kutta 4th Order (RK4) method with adaptive step size:
//
//   - Far from the black hole: large steps (spacetime is ~flat).
//   - Near the event horizon:  tiny steps (extreme curvature).
//
// The integration terminates when one of three things happens:
//   1. The photon falls inside the event horizon    → RayResult::EventHorizon
//   2. The photon escapes to the celestial sphere    → RayResult::Escaped
//   3. The photon intersects the accretion disk      → RayResult::Disk
// =============================================================================

use glam::Vec3A;

use crate::constants::{
    DISK_INNER_RADIUS, DISK_OUTER_RADIUS, ESCAPE_RADIUS, EVENT_HORIZON, MAX_INTEGRATION_STEPS,
    STEP_SIZE_BASE, STEP_SIZE_MAX, STEP_SIZE_MIN,
};
use crate::physics::metric::schwarzschild_acceleration;

// ---------------------------------------------------------------------------
// § Ray — the photon state
// ---------------------------------------------------------------------------

/// A single photon being traced through curved spacetime.
///
/// In flat spacetime, a ray is just `origin + t * direction`. In curved
/// spacetime, we track both position and velocity because the velocity
/// *changes* at every step due to gravitational acceleration.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Current 3D Cartesian position of the photon.
    pub position: Vec3A,
    /// Current velocity (direction). The direction curves through spacetime;
    /// the magnitude is nominally 1.0 (speed of light in natural units).
    pub velocity: Vec3A,
}

// ---------------------------------------------------------------------------
// § RayResult — what happened to the photon
// ---------------------------------------------------------------------------

/// The outcome of integrating a ray through the Schwarzschild spacetime.
#[derive(Debug, Clone, Copy)]
pub enum RayResult {
    /// The photon crossed the event horizon and was swallowed.
    /// Contains the last radial distance before crossing, used for
    /// photon ring glow rendering.
    EventHorizon(f32),

    /// The photon escaped to the celestial background sphere.
    /// Contains the final direction vector (used to sample the background).
    Escaped(Vec3A),

    /// The photon hit the accretion disk.
    /// Contains the intersection position in the disk plane.
    Disk(Vec3A),
}

// ---------------------------------------------------------------------------
// § Adaptive Step Size
// ---------------------------------------------------------------------------

/// Computes the RK4 integration step size based on distance from the
/// singularity.
///
/// - At r >> r_s: spacetime is nearly flat → take huge steps (fast).
/// - At r ≈ r_s: extreme curvature → take tiny steps (accurate).
///
/// The "lapse function" (1 - r_s/r) naturally goes to 0 at the horizon
/// and 1 at infinity — perfect for scaling our step size.
#[inline(always)]
fn adaptive_step_size(r: f32) -> f32 {
    let lapse = (1.0 - EVENT_HORIZON / r).max(0.01);
    (STEP_SIZE_BASE * lapse).clamp(STEP_SIZE_MIN, STEP_SIZE_MAX)
}

// ---------------------------------------------------------------------------
// § RK4 Integrator
// ---------------------------------------------------------------------------

/// Integrates a photon ray through the Schwarzschild spacetime using the
/// Runge-Kutta 4th Order method.
///
/// # How RK4 Works
///
/// RK4 is a numerical method for solving ODEs with O(h⁴) accuracy per step.
/// At each step, we compute 4 intermediate slopes:
///
/// ```text
///   k1 = f(y_n)                          — slope at start
///   k2 = f(y_n + h/2 · k1)              — slope at midpoint (using k1)
///   k3 = f(y_n + h/2 · k2)              — slope at midpoint (using k2)
///   k4 = f(y_n + h · k3)                — slope at end (using k3)
///   y_{n+1} = y_n + h/6 · (k1 + 2k2 + 2k3 + k4)
/// ```
///
/// Our ODE system is:
/// ```text
///   d(pos)/dλ = vel                              (position changes by velocity)
///   d(vel)/dλ = schwarzschild_acceleration(pos, vel)  (velocity curves due to GR)
/// ```
///
/// # Arguments
/// * `ray` — The initial photon state (position + velocity).
///
/// # Returns
/// A `RayResult` describing what happened to the photon.
pub fn integrate(ray: &Ray) -> RayResult {
    // Copy the ray state into mutable locals — we'll evolve these.
    let mut pos = ray.position;
    let mut vel = ray.velocity;

    for _step in 0..MAX_INTEGRATION_STEPS {
        let r = pos.length();

        // ----- Termination: Event Horizon -----
        // If the photon has crossed r_s, it's gone forever — return black.
        if r < EVENT_HORIZON {
            return RayResult::EventHorizon(r);
        }

        // ----- Termination: Escape -----
        // If the photon is far enough away, it has escaped the gravitational
        // field and will travel in a straight line to the celestial sphere.
        if r > ESCAPE_RADIUS {
            return RayResult::Escaped(vel.normalize());
        }

        // ----- Adaptive Step Size -----
        let h = adaptive_step_size(r);

        // Save position before the step for disk intersection detection.
        let pos_before = pos;

        // ===== RK4 INTEGRATION STEP =====
        //
        // We're integrating the coupled ODE:
        //   d(pos)/dλ = vel
        //   d(vel)/dλ = schwarzschild_acceleration(pos, vel)
        //
        // The acceleration is the GR null geodesic equation:
        //   a = -3/2 · r_s · |pos × vel|² / |pos|⁵ · pos

        // --- k1: slope at current position ---
        let k1_pos = vel;
        let k1_vel = schwarzschild_acceleration(pos, vel);

        // --- k2: slope at midpoint using k1 ---
        let mid_pos_2 = pos + k1_pos * (h * 0.5);
        let mid_vel_2 = vel + k1_vel * (h * 0.5);
        let k2_pos = mid_vel_2;
        let k2_vel = schwarzschild_acceleration(mid_pos_2, mid_vel_2);

        // --- k3: slope at midpoint using k2 ---
        let mid_pos_3 = pos + k2_pos * (h * 0.5);
        let mid_vel_3 = vel + k2_vel * (h * 0.5);
        let k3_pos = mid_vel_3;
        let k3_vel = schwarzschild_acceleration(mid_pos_3, mid_vel_3);

        // --- k4: slope at endpoint using k3 ---
        let end_pos_4 = pos + k3_pos * h;
        let end_vel_4 = vel + k3_vel * h;
        let k4_pos = end_vel_4;
        let k4_vel = schwarzschild_acceleration(end_pos_4, end_vel_4);

        // --- Weighted average (the "magic" of RK4) ---
        pos += (k1_pos + k2_pos * 2.0 + k3_pos * 2.0 + k4_pos) * (h / 6.0);
        vel += (k1_vel + k2_vel * 2.0 + k3_vel * 2.0 + k4_vel) * (h / 6.0);

        // ----- Accretion Disk Intersection -----
        //
        // The disk lies in the xz-plane (y = 0). We detect a crossing by
        // checking if the photon's y-coordinate changed sign during this
        // step — meaning it passed through the plane.
        let y_before = pos_before.y;
        let y_after = pos.y;

        if y_before * y_after < 0.0 {
            // The ray crossed the y=0 plane during this step.
            // Linearly interpolate to find the approximate crossing point.
            let t = y_before.abs() / (y_before.abs() + y_after.abs());
            let cross_pos = pos_before.lerp(pos, t);

            // Check if the crossing is within the disk's radial bounds.
            let cross_r = (cross_pos.x * cross_pos.x + cross_pos.z * cross_pos.z).sqrt();
            if cross_r >= DISK_INNER_RADIUS && cross_r <= DISK_OUTER_RADIUS {
                return RayResult::Disk(cross_pos);
            }
        }
    }

    // If we've exhausted all steps, treat it as escaped.
    RayResult::Escaped(vel.normalize())
}

// ---------------------------------------------------------------------------
// § Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;

    /// A ray fired far from the black hole should travel approximately
    /// in a straight line (negligible deflection in weak field).
    #[test]
    fn straight_line_far_from_hole() {
        let ray = Ray {
            position: Vec3A::new(40.0, 0.0, 0.0),
            velocity: Vec3A::new(0.0, 0.0, -1.0),
        };

        let result = integrate(&ray);

        match result {
            RayResult::Escaped(dir) => {
                let dot = dir.dot(Vec3A::new(0.0, 0.0, -1.0));
                assert!(
                    dot > 0.95,
                    "Far ray should barely deflect — dot = {dot}"
                );
            }
            other => panic!("Expected Escaped, got {other:?}"),
        }
    }

    /// A ray passing close to the black hole should deflect significantly
    /// due to the 1/r⁵ GR force law.
    #[test]
    fn deflection_near_hole() {
        let ray = Ray {
            position: Vec3A::new(5.0, 0.0, 30.0),
            velocity: Vec3A::new(0.0, 0.0, -1.0),
        };

        let result = integrate(&ray);

        match result {
            RayResult::Escaped(dir) => {
                let dot = dir.dot(Vec3A::new(0.0, 0.0, -1.0));
                assert!(
                    dot < 0.95,
                    "Near ray should deflect — dot = {dot}"
                );
            }
            RayResult::Disk(_) => {} // acceptable for close pass
            RayResult::EventHorizon(_) => {} // acceptable for very close pass
        }
    }

    /// A ray aimed directly at the singularity should fall into the
    /// event horizon (radial infall = straight line through the origin).
    #[test]
    fn event_horizon_termination() {
        let ray = Ray {
            position: Vec3A::new(10.0, 0.0, 0.0),
            velocity: Vec3A::new(-1.0, 0.0, 0.0),
        };

        let result = integrate(&ray);

        assert!(
            matches!(result, RayResult::EventHorizon(_)),
            "Radial ray should hit event horizon, got {result:?}"
        );
    }

    /// The adaptive step size should be small near the horizon and
    /// large far away.
    #[test]
    fn step_size_adapts() {
        let h_near = adaptive_step_size(EVENT_HORIZON * 1.1);
        let h_far = adaptive_step_size(EVENT_HORIZON * 20.0);

        assert!(
            h_near < h_far,
            "h_near={h_near} should be < h_far={h_far}"
        );
        assert!(h_near >= STEP_SIZE_MIN);
        assert!(h_far <= STEP_SIZE_MAX);
    }

    /// A ray in the disk plane should intersect the disk when aimed
    /// through the valid radial range.
    #[test]
    fn disk_intersection() {
        // Start above the disk, aimed downward through it.
        let ray = Ray {
            position: Vec3A::new(DISK_INNER_RADIUS + 2.0, 5.0, 0.0),
            velocity: Vec3A::new(0.0, -1.0, 0.0),
        };

        let result = integrate(&ray);

        assert!(
            matches!(result, RayResult::Disk(_)),
            "Ray through disk bounds should hit disk, got {result:?}"
        );
    }
}
