use std::error::Error;
use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;
use std::ops::{Index, IndexMut};

use derive_more::{Deref, DerefMut};

// STRUCT DEFS

// Wrapper type for succint debug fmt
#[derive(Deref, DerefMut)]
#[deref(forward)]
#[deref_mut(forward)]
struct DataBuffer(Box<[u16]>);

#[derive(Debug)]
pub struct FrameBufferFile {
    fd: File,
    // mmap: MmapMut,
    buf: DataBuffer,
    pub w: i32,
    pub h: i32,
    // bits_per_pixel: u32,
    // stride: u32,
}

// IMPLS

impl Debug for DataBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataBuffer(len: {})", self.0.len())
    }
}

impl FrameBufferFile {
    pub fn new(device_name: &str) -> Result<Self, Box<dyn Error>> {
        let fd = File::options()
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

        // TODO: Use bits_per_pixel to work correctly for 8 bit colour
        let buf: Vec<_> = vec![0; (bits_per_pixel / 16 * w * h) as usize];
        let buf = DataBuffer(buf.into_boxed_slice());
        // dbg!(fd.set_len(((bits_per_pixel / 8) * w*h) as u64)?);
        // let mmap = unsafe { MmapMut::map_mut(&fd)? };

        Ok(Self {
            fd,
            // mmap,
            buf,
            w,
            h,
        })
    }

    pub fn fill(&mut self, val: u16) {
        self.buf.fill(val);
    }

    pub fn is_in_range(&mut self, val: (i32, i32)) -> bool {
        return val.0 > 0 && val.0 < self.w && val.1 > 0 && val.1 < self.h;
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        // (&mut self.mmap[..]).write_all(&self.buf)?;

        // NOTE: Async available
        // self.mmap.flush()?;

        let (head, body, tail) = unsafe { self.buf.align_to() };
        assert!(head.is_empty());
        assert!(tail.is_empty());

        self.fd.write_all(body)?;

        Ok(())
    }
}

impl Index<(i32, i32)> for FrameBufferFile {
    type Output = u16;
    fn index(&self, index: (i32, i32)) -> &Self::Output {
        // TODO: fix with bits_per_pixel
        let index = (self.w * (index.1) + index.0) as usize;
        return &self.buf[index];
    }
}

impl IndexMut<(i32, i32)> for FrameBufferFile {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        // TODO: fix with bits_per_pixel
        let index = (self.w * (index.1) + index.0) as usize;

        return &mut self.buf[index];
    }
}
