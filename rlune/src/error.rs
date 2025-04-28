use std::io;

use thiserror::Error;

/// Error type for rlune
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum RluneError {
    #[error("{0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Init(#[from] rlune_core::module::registry::builder::InitError),
}
