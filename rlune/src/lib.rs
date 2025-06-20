#[cfg(feature = "contrib")]
pub mod contrib {
    pub use rlune_contrib_auth as auth;
    // pub use rlune_contrib_tracing as tracing;
}

/// Re-export of [`rorm`](rlune_core::re_exports::rorm)
pub mod rorm {
    pub use rlune_core::re_exports::rorm::*;
    /// Re-export from [`rorm`](rlune_core::re_exports::rorm::DbEnum)
    pub use rlune_macros::DbEnum;
    /// Re-export from [`rorm`](rlune_core::re_exports::rorm::Model)
    pub use rlune_macros::Model;
    /// Re-export from [`rorm`](rlune_core::re_exports::rorm::Patch)
    pub use rlune_macros::Patch;
}

pub mod core {
    pub use rlune_core::*;
}

pub use crate::rlune::*;

pub mod error;
#[cfg(feature = "graceful-shutdown")]
mod graceful_shutdown;
mod macro_docs;
#[cfg(feature = "openapi")]
pub mod openapi;
#[cfg(feature = "panic-hook")]
pub mod panic_hook;
mod rlune;

pub use macro_docs::*;
