use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

pub fn crop(src: &RgaBuffer, dst: &RgaBuffer, x: i32, y: i32, w: i32, h: i32) -> Result<(), RgaError> {
    let rect = im_rect { x, y, width: w, height: h };

    let status = unsafe {
        imcrop_t(src.raw(), dst.raw(), rect, 1)
    };
    RgaError::check(status)
}
