// #![warn(missing_docs)]
// #![warn(clippy::todo)]

pub mod as_responses;
mod context;
pub mod handler;
pub mod handler_argument;
pub mod internals;
mod page;
pub mod utils;

pub use self::context::ApiContext;
pub use self::page::SwaggapiPage;
pub use self::page::PAGE_OF_EVERYTHING;

/// Reexports for macros and implementors
pub mod re_exports {
    pub use axum;
    pub use indexmap;
    pub use mime;
    pub use openapiv3;
    pub use schemars;
}
