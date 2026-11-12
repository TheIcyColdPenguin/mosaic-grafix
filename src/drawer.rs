// This moodule exports the drawing API

use std::error::Error;

use image::{DynamicImage, GenericImageView, imageops::FilterType};

use crate::framebuffer::{Colour, FrameBufferFile};

// DEFINITIONS

#[derive(Debug)]
pub struct Drawer {
    fb: FrameBufferFile,
    stroke: Colour,
    fill: Colour,
}

#[derive(Debug)]
pub enum ImageResizeOptions {
    Fit,
    Fill,
    Stretch,
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

    pub fn image_str(
        &mut self,
        image: &str,
        x0: i32,
        y0: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let og_image = image::ImageReader::open(image)?.decode()?;

        self.image(&og_image, x0, y0)?;

        Ok(())
    }

    pub fn image_str_resized(
        &mut self,
        image: &str,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        image_option: ImageResizeOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let og_image = image::ImageReader::open(image)?.decode()?;

        self.image_resized(&og_image, x0, y0, x1, y1, image_option)?;

        Ok(())
    }

    pub fn image_resized(
        &mut self,
        image: &DynamicImage,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        image_option: ImageResizeOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nwidth = x1.abs_diff(x0);
        let nheight = y1.abs_diff(y0);

        // only resize if required
        if nwidth == image.width() && nheight == image.height() {
            self.image(image, x0, y0)?;
        } else {
            let resized_image = match image_option {
                ImageResizeOptions::Fit => image.resize(nwidth, nheight, FilterType::Nearest),
                ImageResizeOptions::Stretch => {
                    image.resize_exact(nwidth, nheight, FilterType::Nearest)
                }
                ImageResizeOptions::Fill => {
                    image.resize_to_fill(nwidth, nheight, FilterType::Nearest)
                }
            };

            self.image(&resized_image, x0, y0)?;
        };

        Ok(())
    }

    pub fn image(
        &mut self,
        image: &DynamicImage,
        x0: i32,
        y0: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (x, y, rgb) in image.pixels() {
            let x = x as i32;
            let y = y as i32;
            self.fb.point((x0 + x, y0 + y), rgb.into());
        }
        Ok(())
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        self.fb.flush()?;
        Ok(())
    }

    pub fn size(&self) -> (i32, i32) {
        return (self.fb.w, self.fb.h);
    }
}
