// =============================================================================
// render/renderer.rs — Parallel Raymarching Renderer
//
// This module ties the physics engine to the frame buffer. For each cell in
// the terminal grid, we:
//
//   1. Generate two rays (one per vertical sub-pixel) from the camera.
//   2. Integrate each ray through the Schwarzschild spacetime.
//   3. Shade the result based on what the ray hit (background, disk, or void).
//   4. Write the colors into the frame buffer.
//
// The rendering is parallelized using rayon's `par_iter_mut()`. Each thread
// processes a slice of cells independently — the closure is completely
// stateless and allocation-free, satisfying the zero-allocation hot path
// requirement.
// =============================================================================

use rayon::prelude::*;
use ratatui::style::Color;

use crate::camera::camera::Camera;
use crate::constants::{COLOR_PHOTON_RING, COLOR_VOID, EVENT_HORIZON, PHOTON_SPHERE};
use crate::physics::ray::{integrate, RayResult};
use crate::render::framebuffer::FrameBuffer;
use crate::render::scene::{accretion_disk_color, celestial_background};

/// Renders one frame into the frame buffer using parallel raymarching.
///
/// This is the performance-critical function called once per frame tick.
/// It partitions the frame buffer across all CPU cores, with each core
/// independently integrating rays for its assigned cells.
///
/// # Arguments
/// * `camera` — The observer's position and orientation.
/// * `fb` — The pre-allocated frame buffer to write into.
///
/// # Returns
/// The total number of rays traced this frame (= width × pixel_height).
pub fn render_frame(camera: &Camera, fb: &mut FrameBuffer) -> u64 {
    let width = fb.width;
    let pixel_h = fb.pixel_height();

    // Aspect ratio: terminal characters are typically ~2x taller than wide.
    // We compensate so the rendered image isn't squished.
    let aspect = width as f32 / pixel_h as f32;

    // Total rays = 2 per cell (top + bottom sub-pixel).
    let total_rays = width as u64 * fb.height as u64 * 2;

    // Use rayon to process all cells in parallel.
    // The closure captures only immutable data (camera, dimensions) and
    // writes only to its own cell — no synchronization needed.
    fb.cells_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, cell)| {
            // Decompose flat index into terminal coordinates.
            let tx = (idx % width as usize) as u16;
            let ty = (idx / width as usize) as u16;

            // Each terminal cell covers two vertical sub-pixels.
            // Top pixel: row = ty * 2,     Bottom pixel: row = ty * 2 + 1
            let pixel_top_y = ty * 2;
            let pixel_bot_y = ty * 2 + 1;

            // Convert pixel coordinates to normalized screen space [-1, 1].
            // u: left=-1, right=1; v: bottom=-1, top=1
            let u = (tx as f32 / (width as f32 - 1.0)) * 2.0 - 1.0;
            let v_top = 1.0 - (pixel_top_y as f32 / (pixel_h as f32 - 1.0)) * 2.0;
            let v_bot = 1.0 - (pixel_bot_y as f32 / (pixel_h as f32 - 1.0)) * 2.0;

            // Generate and trace rays for both sub-pixels.
            let ray_top = camera.generate_ray(u, v_top, aspect);
            let ray_bot = camera.generate_ray(u, v_bot, aspect);

            let result_top = integrate(&ray_top);
            let result_bot = integrate(&ray_bot);

            // Shade based on what the ray hit.
            cell.top_color = shade(result_top);
            cell.bottom_color = shade(result_bot);
        });

    total_rays
}

/// Converts a `RayResult` into a terminal color.
///
/// This is the "shader" function in traditional graphics terms.
/// It handles the photon ring glow — rays that barely escape or barely
/// fall in near the photon sphere (r ≈ 1.5 r_s) get a bright glow.
fn shade(result: RayResult) -> Color {
    match result {
        RayResult::EventHorizon(last_r) => {
            // Photon ring glow: rays that fell in very close to the
            // photon sphere get a bright ring color instead of pure black.
            // This creates the visually striking photon ring.
            if last_r < PHOTON_SPHERE * 1.1 && last_r > EVENT_HORIZON {
                let glow = 1.0 - ((last_r - EVENT_HORIZON) / (PHOTON_SPHERE * 1.1 - EVENT_HORIZON));
                lerp_shade_color(COLOR_VOID, COLOR_PHOTON_RING, glow * 0.8)
            } else {
                COLOR_VOID
            }
        }
        RayResult::Escaped(dir) => celestial_background(dir),
        RayResult::Disk(pos) => accretion_disk_color(pos),
    }
}

/// Linearly interpolates between two RGB colors for the shader.
#[inline(always)]
fn lerp_shade_color(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = match a {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (0.0, 0.0, 0.0),
    };
    let (br, bg, bb) = match b {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (0.0, 0.0, 0.0),
    };
    Color::Rgb(
        (ar + (br - ar) * t) as u8,
        (ag + (bg - ag) * t) as u8,
        (ab + (bb - ab) * t) as u8,
    )
}
