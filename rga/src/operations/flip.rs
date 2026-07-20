use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipMode {
    Horizontal,
    Vertical,
    Both,
}

impl FlipMode {
    fn as_raw(self) -> std::os::raw::c_int {
        match self {
            FlipMode::Horizontal => IM_USAGE_IM_HAL_TRANSFORM_FLIP_H as i32,
            FlipMode::Vertical   => IM_USAGE_IM_HAL_TRANSFORM_FLIP_V as i32,
            FlipMode::Both       => IM_USAGE_IM_HAL_TRANSFORM_FLIP_H_V as i32,
        }
    }
}

pub fn flip(src: &RgaBuffer, dst: &RgaBuffer, mode: FlipMode) -> Result<(), RgaError> {
    let status = unsafe { imflip_t(src.raw(), dst.raw(), mode.as_raw(), 1) };
    RgaError::check(status)
}