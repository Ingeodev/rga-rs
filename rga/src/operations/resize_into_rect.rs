use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

/// Redimensiona `src` (usando su dimensión completa) y escribe el resultado
/// únicamente dentro de `dst_rect` sobre `dst`, sin tocar el resto del buffer.
/// A diferencia de `resize`, `dst` puede ser más grande que el resultado —
/// es la operación base para letterbox.
pub fn resize_into_rect(
    src: &RgaBuffer,
    dst: &RgaBuffer,
    dst_rect: (i32, i32, i32, i32),
) -> Result<(), RgaError> {
    let (x, y, width, height) = dst_rect;

    let src_rect = im_rect { x: 0, y: 0, width: src.width() as i32, height: src.height() as i32 };
    let drect = im_rect { x, y, width, height };
    let empty_rect = im_rect { x: 0, y: 0, width: 0, height: 0 };

    let status = unsafe {
        improcess(
            src.raw(),
            dst.raw(),
            std::mem::zeroed(), // pat: no se usa un patrón de blend aquí
            src_rect,
            drect,
            empty_rect,
            0, // usage=0: resize/escalado puro, sin flags de blend/rop/etc.
        )
    };
    RgaError::check(status)
}