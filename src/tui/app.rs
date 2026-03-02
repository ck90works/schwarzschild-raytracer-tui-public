// =============================================================================
// tui/app.rs — Application State & Event Handling
//
// The App struct holds the entire application state: camera, frame buffer,
// timing, and input tracking. The event loop (in main.rs) calls methods on
// App to update state and redraw.
// =============================================================================

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::camera::camera::Camera;
use crate::render::framebuffer::FrameBuffer;
use crate::render::renderer::render_frame;

/// The main application state.
pub struct App {
    /// The orbital camera.
    pub camera: Camera,
    /// Pre-allocated frame buffer (resized on terminal reshape).
    pub framebuffer: FrameBuffer,
    /// Whether the application should continue running.
    pub running: bool,

    // --- Performance telemetry ---
    /// Current frames per second.
    pub fps: f64,
    /// Total ray integration steps completed in the last frame.
    pub total_steps: u64,
    /// Time taken to render the last frame (milliseconds).
    pub frame_time_ms: f64,

    // --- Mouse tracking ---
    /// Whether the left mouse button is currently held.
    mouse_held: bool,
    /// Last mouse position (for computing drag deltas).
    last_mouse_x: u16,
    last_mouse_y: u16,
}

impl App {
    /// Creates a new application with default state.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            camera: Camera::new(),
            // Reserve a few rows for the HUD overlay at the top.
            framebuffer: FrameBuffer::new(width, height),
            running: true,
            fps: 0.0,
            total_steps: 0,
            frame_time_ms: 0.0,
            mouse_held: false,
            last_mouse_x: 0,
            last_mouse_y: 0,
        }
    }

    /// Renders the current frame using the parallel raymarcher.
    /// Tracks the total number of rays traced for HUD telemetry.
    pub fn render(&mut self) {
        // Clear the buffer before rendering to avoid stale pixels
        // (e.g., after a terminal resize where new cells may have garbage).
        self.framebuffer.clear();
        self.total_steps = render_frame(&self.camera, &mut self.framebuffer);
    }

    /// Handles a crossterm event (key press, mouse, resize).
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(w, h) => {
                self.framebuffer.resize(w, h);
            }
            _ => {}
        }
    }

    /// Handles keyboard input.
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // Quit: q or Ctrl+C.
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }

            // Zoom in/out with +/- keys.
            KeyCode::Char('+') | KeyCode::Char('=') => self.camera.zoom(-1.0),
            KeyCode::Char('-') | KeyCode::Char('_') => self.camera.zoom(1.0),

            // Arrow keys for orbital rotation (alternative to mouse drag).
            KeyCode::Left => self.camera.orbit(-10.0, 0.0),
            KeyCode::Right => self.camera.orbit(10.0, 0.0),
            KeyCode::Up => self.camera.orbit(0.0, 10.0),
            KeyCode::Down => self.camera.orbit(0.0, -10.0),

            _ => {}
        }
    }

    /// Handles mouse input (drag for orbit, scroll for zoom).
    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            // Left button pressed — start tracking drag.
            MouseEventKind::Down(MouseButton::Left) => {
                self.mouse_held = true;
                self.last_mouse_x = mouse.column;
                self.last_mouse_y = mouse.row;
            }
            // Left button released — stop tracking.
            MouseEventKind::Up(MouseButton::Left) => {
                self.mouse_held = false;
            }
            // Mouse moved while button held — orbit the camera.
            MouseEventKind::Drag(MouseButton::Left) => {
                let dx = mouse.column as f32 - self.last_mouse_x as f32;
                let dy = mouse.row as f32 - self.last_mouse_y as f32;
                self.camera.orbit(dx, dy);
                self.last_mouse_x = mouse.column;
                self.last_mouse_y = mouse.row;
            }
            // Scroll wheel — zoom.
            MouseEventKind::ScrollUp => self.camera.zoom(-0.5),
            MouseEventKind::ScrollDown => self.camera.zoom(0.5),
            _ => {}
        }
    }
}
