use std::error::Error;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;

use derive_more::Deref;
use image::Pixel;
use memmap2::{MmapMut, MmapOptions};

// DEFINITIONS

#[derive(Debug, Deref, Clone, Copy)]
pub struct Colour(pub u16);

#[derive(Debug)]
pub struct FrameBufferFile {
    #[allow(unused)]
    fd: File,
    mmap: MmapMut,
    pub w: i32,
    pub h: i32,

    // to allow for automatic rotation
    #[allow(unused)]
    actual_w: i32,
    #[allow(unused)]
    actual_h: i32,
}

// IMPLS

impl Colour {
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);

    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        return Colour((((b as u16) >> 3) << 11) | (((g as u16) >> 2) << 5) | ((r as u16) >> 3));
    }

    pub fn as_bytes(&self) -> (u8, u8) {
        let val = self.to_le_bytes();
        return (val[0], val[1]);
    }
}

impl<T> From<T> for Colour
where
    T: Pixel<Subpixel = u8>,
{
    fn from(value: T) -> Self {
        let channels = value.channels();
        return Colour::rgb(channels[0], channels[1], channels[2]);
    }
}

impl FrameBufferFile {
    pub fn new(device_name: &str) -> Result<Self, Box<dyn Error>> {
        let mut fd = File::options()
            .write(true)
            .read(true)
            .open(format!("/dev/{}", device_name))?;

        let size_str =
            fs::read_to_string(format!("/sys/class/graphics/{}/virtual_size", device_name))?;
        let bits_per_pixel_str = fs::read_to_string(format!(
            "/sys/class/graphics/{}/bits_per_pixel",
            device_name
        ))?;

        let size: Vec<i32> = size_str
            .split(',')
            .filter_map(|x| x.trim().parse().ok())
            .collect();

        if size.len() != 2 {
            return Err(format!(
                "Unexpected number of elements in display size {}",
                size.len()
            ))?;
        }

        let actual_w = size[0];
        let actual_h = size[1];

        let w = actual_h;
        let h = actual_w;

        let bits_per_pixel: i32 = bits_per_pixel_str.trim().parse()?;

        let buf: Vec<_> = vec![0u8; (bits_per_pixel / 8 * w * h) as usize];
        // write in order to tell mmap how large the file is
        fd.write_all(&buf)?;

        let mmap = unsafe { MmapOptions::new().len(buf.len()).map_mut(&fd)? };

        Ok(Self {
            fd,
            mmap,
            w,
            h,
            actual_w,
            actual_h,
        })
    }

    pub fn fill(&mut self, val: Colour) {
        let col = val.as_bytes();
        for chunk in self.mmap.chunks_exact_mut(2) {
            chunk[0] = col.0;
            chunk[1] = col.1;
        }
    }
    pub fn point(&mut self, pos: (i32, i32), val: Colour) {
        if self.is_in_range(pos) {
            let index = self.at(pos);
            let col = val.as_bytes();

            self.mmap[index] = col.0;
            self.mmap[index + 1] = col.1;
        }
    }

    pub fn is_in_range(&self, pos: (i32, i32)) -> bool {
        return pos.0 >= 0 && pos.0 < self.w && pos.1 >= 0 && pos.1 < self.h;
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        // NOTE: Async available
        self.mmap.flush()?;

        Ok(())
    }

    fn at(&self, pos: (i32, i32)) -> usize {
        // automatic rotation by changing index system

        // Old
        // 2*(self.w * (pos.1) + pos.0) as usize

        // New with Rotation
        2 * (self.actual_w * (pos.0) + (self.actual_w - 1 - pos.1)) as usize
    }
}
