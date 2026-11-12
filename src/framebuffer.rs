use std::error::Error;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;
use std::ops::{Index, IndexMut};

use derive_more::Deref;
use image::Pixel;
use memmap2::{MmapMut, MmapOptions};

// DEFINITIONS

#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq)]
pub struct Colour(pub u16);

#[derive(Debug)]
pub struct FrameBufferFile<'a> {
    #[allow(unused)]
    fd: File,
    mmap: MmapMut,
    pixels: &'a mut [u16],

    pub w: i32,
    pub h: i32,
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
        return Colour((((r as u16) >> 3) << 11) | (((g as u16) >> 2) << 5) | ((b as u16) >> 3));
    }

    pub fn as_bytes(&self) -> (u8, u8) {
        let val = self.to_le_bytes();
        return (val[0], val[1]);
    }

    pub fn as_raw(&self) -> u16 {
        return self.0;
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

impl<'a> FrameBufferFile<'a> {
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

        let w = size[0];
        let h = size[1];

        let bits_per_pixel: i32 = bits_per_pixel_str.trim().parse()?;

        let buf: Vec<_> = vec![0u8; (bits_per_pixel / 8 * w * h) as usize];
        // write in order to tell mmap how large the file is
        fd.write_all(&buf)?;

        let mut mmap = unsafe { MmapOptions::new().len(buf.len()).map_mut(&fd)? };

        // remap to u16
        let m_ptr = mmap.as_mut_ptr() as *mut u16;
        let aligned = m_ptr.is_aligned();
        if !aligned {
            return Err("Unaligned map".into());
        }
        let m_len = mmap.len() / 2;

        let pixels = unsafe { std::slice::from_raw_parts_mut(m_ptr, m_len) };

        Ok(Self {
            fd,
            mmap,
            w,
            h,
            pixels,
        })
    }

    pub fn fill(&mut self, val: Colour) {
        self.pixels.fill(val.as_raw());
    }

    pub fn point(&mut self, pos: (i32, i32), val: Colour) {
        if let Some(index) = self.at(pos) {
            self.pixels[index] = val.as_raw();
        }
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        // NOTE: Async available
        self.mmap.flush()?;

        Ok(())
    }

    fn at(&self, pos: (i32, i32)) -> Option<usize> {
        if pos.0 >= 0 && pos.0 < self.w && pos.1 >= 0 && pos.1 < self.h {
            Some((self.w * (pos.1) + (pos.0)) as usize)
        } else {
            None
        }
    }
}
