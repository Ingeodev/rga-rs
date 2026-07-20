use rga_sys::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RgaError {
    #[error("RGA operation failed (status={0})")]
    Failed(i32),
    #[error("operation not supported by this RGA core")]
    NotSupported,
    #[error("RGA ran out of memory")]
    OutOfMemory,
    #[error("invalid parameter")]
    InvalidParam,
    #[error("illegal parameter combination")]
    IllegalParam,
    #[error("RGA API version mismatch")]
    VersionError,
    #[error("no active RGA session")]
    NoSession,
    #[error("failed to import buffer: driver returned null handle")]
    ImportFailed,
}

impl RgaError {
    /// Traduce el IM_STATUS crudo del binding a Result<(), RgaError>.
    /// Trata tanto SUCCESS como NOERROR como éxito (ver nota abajo).
    pub(crate) fn check(status: IM_STATUS) -> Result<(), RgaError> {
        match status {
            IM_STATUS_IM_STATUS_SUCCESS | IM_STATUS_IM_STATUS_NOERROR => Ok(()),
            IM_STATUS_IM_STATUS_NOT_SUPPORTED => Err(RgaError::NotSupported),
            IM_STATUS_IM_STATUS_OUT_OF_MEMORY => Err(RgaError::OutOfMemory),
            IM_STATUS_IM_STATUS_INVALID_PARAM => Err(RgaError::InvalidParam),
            IM_STATUS_IM_STATUS_ILLEGAL_PARAM => Err(RgaError::IllegalParam),
            IM_STATUS_IM_STATUS_ERROR_VERSION => Err(RgaError::VersionError),
            IM_STATUS_IM_STATUS_NO_SESSION => Err(RgaError::NoSession),
            _ => Err(RgaError::Failed(status)),
        }
    }
}