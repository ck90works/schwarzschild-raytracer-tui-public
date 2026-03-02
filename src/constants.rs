// =============================================================================
// constants.rs — Centralized physical constants, simulation parameters, and
// style tokens for the Schwarzschild Raytracer TUI.
//
// All "magic numbers" live here. Nothing is hardcoded in the rendering or
// physics code. If you want to tweak the simulation, this is the single
// source of truth.
// =============================================================================

use ratatui::style::Color;

// ---------------------------------------------------------------------------
// § Physical Constants (Natural Units: G = c = M = 1)
// ---------------------------------------------------------------------------

/// The Schwarzschild radius r_s = 2GM/c² = 2 in natural units.
/// Any photon crossing this boundary is swallowed by the singularity.
pub const SCHWARZSCHILD_RADIUS: f32 = 2.0;

/// Alias for readability — the event horizon *is* the Schwarzschild radius
/// for a non-rotating black hole.
pub const EVENT_HORIZON: f32 = SCHWARZSCHILD_RADIUS;

/// The photon sphere sits at r = 1.5 * r_s. Photons here orbit the black
/// hole on unstable circular paths.
pub const PHOTON_SPHERE: f32 = 1.5 * SCHWARZSCHILD_RADIUS;

// ---------------------------------------------------------------------------
// § Raymarcher Tuning
// ---------------------------------------------------------------------------

/// Maximum number of RK4 integration steps before we give up on a ray.
/// Higher = more accurate distant lensing, but slower frames.
pub const MAX_INTEGRATION_STEPS: u32 = 500;

/// If a ray's radial distance exceeds this, we consider it "escaped" to
/// the celestial sphere and shade it with the background.
pub const ESCAPE_RADIUS: f32 = 50.0;

/// Base step size for the RK4 integrator. This is scaled dynamically based
/// on proximity to the event horizon.
pub const STEP_SIZE_BASE: f32 = 0.15;

/// Minimum step size near the event horizon. Prevents the integrator from
/// taking infinitely small steps and stalling.
pub const STEP_SIZE_MIN: f32 = 0.005;

/// Maximum step size far from the black hole. Allows the integrator to
/// take large jumps through approximately flat spacetime.
pub const STEP_SIZE_MAX: f32 = 0.5;

// ---------------------------------------------------------------------------
// § Accretion Disk Geometry
// ---------------------------------------------------------------------------

/// Inner edge of the accretion disk, at the ISCO (Innermost Stable Circular
/// Orbit) = 3 * r_s for a Schwarzschild black hole.
pub const DISK_INNER_RADIUS: f32 = 3.0 * SCHWARZSCHILD_RADIUS;

/// Outer edge of the accretion disk — purely aesthetic choice.
pub const DISK_OUTER_RADIUS: f32 = 12.0 * SCHWARZSCHILD_RADIUS;



// ---------------------------------------------------------------------------
// § Camera Defaults
// ---------------------------------------------------------------------------

/// Default observer distance from the singularity (in natural units).
pub const CAMERA_DEFAULT_DISTANCE: f32 = 15.0;

/// Minimum camera distance — don't let the user fall past the photon sphere.
pub const CAMERA_MIN_DISTANCE: f32 = PHOTON_SPHERE + 0.5;

/// Maximum camera distance.
pub const CAMERA_MAX_DISTANCE: f32 = 80.0;

/// Default vertical field of view in radians (~60°).
pub const CAMERA_DEFAULT_FOV: f32 = std::f32::consts::FRAC_PI_3;

/// Mouse drag sensitivity for orbital rotation (radians per pixel of drag).
pub const MOUSE_SENSITIVITY: f32 = 0.015;

/// Zoom speed per key press or scroll tick.
pub const ZOOM_SPEED: f32 = 1.0;

// ---------------------------------------------------------------------------
// § Target Frame Rate
// ---------------------------------------------------------------------------

/// Target frames per second for the main render loop.
pub const TARGET_FPS: u64 = 60;

/// Duration of one frame in milliseconds.
pub const FRAME_DURATION_MS: u64 = 1000 / TARGET_FPS;

// ---------------------------------------------------------------------------
// § Luminance-to-Character Mapping
// ---------------------------------------------------------------------------

/// Characters ordered from darkest to brightest. Used to convert a
/// floating-point luminance [0.0, 1.0] to an ASCII representation.
pub const LUMINANCE_CHARS: &[char] = &[' ', '.', '·', ':', '*', '+', '#', '@'];

// ---------------------------------------------------------------------------
// § Color Palette — Centralized Style Tokens
// ---------------------------------------------------------------------------

/// Event horizon / void color.
pub const COLOR_VOID: Color = Color::Rgb(0, 0, 0);

/// Hottest part of the accretion disk (inner edge — white-hot).
pub const COLOR_DISK_HOT: Color = Color::Rgb(255, 240, 220);

/// Mid-temperature disk (orange glow).
pub const COLOR_DISK_MID: Color = Color::Rgb(255, 140, 50);

/// Cooler outer disk (deep red/maroon).
pub const COLOR_DISK_COOL: Color = Color::Rgb(180, 40, 20);

/// Background celestial grid — primary color.
pub const COLOR_GRID_PRIMARY: Color = Color::Rgb(30, 35, 60);

/// Background celestial grid — secondary color (alternating squares).
pub const COLOR_GRID_SECONDARY: Color = Color::Rgb(20, 22, 40);

/// Bright "star" dots scattered on the celestial sphere.
pub const COLOR_STAR: Color = Color::Rgb(200, 210, 255);

/// HUD text color.
pub const COLOR_HUD_TEXT: Color = Color::Rgb(180, 220, 255);

/// HUD background (semi-transparent dark).
pub const COLOR_HUD_BG: Color = Color::Rgb(10, 12, 20);

/// Photon ring glow — extremely bright.
pub const COLOR_PHOTON_RING: Color = Color::Rgb(255, 255, 240);
