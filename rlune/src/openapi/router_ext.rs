//! [`RluneRouter`] extension trait

use rlune_core::RluneRouter;

use crate::openapi::metadata::OpenapiMetadata;

/// Extension trait for [`RluneRouter`]
///
/// It provides convenient methods for adding openapi related metadata
/// to a route. (For example tags)
pub trait OpenapiRouterExt {
    /// Adds a tag to all handlers in this router
    fn openapi_tag(self, tag: &'static str) -> Self;

    /// Creates a new router with a tag
    ///
    /// (Shorthand for `RluneRouter::new().openapi_tag(...)`)
    fn with_openapi_tag(tag: &'static str) -> Self;
}

impl OpenapiRouterExt for RluneRouter {
    fn openapi_tag(self, tag: &'static str) -> Self {
        self.metadata(OpenapiMetadata { tags: vec![tag] })
    }

    fn with_openapi_tag(tag: &'static str) -> Self {
        Self::new().openapi_tag(tag)
    }
}
