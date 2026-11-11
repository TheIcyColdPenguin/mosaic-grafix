// This moodule exports the drawing API

use std::error::Error;

use crate::framebuffer::{Colour, FrameBufferFile};

// DEFINITIONS

#[derive(Debug)]
pub struct Drawer {
    fb: FrameBufferFile,
    stroke: Colour,
    fill: Colour,
}

// IMPLS

impl Drawer {
    pub fn new(device_name: &str) -> Result<Self, Box<dyn Error>> {
        let fb = FrameBufferFile::new(device_name)?;

        return Ok(Drawer {
            fb,
            fill: Colour::BLACK,
            stroke: Colour::BLACK,
        });
    }

    // colours
    pub fn stroke(&mut self, col: Colour) {
        self.stroke = col;
    }
    pub fn fill(&mut self, col: Colour) {
        self.fill = col;
    }

    // drawing functions

    pub fn background(&mut self, col: Colour) {
        self.fb.fill(col);
    }

    pub fn point(&mut self, x: i32, y: i32, col: Colour) {
        self.fb.point((x, y), col);
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let dx = x1.abs_diff(x0) as i32;
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1.abs_diff(y0) as i32);
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            self.fb.point((x, y), self.stroke);

            let e2 = 2 * error;

            if e2 >= dy {
                if x == x1 {
                    break;
                }
                error += dy;
                x += sx;
            }
            if e2 <= dx {
                if y == y1 {
                    break;
                }

                error += dx;
                y += sy;
            }
        }
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.fb.flush()?;
        Ok(())
    }

    pub fn size(&self) -> (i32, i32) {
        return (self.fb.w, self.fb.h);
    }
}
