// =============================================================================
// render/framebuffer.rs — Pre-allocated Frame Buffer with Half-Block Rendering
//
// The frame buffer is a flat 1D array of "cells" that represents the terminal
// screen. It is pre-allocated at startup and only resized on terminal reshape
// events — the hot rendering loop never allocates memory.
//
// We use Unicode half-block characters (▀) to double the vertical resolution.
// Each terminal cell encodes TWO vertical pixels:
//   - The top pixel → foreground color
//   - The bottom pixel → background color
//   - Character = '▀' (upper half block)
//
// This effectively gives us 2× the vertical resolution of a normal terminal
// grid — critical for making the raytraced image look smooth.
// =============================================================================

use ratatui::style::Color;

use crate::constants::COLOR_VOID;

/// The half-block character used for sub-cell rendering.
/// '▀' fills the top half of the cell — we set its foreground to the top
/// pixel's color and background to the bottom pixel's color.
pub const HALF_BLOCK: char = '▀';

/// A single cell in the frame buffer, representing two vertical pixels.
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    /// Color of the top pixel (rendered as foreground of '▀').
    pub top_color: Color,
    /// Color of the bottom pixel (rendered as background of '▀').
    pub bottom_color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            top_color: COLOR_VOID,
            bottom_color: COLOR_VOID,
        }
    }
}

/// The frame buffer holds the entire rendered image as a flat array of cells.
///
/// Layout: cells[y * width + x] for terminal column x, terminal row y.
/// Since each row encodes 2 vertical pixels, the effective pixel resolution
/// is (width, height * 2).
pub struct FrameBuffer {
    /// Width in terminal columns.
    pub width: u16,
    /// Height in terminal rows (each row = 2 pixels vertically).
    pub height: u16,
    /// Flat array of cells, length = width × height.
    pub cells: Vec<Cell>,
}

impl FrameBuffer {
    /// Creates a new frame buffer of the given terminal dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let cell_count = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![Cell::default(); cell_count],
        }
    }

    /// Resizes the frame buffer to new dimensions. Only reallocates if
    /// the new size is larger than the current capacity.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let cell_count = width as usize * height as usize;
        self.cells.resize(cell_count, Cell::default());
    }

    /// Returns a mutable reference to the cell at (x, y).
    /// Used in tests for targeted pixel inspection.
    #[cfg(test)]
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        &mut self.cells[y as usize * self.width as usize + x as usize]
    }

    /// Returns the total number of "virtual pixels" — double the height
    /// because each cell row encodes two vertical pixels.
    pub fn pixel_height(&self) -> u16 {
        self.height * 2
    }

    /// Clears the entire buffer to black (void color).
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Returns a mutable slice of all cells for parallel rendering.
    /// Each cell's index can be decomposed into (x, y) via:
    ///   x = index % width
    ///   y = index / width
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_creation() {
        let fb = FrameBuffer::new(80, 40);
        assert_eq!(fb.cells.len(), 80 * 40);
        assert_eq!(fb.pixel_height(), 80);
    }

    #[test]
    fn framebuffer_resize() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.resize(20, 20);
        assert_eq!(fb.cells.len(), 400);
        assert_eq!(fb.width, 20);
        assert_eq!(fb.height, 20);
    }

    #[test]
    fn framebuffer_clear() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.get_mut(5, 5).top_color = Color::Rgb(255, 0, 0);
        fb.clear();
        assert_eq!(fb.get_mut(5, 5).top_color, COLOR_VOID);
    }
}
