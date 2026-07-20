mod error;
mod format;
mod buffer;
mod operations;

pub use error::RgaError;
pub use format::PixelFormat;
pub use buffer::{RgaBuffer, ImageLayout};   // ← sin esta línea, el error es exactamente el que tienes
pub use operations::*;
pub use rga_sys;