// =============================================================================
// physics/metric.rs — Schwarzschild Metric & Geodesic Acceleration
//
// This module computes the gravitational acceleration experienced by a photon
// in the Schwarzschild spacetime.
//
// ## The Physics
//
// Photons travel along **null geodesics** — paths where ds² = 0. Unlike
// massive particles, photons do NOT experience "Newtonian gravity" in the
// traditional sense. Their trajectories are determined entirely by the
// curvature of spacetime.
//
// Starting from the Schwarzschild metric in natural units (G = c = M = 1,
// so r_s = 2M = 2):
//
//   ds² = -(1 - r_s/r)dt² + (1 - r_s/r)⁻¹dr² + r²dΩ²
//
// The photon orbit equation (Binet equation for null geodesics) is:
//
//   d²u/dφ² + u = 3M·u²    where u = 1/r, M = r_s/2
//
// The "u" term on the left represents flat-space straight-line propagation.
// The "3Mu²" on the right is the **General Relativistic correction** — this
// is the ONLY force acting on photons and is what causes gravitational
// lensing, the photon sphere, and the bending of light.
//
// Converting the Binet equation to 3D Cartesian acceleration:
//
//   **a = -3/2 · r_s · h² / r⁵ · pos**
//
// where:
//   - h = |pos × vel| is the specific angular momentum (conserved in
//     Schwarzschild spacetime due to spherical symmetry)
//   - r = |pos| is the Schwarzschild radial coordinate
//   - pos is the position vector
//
// Key features of this force law:
//   - Proportional to h² → radially-infalling photons (h=0) travel straight
//     lines, which is physically correct.
//   - Falls off as 1/r⁵ → the GR effect is negligible far from the hole
//     but dominates near the photon sphere.
//   - Produces an unstable circular orbit (the photon sphere) where the
//     gravitational pull exactly balances the centrifugal tendency.
//   - Correctly reproduces the weak-field deflection angle Δφ = 2r_s/b
//     for photons passing at large impact parameter b >> r_s.
//
// ## Note on Coordinate Approximation
//
// This approach works by integrating photon trajectories in "flat" Cartesian
// coordinates using the Schwarzschild radial coordinate as if it were a
// Euclidean distance. This is standard practice for black hole raytracers
// (see Rantonels, Riazuelo, et al.) and produces visually faithful results
// — the event horizon, photon ring, Einstein ring, and lensed accretion
// disk all appear correctly.
// =============================================================================

use glam::Vec3A;

use crate::constants::SCHWARZSCHILD_RADIUS;

/// Computes the gravitational acceleration on a photon at position `pos`
/// traveling with velocity `vel` in the Schwarzschild spacetime.
///
/// This is the single GR acceleration term for null geodesics derived from
/// the Binet orbit equation. There is no separate "Newtonian" term — photon
/// trajectories are determined entirely by spacetime curvature.
///
/// # Formula
/// ```text
///   a = -3/2 · r_s · h² / r⁵ · pos
/// ```
/// where h² = |pos × vel|² (squared angular momentum).
///
/// # Arguments
/// * `pos` — Current 3D position of the photon (Vec3A for SIMD alignment).
/// * `vel` — Current velocity vector (direction of travel).
///
/// # Returns
/// The acceleration vector pointing toward the singularity.
pub fn schwarzschild_acceleration(pos: Vec3A, vel: Vec3A) -> Vec3A {
    // Distance from the singularity (origin).
    let r_sq = pos.dot(pos);
    let r = r_sq.sqrt();

    // Guard: avoid division by zero near the singularity.
    // The ray should have been terminated before reaching here.
    if r < 0.01 {
        return Vec3A::ZERO;
    }

    // Angular momentum: h = pos × vel.
    // Its magnitude squared determines how strongly the ray curves.
    // A purely radial ray (h=0) experiences zero acceleration — it
    // continues straight toward/away from the singularity.
    let h = pos.cross(vel);
    let h_sq = h.dot(h);

    // The GR acceleration for null geodesics:
    //   a = -3/2 · r_s · h² / r⁵ · pos
    //
    // Breaking down the denominator: r⁵ = r² · r² · r = r_sq · r_sq · r
    let r_fifth = r_sq * r_sq * r;
    let coefficient = -1.5 * SCHWARZSCHILD_RADIUS * h_sq / r_fifth;

    coefficient * pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3A;

    /// A radially infalling photon (h = 0) should experience zero
    /// acceleration — it travels in a straight line.
    #[test]
    fn radial_infall_zero_acceleration() {
        let pos = Vec3A::new(10.0, 0.0, 0.0);
        let vel = Vec3A::new(-1.0, 0.0, 0.0); // purely radial
        let acc = schwarzschild_acceleration(pos, vel);

        // h = pos × vel = (10,0,0) × (-1,0,0) = (0,0,0)
        // So acceleration should be zero.
        assert!(
            acc.length() < 1e-10,
            "Radial photon should have zero acceleration, got {acc:?}"
        );
    }

    /// A photon with angular momentum should experience inward acceleration.
    #[test]
    fn tangential_photon_accelerates_inward() {
        let pos = Vec3A::new(10.0, 0.0, 0.0);
        let vel = Vec3A::new(0.0, 1.0, 0.0); // tangential
        let acc = schwarzschild_acceleration(pos, vel);

        // Acceleration should point inward (negative x direction).
        assert!(
            acc.x < 0.0,
            "Tangential photon should accelerate inward: {acc:?}"
        );
        // And should be purely radial (y and z ~0).
        assert!(acc.y.abs() < 1e-10);
        assert!(acc.z.abs() < 1e-10);
    }

    /// Acceleration should be stronger closer to the black hole
    /// (1/r⁵ force law).
    #[test]
    fn acceleration_stronger_closer() {
        let vel = Vec3A::new(0.0, 1.0, 0.0);

        let acc_far = schwarzschild_acceleration(Vec3A::new(20.0, 0.0, 0.0), vel);
        let acc_near = schwarzschild_acceleration(Vec3A::new(5.0, 0.0, 0.0), vel);

        assert!(
            acc_near.length() > acc_far.length(),
            "Closer photon should feel stronger acceleration"
        );
    }

    /// Verify the 1/r⁵ scaling law quantitatively.
    #[test]
    fn inverse_fifth_power_scaling() {
        let vel = Vec3A::new(0.0, 1.0, 0.0);
        let r1 = 10.0;
        let r2 = 20.0;

        let a1 = schwarzschild_acceleration(Vec3A::new(r1, 0.0, 0.0), vel).length();
        let a2 = schwarzschild_acceleration(Vec3A::new(r2, 0.0, 0.0), vel).length();

        // a_vec = coeff * pos, so |a| = coeff * r = (h²/r⁵) * r = h²/r⁴.
        // With perpendicular unit velocity: h = r, so h² = r², giving
        // |a| ∝ r²/r⁴ = 1/r².  Therefore ratio = (r2/r1)².
        let ratio = a1 / a2;
        let expected_ratio = (r2 / r1).powi(2); // 2² = 4
        assert!(
            (ratio - expected_ratio).abs() / expected_ratio < 0.01,
            "Expected ratio {expected_ratio}, got {ratio}"
        );
    }
}
