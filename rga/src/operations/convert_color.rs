use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError, format::PixelFormat};

pub fn convert_color(src: &RgaBuffer, dst: &RgaBuffer, src_fmt: PixelFormat, dst_fmt: PixelFormat) -> Result<(), RgaError> {
    let status = unsafe {
        imcvtcolor_t(
            src.raw(), dst.raw(),
            src_fmt.as_raw(), dst_fmt.as_raw(),
            IM_COLOR_SPACE_MODE_IM_COLOR_SPACE_DEFAULT as i32, 1,
        )
    };
    RgaError::check(status)
}