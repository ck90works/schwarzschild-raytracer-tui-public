// =============================================================================
// tui/ui.rs — ratatui Drawing & HUD Overlay
//
// Translates the frame buffer into ratatui widgets for display. The key
// optimization is "span batching": instead of creating one Span per cell
// (which would be 20,000+ Spans for a typical terminal), we merge
// consecutive cells that share the same colors into a single Span.
//
// The HUD overlay is a small semi-transparent box at the top-left corner
// showing real-time telemetry: observer distance, FPS, and frame time.
// =============================================================================

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::constants::{COLOR_HUD_BG, COLOR_HUD_TEXT, SCHWARZSCHILD_RADIUS};
use crate::render::framebuffer::{FrameBuffer, HALF_BLOCK};
use crate::tui::app::App;

/// Main draw function — renders the frame buffer and HUD overlay.
///
/// Called once per frame by the main event loop (main.rs).
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // --- Render the raytraced image ---
    let image = render_framebuffer_to_paragraph(&app.framebuffer);
    frame.render_widget(image, area);

    // --- Render the HUD overlay on top ---
    let hud_area = Rect::new(1, 1, 36, 8);
    let hud = build_hud(app);
    frame.render_widget(hud, hud_area);
}

/// Converts the frame buffer into a ratatui `Paragraph` widget.
///
/// # Span Batching
/// To avoid creating an individual `Span` for every single cell (which would
/// be extremely wasteful), we batch consecutive cells that share the same
/// foreground+background color pair into a single `Span`. This typically
/// reduces the Span count by 10-50×.
fn render_framebuffer_to_paragraph(fb: &FrameBuffer) -> Paragraph<'static> {
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(fb.height as usize);

    for y in 0..fb.height {
        let mut spans: Vec<Span<'static>> = Vec::new();

        // Track the current "batch" — consecutive cells with the same style.
        let mut batch_text = String::new();
        let mut batch_style = Style::default();
        let mut batch_started = false;

        for x in 0..fb.width {
            let cell = &fb.cells[y as usize * fb.width as usize + x as usize];

            // Build the style for this cell: fg = top pixel, bg = bottom pixel.
            let style = Style::default().fg(cell.top_color).bg(cell.bottom_color);

            if batch_started && style == batch_style {
                // Same style as previous cell — extend the current batch.
                batch_text.push(HALF_BLOCK);
            } else {
                // Different style — flush the previous batch and start a new one.
                if batch_started {
                    spans.push(Span::styled(batch_text.clone(), batch_style));
                }
                batch_text.clear();
                batch_text.push(HALF_BLOCK);
                batch_style = style;
                batch_started = true;
            }
        }

        // Flush the final batch for this row.
        if batch_started {
            spans.push(Span::styled(batch_text, batch_style));
        }

        lines.push(Line::from(spans));
    }

    Paragraph::new(lines)
}

/// Builds the HUD overlay widget displaying telemetry data.
fn build_hud(app: &App) -> Paragraph<'static> {
    let distance_rs = app.camera.r / SCHWARZSCHILD_RADIUS;
    let theta_deg = app.camera.theta.to_degrees();

    let hud_style = Style::default().fg(COLOR_HUD_TEXT).bg(COLOR_HUD_BG);
    let label_style = hud_style.add_modifier(Modifier::BOLD);

    let lines = vec![
        Line::from(vec![
            Span::styled(" OBSERVER ", label_style),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  r = {distance_rs:.2} r_s"),
                hud_style,
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  θ = {theta_deg:.1}°"),
                hud_style,
            ),
        ]),
        Line::from(Span::styled(
            format!("  FPS: {:.0}", app.fps),
            hud_style,
        )),
        Line::from(Span::styled(
            format!("  Frame: {:.1}ms", app.frame_time_ms),
            hud_style,
        )),
        Line::from(Span::styled(
            format!("  Rays: {}", app.total_steps),
            hud_style,
        )),
    ];

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_HUD_TEXT).bg(COLOR_HUD_BG))
            .style(hud_style),
    )
}
