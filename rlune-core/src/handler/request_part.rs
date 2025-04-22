use crate::macro_utils::type_metadata::HasMetadata;
use crate::macro_utils::type_metadata::ShouldHaveMetadata;

/// Describes the behaviour of a type implementing [`FromRequestParts`](axum::extract::FromRequestParts)
pub trait RequestPart: ShouldBeRequestPart {}

pub trait ShouldBeRequestPart {}

#[derive(Clone, Debug)]
pub struct RequestPartMetadata {}

impl<T: ShouldBeRequestPart> ShouldHaveMetadata<RequestPartMetadata> for T {}
impl<T: RequestPart> HasMetadata<RequestPartMetadata> for T {
    fn metadata() -> RequestPartMetadata {
        RequestPartMetadata {}
    }
}
