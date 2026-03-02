// =============================================================================
// camera/camera.rs — Spherical Orbit Camera
//
// The camera orbits the black hole (at the origin) in spherical coordinates.
// It always looks at the singularity, making orbital navigation intuitive:
//
//   - Mouse drag → change θ (polar) and φ (azimuthal) angles.
//   - Scroll/keys → change r (distance from singularity).
//
// The camera generates rays by constructing a local coordinate frame
// (right, up, forward) from its position, then shooting rays through
// a virtual screen at each pixel.
// =============================================================================

use glam::Vec3A;

use crate::constants::{
    CAMERA_DEFAULT_DISTANCE, CAMERA_DEFAULT_FOV, CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE,
    MOUSE_SENSITIVITY, ZOOM_SPEED,
};
use crate::physics::ray::Ray;

/// An orbital camera that always faces the origin (the singularity).
#[derive(Debug, Clone)]
pub struct Camera {
    /// Radial distance from the singularity.
    pub r: f32,
    /// Polar angle (0 = north pole, π = south pole).
    pub theta: f32,
    /// Azimuthal angle (rotation around the y-axis).
    pub phi: f32,
    /// Vertical field of view in radians.
    pub fov: f32,
}

impl Camera {
    /// Creates a camera at the default orbital position.
    pub fn new() -> Self {
        Self {
            r: CAMERA_DEFAULT_DISTANCE,
            // Start slightly above the equatorial plane for a dramatic view
            // of the accretion disk from an oblique angle.
            theta: std::f32::consts::FRAC_PI_4, // 45° from pole
            phi: 0.0,
            fov: CAMERA_DEFAULT_FOV,
        }
    }

    /// Converts the camera's spherical coordinates (r, θ, φ) to a 3D
    /// Cartesian position.
    ///
    /// Spherical → Cartesian:
    ///   x = r * sin(θ) * cos(φ)
    ///   y = r * cos(θ)           ← y is "up"
    ///   z = r * sin(θ) * sin(φ)
    pub fn position(&self) -> Vec3A {
        let sin_theta = self.theta.sin();
        Vec3A::new(
            self.r * sin_theta * self.phi.cos(),
            self.r * self.theta.cos(),
            self.r * sin_theta * self.phi.sin(),
        )
    }

    /// Generates a ray from the camera through the virtual screen at
    /// normalized coordinates (u, v), where:
    ///   - u ∈ [-1, 1] is horizontal (left to right)
    ///   - v ∈ [-1, 1] is vertical (bottom to top)
    ///
    /// The ray is constructed using a look-at matrix pointing toward the
    /// origin, with "up" defined as the global y-axis.
    pub fn generate_ray(&self, u: f32, v: f32, aspect_ratio: f32) -> Ray {
        let pos = self.position();

        // Forward vector: from camera position toward the origin.
        let forward = (-pos).normalize();

        // Right vector: perpendicular to forward and world up (y-axis).
        // We use the cross product to find it.
        let world_up = Vec3A::Y;
        let right = forward.cross(world_up).normalize();

        // True up vector: perpendicular to both forward and right.
        // This handles the case where the camera isn't exactly level.
        let up = right.cross(forward).normalize();

        // Scale the screen coordinates by the field of view.
        // The tangent of half the FOV gives the screen plane's half-height.
        let half_fov_tan = (self.fov * 0.5).tan();

        // Construct the ray direction in world space by combining the
        // camera's local axes weighted by the screen coordinates.
        let direction = (forward + right * (u * half_fov_tan * aspect_ratio)
            + up * (v * half_fov_tan))
            .normalize();

        Ray {
            position: pos,
            velocity: direction,
        }
    }

    /// Orbits the camera by adjusting θ and φ based on mouse drag deltas.
    /// `dx` and `dy` are pixel distances of the drag.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        // Horizontal drag → azimuthal rotation.
        self.phi += dx * MOUSE_SENSITIVITY;

        // Vertical drag → polar rotation, clamped to avoid gimbal lock
        // at the poles.
        self.theta = (self.theta - dy * MOUSE_SENSITIVITY)
            .clamp(0.05, std::f32::consts::PI - 0.05);
    }

    /// Zooms the camera in or out by adjusting the radial distance.
    /// Positive delta = zoom out, negative delta = zoom in.
    pub fn zoom(&mut self, delta: f32) {
        self.r = (self.r + delta * ZOOM_SPEED).clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The camera at θ=π/2, φ=0 should be on the +x axis.
    #[test]
    fn position_on_x_axis() {
        let cam = Camera {
            r: 10.0,
            theta: std::f32::consts::FRAC_PI_2,
            phi: 0.0,
            fov: CAMERA_DEFAULT_FOV,
        };
        let pos = cam.position();
        assert!((pos.x - 10.0).abs() < 0.01);
        assert!(pos.y.abs() < 0.01);
        assert!(pos.z.abs() < 0.01);
    }

    /// A ray generated at the screen center should point approximately
    /// toward the origin.
    #[test]
    fn center_ray_points_at_origin() {
        let cam = Camera::new();
        let ray = cam.generate_ray(0.0, 0.0, 1.0);

        // The velocity should point roughly opposite to the position.
        let toward_origin = -cam.position().normalize();
        let dot = ray.velocity.dot(toward_origin);
        assert!(
            dot > 0.99,
            "Center ray should point at origin, dot = {dot}"
        );
    }

    /// Corner rays should be offset from the center ray by the FOV.
    #[test]
    fn corner_rays_diverge_from_center() {
        let cam = Camera::new();
        let center = cam.generate_ray(0.0, 0.0, 1.0);
        let corner = cam.generate_ray(1.0, 1.0, 1.0);

        let dot = center.velocity.dot(corner.velocity);
        assert!(
            dot < 0.99 && dot > 0.7,
            "Corner ray should diverge from center: dot = {dot}"
        );
    }

    /// Zoom should clamp at minimum distance.
    #[test]
    fn zoom_clamp_min() {
        let mut cam = Camera::new();
        cam.zoom(-100.0); // Try to zoom way in
        assert!(cam.r >= CAMERA_MIN_DISTANCE);
    }
}
