use mime::Mime;
use schemars::schema::Schema;

use crate::handler::request_part::RequestPart;
use crate::handler::request_part::ShouldBeRequestPart;
<<<<<<< HEAD:rlune-core/src/handler/request_body.rs
use crate::macro_utils::type_metadata::HasMetadata;
use crate::macro_utils::type_metadata::ShouldHaveMetadata;
=======
>>>>>>> 1f796685028b63c8017575160e850f6d68661856:swaggapi/src/handler/request_body.rs
use crate::schema_generator::SchemaGenerator;

/// Describes the behaviour of a type implementing [`FromRequest`](axum::extract::FromRequest)
pub trait RequestBody: ShouldBeRequestBody {
    fn body(_gen: &mut SchemaGenerator) -> (Mime, Option<Schema>);
}

pub trait ShouldBeRequestBody {}

#[derive(Clone, Debug)]
pub struct RequestBodyMetadata {
    pub body: fn(&mut SchemaGenerator) -> (Mime, Option<Schema>),
}

impl<T: ShouldBeRequestBody> ShouldHaveMetadata<RequestBodyMetadata> for T {}
impl<T: RequestBody> HasMetadata<RequestBodyMetadata> for T {
    fn metadata() -> RequestBodyMetadata {
        RequestBodyMetadata { body: T::body }
    }
}

impl<T: ShouldBeRequestBody> ShouldBeRequestPart for T {}
impl<T: RequestBody> RequestPart for T {}
