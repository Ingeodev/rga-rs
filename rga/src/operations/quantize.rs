use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

/// Parámetros de normalización por canal RGB, aplicados como:
/// output = input * scale + offset
///
/// Ejemplos comunes:
/// - Normalización [0,255] -> [0,1]: scale = 1/255, offset = 0 en los 3 canales
/// - Normalización estilo ImageNet: scale = 1/(255*std), offset = -mean/std por canal
#[derive(Debug, Clone, Copy)]
pub struct QuantizeParams {
    pub scale_r: i32,
    pub scale_g: i32,
    pub scale_b: i32,
    pub offset_r: i32,
    pub offset_g: i32,
    pub offset_b: i32,
}

impl QuantizeParams {
    fn as_raw(self) -> im_nn_t {
        im_nn {
            scale_r: self.scale_r,
            scale_g: self.scale_g,
            scale_b: self.scale_b,
            offset_r: self.offset_r,
            offset_g: self.offset_g,
            offset_b: self.offset_b,
        }
    }
}

pub fn quantize(src: &RgaBuffer, dst: &RgaBuffer, params: QuantizeParams) -> Result<(), RgaError> {
    let status = unsafe { imquantize_t(src.raw(), dst.raw(), params.as_raw(), 1) };
    RgaError::check(status)
}