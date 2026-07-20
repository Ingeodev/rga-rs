use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

/// Rellena un rectángulo del buffer destino con un color sólido.
/// `rect` es (x, y, width, height) en coordenadas del buffer.
/// `color` es el valor empaquetado según el formato del buffer
/// (por ejemplo, para RGB888: 0x00RRGGBB).
pub fn fill(dst: &RgaBuffer, rect: (i32, i32, i32, i32), color: u32) -> Result<(), RgaError> {
    let (x, y, width, height) = rect;
    let im_rect_val = im_rect { x, y, width, height };
    let status = unsafe { imfill_t(dst.raw(), im_rect_val, color as i32, 1) };
    RgaError::check(status)
}