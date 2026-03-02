// =============================================================================
// render/scene.rs — Scene Description: Accretion Disk & Celestial Background
//
// This module provides the "shading" functions that determine what color
// a ray should produce once it reaches its destination:
//
//   - If it escapes → sample the celestial background (procedural grid).
//   - If it hits the disk → compute a temperature-based color gradient.
//   - If it hits the event horizon → pure black.
//
// The celestial background uses a procedural checkerboard pattern on a
// sphere so that gravitational lensing is clearly visible as a warping
// of the grid lines.
// =============================================================================

use glam::Vec3A;
use ratatui::style::Color;

use crate::constants::{
    COLOR_DISK_COOL, COLOR_DISK_HOT, COLOR_DISK_MID, COLOR_GRID_PRIMARY, COLOR_GRID_SECONDARY,
    COLOR_STAR, DISK_INNER_RADIUS, DISK_OUTER_RADIUS,
};

// ---------------------------------------------------------------------------
// § Celestial Background
// ---------------------------------------------------------------------------

/// Samples the celestial sphere background for a given ray direction.
///
/// We project the direction onto spherical coordinates and create a
/// checkerboard pattern. This makes gravitational lensing dramatically
/// visible — the normally-straight grid lines will bend and warp around
/// the black hole.
///
/// Returns the color for this direction on the celestial sphere.
pub fn celestial_background(dir: Vec3A) -> Color {
    // Convert direction to spherical coordinates (θ, φ).
    // θ = polar angle from +y axis, φ = azimuthal angle in xz-plane.
    let theta = dir.y.acos();
    let phi = dir.z.atan2(dir.x);

    // Create a checkerboard by dividing the sphere into grid squares.
    // The grid density controls how many squares are visible.
    let grid_density = 12.0;
    let grid_u = (theta * grid_density / std::f32::consts::PI).floor() as i32;
    let grid_v = (phi * grid_density / std::f32::consts::PI).floor() as i32;

    // Alternate between primary and secondary colors.
    let is_primary = (grid_u + grid_v) % 2 == 0;

    // Add scattered "stars" using a pseudo-random hash of the grid cell.
    // This gives a few bright dots on the celestial sphere.
    let hash = ((grid_u.wrapping_mul(73856093)) ^ (grid_v.wrapping_mul(19349663))) as u32;
    let is_star = hash % 47 == 0;

    if is_star {
        COLOR_STAR
    } else if is_primary {
        COLOR_GRID_PRIMARY
    } else {
        COLOR_GRID_SECONDARY
    }
}

// ---------------------------------------------------------------------------
// § Accretion Disk Coloring
// ---------------------------------------------------------------------------

/// Computes the color for a point on the accretion disk.
///
/// The disk radiates due to friction heating of infalling matter. Inner
/// regions are hotter (whiter) and outer regions are cooler (redder).
/// This creates the characteristic gradient from white-hot to deep red.
///
/// # Arguments
/// * `pos` — The 3D position where the ray intersected the disk plane.
///
/// # Returns
/// The foreground color for this disk position.
pub fn accretion_disk_color(pos: Vec3A) -> Color {
    // Compute the radial distance from the disk center (in the xz-plane).
    let r = (pos.x * pos.x + pos.z * pos.z).sqrt();

    // Normalize radius to [0, 1] range across the disk width.
    // t = 0 at inner edge (hottest), t = 1 at outer edge (coolest).
    let t = ((r - DISK_INNER_RADIUS) / (DISK_OUTER_RADIUS - DISK_INNER_RADIUS)).clamp(0.0, 1.0);

    // Temperature-based color: interpolate from hot → mid → cool.
    let color = if t < 0.3 {
        // Inner disk: white-hot → orange.
        let local_t = t / 0.3;
        lerp_color(COLOR_DISK_HOT, COLOR_DISK_MID, local_t)
    } else {
        // Outer disk: orange → deep red.
        let local_t = (t - 0.3) / 0.7;
        lerp_color(COLOR_DISK_MID, COLOR_DISK_COOL, local_t)
    };

    // Add some radial "texture" — spiral structure in the disk.
    let angle = pos.z.atan2(pos.x);
    let spiral = ((angle * 3.0 + r * 0.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    lerp_color(color, brighten(color, 0.3), spiral * 0.3)
}


// ---------------------------------------------------------------------------
// § Color Utilities
// ---------------------------------------------------------------------------

/// Linearly interpolates between two ratatui RGB colors.
///
/// `t = 0.0` returns `a`, `t = 1.0` returns `b`.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    // Extract RGB components — we only work with Rgb colors here.
    let (ar, ag, ab) = match a {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    };
    let (br, bg, bb) = match b {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    };

    Color::Rgb(
        lerp_u8(ar, br, t),
        lerp_u8(ag, bg, t),
        lerp_u8(ab, bb, t),
    )
}

/// Linearly interpolates between two u8 values.
#[inline(always)]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

/// Brightens a color by blending it toward white.
fn brighten(color: Color, amount: f32) -> Color {
    lerp_color(color, Color::Rgb(255, 255, 255), amount.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Celestial background should return consistent results for a given
    /// direction.
    #[test]
    fn background_deterministic() {
        let dir = Vec3A::new(0.3, 0.5, 0.8).normalize();
        let col1 = celestial_background(dir);
        let col2 = celestial_background(dir);
        assert_eq!(col1, col2);
    }

    /// Disk color at inner edge should be hotter (brighter) than outer.
    #[test]
    fn disk_inner_hotter_than_outer() {
        let inner = Vec3A::new(DISK_INNER_RADIUS + 0.1, 0.0, 0.0);
        let outer = Vec3A::new(DISK_OUTER_RADIUS - 0.1, 0.0, 0.0);

        let color_inner = accretion_disk_color(inner);
        let color_outer = accretion_disk_color(outer);

        // Extract red channel — inner should be brighter.
        let inner_r = match color_inner {
            Color::Rgb(r, _, _) => r,
            _ => 0,
        };
        let outer_r = match color_outer {
            Color::Rgb(r, _, _) => r,
            _ => 0,
        };

        assert!(
            inner_r >= outer_r,
            "Inner disk should be at least as bright: inner_r={inner_r}, outer_r={outer_r}"
        );
    }
}
