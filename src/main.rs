// =============================================================================
// main.rs — Entry Point: Terminal Setup & Main Event Loop
//
// This is the outermost layer. It:
//   1. Enters crossterm's raw mode + alternate screen + mouse capture.
//   2. Constructs the App with default camera and frame buffer.
//   3. Runs the main loop: poll events → handle → render → draw → repeat.
//   4. Restores the terminal to its original state on exit.
//
// The event loop targets ~60 FPS by using crossterm's `poll()` with a
// timeout calculated from the remaining frame budget.
// =============================================================================

mod camera;
mod constants;
mod physics;
mod render;
mod tui;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::ExecutableCommand;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::constants::FRAME_DURATION_MS;
use crate::tui::app::App;
use crate::tui::ui::draw;

fn main() -> io::Result<()> {
    // ---- Step 1: Set up the terminal ----
    // Raw mode: disables line buffering and special key handling (Ctrl+C, etc.)
    // so we can capture every keypress ourselves.
    enable_raw_mode()?;

    // Switch to the alternate screen buffer (like vim does) so we don't
    // trash the user's terminal history.
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;

    // Create the ratatui Terminal using crossterm as the backend.
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ---- Step 2: Initialize application state ----
    let size = terminal.size()?;
    let mut app = App::new(size.width, size.height);

    // FPS tracking.
    let mut frame_count: u64 = 0;
    let mut fps_timer = Instant::now();

    // ---- Step 3: Main event loop ----
    while app.running {
        let frame_start = Instant::now();

        // --- 3a. Poll for input events ---
        // We use a short timeout so we don't block the render loop.
        // If no event arrives within the timeout, we just render the
        // next frame.
        let poll_timeout = Duration::from_millis(1);
        if event::poll(poll_timeout)? {
            let evt = event::read()?;
            app.handle_event(evt);
        }

        // --- 3b. Resize framebuffer if terminal size changed ---
        let current_size = terminal.size()?;
        if current_size.width != app.framebuffer.width
            || current_size.height != app.framebuffer.height
        {
            app.framebuffer.resize(current_size.width, current_size.height);
        }

        // --- 3c. Render the scene (parallel raymarching) ---
        let render_start = Instant::now();
        app.render();
        app.frame_time_ms = render_start.elapsed().as_secs_f64() * 1000.0;

        // --- 3d. Draw to terminal ---
        terminal.draw(|frame| {
            draw(frame, &app);
        })?;

        // --- 3e. FPS tracking ---
        frame_count += 1;
        let elapsed = fps_timer.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            app.fps = frame_count as f64 / elapsed;
            frame_count = 0;
            fps_timer = Instant::now();
        }

        // --- 3f. Frame rate limiting ---
        // Sleep for the remaining frame budget to avoid burning CPU
        // when frames render faster than the target rate.
        let frame_elapsed = frame_start.elapsed();
        let target_duration = Duration::from_millis(FRAME_DURATION_MS);
        if frame_elapsed < target_duration {
            std::thread::sleep(target_duration - frame_elapsed);
        }
    }

    // ---- Step 4: Restore the terminal ----
    disable_raw_mode()?;
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)?;
    terminal
        .backend_mut()
        .execute(DisableMouseCapture)?;
    terminal.show_cursor()?;

    println!("👋 Schwarzschild Raytracer — session ended.");
    Ok(())
}
