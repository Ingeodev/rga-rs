use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError, format::PixelFormat};

/// Modo de conversión de espacio de color (`IM_COLOR_SPACE_MODE`).
///
/// Mapeo directo de los valores de los headers de librga. `Default` conserva
/// el comportamiento previo de `convert_color` (versiones anteriores).
///
/// Reglas de oro en este pipeline:
/// - RGB ↔ YUV: siempre explícito (BT601 limit es el estándar de cámaras ISP).
/// - YUV → YUV (p. ej. `Uyvy422` → `Nv12`): usar el modo *range* que indique
///   el rango real del origen (`YuvBt601LimitRange` para cámara).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ColorSpaceMode {
    Default             = IM_COLOR_SPACE_MODE_IM_COLOR_SPACE_DEFAULT,
    // RGB → YUV
    RgbToYuvBt601Full   = IM_COLOR_SPACE_MODE_IM_RGB_TO_YUV_BT601_FULL,
    RgbToYuvBt601Limit  = IM_COLOR_SPACE_MODE_IM_RGB_TO_YUV_BT601_LIMIT,
    RgbToYuvBt709Limit  = IM_COLOR_SPACE_MODE_IM_RGB_TO_YUV_BT709_LIMIT,
    // YUV → RGB
    YuvToRgbBt601Limit  = IM_COLOR_SPACE_MODE_IM_YUV_TO_RGB_BT601_LIMIT,
    YuvToRgbBt601Full   = IM_COLOR_SPACE_MODE_IM_YUV_TO_RGB_BT601_FULL,
    YuvToRgbBt709Limit  = IM_COLOR_SPACE_MODE_IM_YUV_TO_RGB_BT709_LIMIT,
    // Rango YUV (para conversiones YUV → YUV / interpretación del origen)
    YuvBt601LimitRange  = IM_COLOR_SPACE_MODE_IM_YUV_BT601_LIMIT_RANGE,
    YuvBt601FullRange   = IM_COLOR_SPACE_MODE_IM_YUV_BT601_FULL_RANGE,
    YuvBt709LimitRange  = IM_COLOR_SPACE_MODE_IM_YUV_BT709_LIMIT_RANGE,
    YuvBt709FullRange   = IM_COLOR_SPACE_MODE_IM_YUV_BT709_FULL_RANGE,
}

impl ColorSpaceMode {
    pub(crate) fn as_raw(self) -> std::os::raw::c_uint {
        self as u32
    }
}

/// Conversión de color con espacio explícito (recomendado).
pub fn convert_color_with_mode(
    src: &RgaBuffer,
    dst: &RgaBuffer,
    src_fmt: PixelFormat,
    dst_fmt: PixelFormat,
    mode: ColorSpaceMode,
) -> Result<(), RgaError> {
    let status = unsafe {
        imcvtcolor_t(
            src.raw(), dst.raw(),
            src_fmt.as_raw(), dst_fmt.as_raw(),
            mode.as_raw() as i32, 1,
        )
    };
    RgaError::check(status)
}

/// Conversión de color con `ColorSpaceMode::Default`.
pub fn convert_color(src: &RgaBuffer, dst: &RgaBuffer, src_fmt: PixelFormat, dst_fmt: PixelFormat) -> Result<(), RgaError> {
    convert_color_with_mode(src, dst, src_fmt, dst_fmt, ColorSpaceMode::Default)
}
