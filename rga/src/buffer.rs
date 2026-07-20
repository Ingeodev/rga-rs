use std::ffi::{c_int, c_void};
use rga_sys::*;
use crate::error::RgaError;
use crate::format::PixelFormat;

/// El RGA requiere que `wstride * bytes_per_pixel` sea múltiplo de 16.
fn align_stride(stride: u32, bpp: u32) -> u32 {
    // Necesitamos stride * bpp % 16 == 0. El paso de alineación es 16 / gcd(16, bpp).
    let align = 16 / gcd(16, bpp);
    (stride + align - 1) / align * align
}

const fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

enum Memory<'a> {
    Cpu { _keep_alive: &'a [u8] },
    CpuMut { _keep_alive: &'a mut [u8] }, //Para escribir la imagen de salida se requiere un buffer mutable
    Dma,
}

#[derive(Clone, Copy)]
pub struct ImageLayout {
    pub width: u32,
    pub height: u32,
    pub wstride: u32, //El ancho si tuviese padding
    pub hstride: u32, //La altura si tuviera padding
}

impl ImageLayout {
    pub fn tight(width: u32, height: u32) -> Self {
        Self { width, height, wstride: width, hstride: height }
    }

    pub fn aligned(&self, format: PixelFormat) -> Self {
        let bpp = format.bytes_per_pixel();
        Self {
            width: self.width,
            height: self.height,
            wstride: align_stride(self.wstride, bpp),
            hstride: self.hstride,
        }
    }
}

pub struct RgaBuffer<'a> {
    memory: Memory<'a>,
    handle: rga_buffer_handle_t,
    raw: rga_buffer_t,
    layout: ImageLayout,
    format: PixelFormat,
}

impl<'a> RgaBuffer<'a> {
    fn from_handle(
        handle: rga_buffer_handle_t,
        layout: ImageLayout,
        format: PixelFormat,
        memory: Memory<'a>,
    ) -> Result<Self, RgaError> {
        if handle == 0 {
            return Err(RgaError::ImportFailed); // fallback legítimo, no lo elimina From<IM_STATUS>
        }
        let raw = unsafe {
            wrapbuffer_handle_t(
                handle,
                layout.width as i32, layout.height as i32,
                layout.wstride as i32, layout.hstride as i32,
                format.as_raw(),
            )
        };
        Ok(Self { memory, handle, raw, layout, format })
    }

    fn import_param(layout: &ImageLayout, format: PixelFormat) -> im_handle_param_t {
        im_handle_param_t {
            width: layout.wstride,
            height: layout.hstride,
            format: format.as_raw() as u32,
        }
    }

    pub fn import_dma_fd(fd: c_int, layout: ImageLayout, format: PixelFormat) -> Result<Self, RgaError> {
        let mut param = Self::import_param(&layout, format);
        let handle = unsafe { importbuffer_fd(fd, &mut param) };
        Self::from_handle(handle, layout, format, Memory::Dma)
    }

    pub fn import_virtual(bytes: &'a [u8], layout: ImageLayout, format: PixelFormat) -> Result<Self, RgaError> {
        let mut param = Self::import_param(&layout, format);
        let va = bytes.as_ptr() as *mut c_void;
        let handle = unsafe { importbuffer_virtualaddr(va, &mut param) };
        Self::from_handle(handle, layout, format, Memory::Cpu { _keep_alive: bytes })
    }

    pub fn import_virtual_mut(bytes: &'a mut [u8], layout: ImageLayout, format: PixelFormat) -> Result<Self, RgaError> {
        let mut param = Self::import_param(&layout, format);
        let va = bytes.as_mut_ptr() as *mut c_void;
        let handle = unsafe { importbuffer_virtualaddr(va, &mut param) };
        Self::from_handle(handle, layout, format, Memory::CpuMut { _keep_alive: bytes })
    }

    pub fn width(&self) -> u32 { self.layout.width }
    pub fn height(&self) -> u32 { self.layout.height }
    pub fn format(&self) -> PixelFormat { self.format }

    /// Tamaño de buffer necesario para este layout y formato.
    pub fn buffer_size(&self) -> usize {
        (self.layout.hstride * self.layout.wstride * self.format.bytes_per_pixel()) as usize
    }
}

impl<'a> RgaBuffer<'a> {
    pub fn raw(&self) -> rga_buffer_t {
        self.raw
    }

    pub fn tight_data(&self) -> Vec<u8> {
        let bpp = self.format.bytes_per_pixel() as usize;
        let src_pitch = self.layout.wstride as usize * bpp;
        let dst_pitch = self.layout.width as usize * bpp;
        let height = self.layout.height as usize;
        let bytes = self.bytes();
        let mut out = vec![0u8; height * dst_pitch];
        for y in 0..height {
            let src_start = y * src_pitch;
            let dst_start = y * dst_pitch;
            out[dst_start..dst_start + dst_pitch]
                .copy_from_slice(&bytes[src_start..src_start + dst_pitch]);
        }
        out
    }

    pub fn bytes(&self) -> &[u8] {
        match &self.memory {
            Memory::Cpu { _keep_alive: bytes } => bytes,
            Memory::CpuMut { _keep_alive: bytes } => bytes,
            Memory::Dma => panic!("bytes() requires a CPU-backed buffer"),
        }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        match &mut self.memory {
            Memory::CpuMut { _keep_alive: bytes } => bytes,
            _ => panic!("bytes_mut() requires a mutable CPU-backed buffer"),
        }
    }
}

impl<'a> Drop for RgaBuffer<'a> {
    fn drop(&mut self) {
        unsafe { releasebuffer_handle(self.handle) };
    }
}