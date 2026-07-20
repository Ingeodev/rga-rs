use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

pub fn resize(src: &RgaBuffer, dst: &RgaBuffer, fx: f64, fy: f64) -> Result<(), RgaError> {
    let status = unsafe {
        imresize_t(src.raw(), dst.raw(), fx, fy, IM_INTER_MODE_IM_INTERP_DEFAULT as i32, 1)
    };
    RgaError::check(status)
}