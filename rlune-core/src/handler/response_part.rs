use axum::http::HeaderName;

<<<<<<< HEAD:rlune-core/src/handler/response_part.rs
use crate::macro_utils::type_metadata::HasMetadata;
use crate::macro_utils::type_metadata::ShouldHaveMetadata;
=======
use crate::type_metadata::HasMetadata;
use crate::type_metadata::ShouldHaveMetadata;
>>>>>>> 1f796685028b63c8017575160e850f6d68661856:swaggapi/src/handler/response_part.rs

/// Describes the behaviour of a type implementing [`IntoResponseParts`](axum::response::IntoResponseParts)
pub trait ResponsePart: ShouldBeResponsePart {
    fn header() -> Vec<HeaderName>;
}

pub trait ShouldBeResponsePart {}

#[derive(Clone, Debug)]

pub struct ResponsePartMetadata {
    pub header: fn() -> Vec<HeaderName>,
}

impl<T: ShouldBeResponsePart> ShouldHaveMetadata<ResponsePartMetadata> for T {}
impl<T: ResponsePart> HasMetadata<ResponsePartMetadata> for T {
    fn metadata() -> ResponsePartMetadata {
        ResponsePartMetadata { header: T::header }
    }
}
