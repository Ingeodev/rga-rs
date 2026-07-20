use rga_sys::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PixelFormat {
    Rgba8888 = _Rga_SURF_FORMAT_RK_FORMAT_RGBA_8888 as i32,
    Rgb888   = _Rga_SURF_FORMAT_RK_FORMAT_RGB_888 as i32,
    Bgr888   = _Rga_SURF_FORMAT_RK_FORMAT_BGR_888 as i32,
    Nv12     = _Rga_SURF_FORMAT_RK_FORMAT_YCbCr_420_SP as i32, // el formato típico de salida del ISP/cámara
    Nv21     = _Rga_SURF_FORMAT_RK_FORMAT_YCrCb_420_SP as i32,
    // agrega las que realmente uses en el pipeline; no hace falta cubrir las ~50
}

impl PixelFormat {
    pub(crate) fn as_raw(self) -> std::os::raw::c_int {
        self as i32
    }

    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            PixelFormat::Rgba8888 => 4,
            PixelFormat::Rgb888 | PixelFormat::Bgr888 => 3,
            PixelFormat::Nv12 | PixelFormat::Nv21 => 1, // planar: 1 byte/pixel en Y, UV comparten
        }
    }
}