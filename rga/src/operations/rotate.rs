use rga_sys::*;
use crate::{buffer::RgaBuffer, error::RgaError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    Rot90,
    Rot180,
    Rot270,
}

impl Rotation {
    fn as_raw(self) -> std::os::raw::c_int {
        match self {
            // el binding no trae constantes separadas para rotación en imrotate_t;
            // usa las mismas IM_USAGE de transform que las de flip/blend
            Rotation::Rot90  => IM_USAGE_IM_HAL_TRANSFORM_ROT_90 as i32,
            Rotation::Rot180 => IM_USAGE_IM_HAL_TRANSFORM_ROT_180 as i32,
            Rotation::Rot270 => IM_USAGE_IM_HAL_TRANSFORM_ROT_270 as i32,
        }
    }
}

pub fn rotate(src: &RgaBuffer, dst: &RgaBuffer, rotation: Rotation) -> Result<(), RgaError> {
    let status = unsafe { imrotate_t(src.raw(), dst.raw(), rotation.as_raw(), 1) };
    RgaError::check(status)
}