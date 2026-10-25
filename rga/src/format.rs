use rga_sys::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PixelFormat {
    Rgba8888 = _Rga_SURF_FORMAT_RK_FORMAT_RGBA_8888 as i32,
    Rgb888   = _Rga_SURF_FORMAT_RK_FORMAT_RGB_888 as i32,
    Bgr888   = _Rga_SURF_FORMAT_RK_FORMAT_BGR_888 as i32,
    Nv12     = _Rga_SURF_FORMAT_RK_FORMAT_YCbCr_420_SP as i32, // el formato típico de salida del ISP/cámara
    Nv21     = _Rga_SURF_FORMAT_RK_FORMAT_YCrCb_420_SP as i32,
    /// UYVY packed 4:2:2 (2 bytes/pixel): el formato nativo de la IMX662.
    Uyvy422  = _Rga_SURF_FORMAT_RK_FORMAT_UYVY_422 as i32,
    // agrega las que realmente uses en el pipeline; no hace falta cubrir las ~50
}

impl PixelFormat {
    pub(crate) fn as_raw(self) -> std::os::raw::c_int {
        self as i32
    }

    /// Bytes por pixel del formato **packed** base. Para formatos
    /// semi-planarios (NV12/NV21), el tamaño real del buffer incluye los
    /// planos cromáticos: usa `RgaBuffer::buffer_size()` en su lugar.
    ///
    /// Importante: este valor es el que usa `ImageLayout::aligned()` para la
    /// invariante de stride del RGA (`wstride * bpp % 16 == 0`).
    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            PixelFormat::Rgba8888 => 4,
            PixelFormat::Rgb888 | PixelFormat::Bgr888 => 3,
            PixelFormat::Uyvy422 => 2,
            PixelFormat::Nv12 | PixelFormat::Nv21 => 1, // plano Y: 1 byte/pixel, UV comparten
        }
    }

    /// ¿El formato es semi-planario (Y plano + UV intercalado)?
    pub(crate) fn is_semiplanar(self) -> bool {
        matches!(self, PixelFormat::Nv12 | PixelFormat::Nv21)
    }
}