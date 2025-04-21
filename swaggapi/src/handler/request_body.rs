use mime::Mime;
use schemars::schema::Schema;

use crate::handler::request_part::RequestPart;
use crate::handler::request_part::ShouldBeRequestPart;
use crate::schema_generator::SchemaGenerator;
use crate::type_metadata::HasMetadata;
use crate::type_metadata::ShouldHaveMetadata;

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
