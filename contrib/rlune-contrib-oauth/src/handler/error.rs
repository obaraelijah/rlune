use std::panic::Location;

use rlune_core::handler::response_body::ResponseBody;
use rlune_core::handler::response_body::ShouldBeResponseBody;
use rlune_core::re_exports::axum::http::StatusCode;
use rlune_core::re_exports::axum::response::IntoResponse;
use rlune_core::re_exports::axum::response::Redirect;
use rlune_core::re_exports::axum::response::Response;
use rlune_core::re_exports::mime::Mime;
use rlune_core::schema_generator::SchemaGenerator;
use rlune_core::stuff::api_error::ApiError;
use rlune_core::stuff::api_json::ApiJson;
use rlune_core::stuff::schema::ApiStatusCode;
use schemars::schema::Schema;
use serde::Serialize;
use tracing::info;
use url::Url;

use crate::handler::schema::AuthError;
use crate::handler::schema::AuthErrorType;
use crate::handler::schema::AuthRequest;

pub struct OauthErrorBuilder {
    redirect_uri: Option<Url>,
    state: Option<String>,
}

impl OauthErrorBuilder {
    /// Constructs a new `OauthErrorBuilder`
    /// by parsing an `AuthRequest`'s `redirect_uri` and cloning its `state`.
    pub fn from_request(request: &AuthRequest) -> OauthResult<Self> {
        Ok(Self {
            redirect_uri: request
                .redirect_uri
                .as_deref()
                .map(Url::parse)
                .transpose()
                .map_err(|error| {
                    info!(
                        redirect_uri = request.redirect_uri.as_deref(),
                        error.debug = ?error,
                        error.display = %error,
                        "Oauth client set invalid `redirect_uri`"
                    );
                    OauthError {
                        redirect_uri: None,
                        state: None,
                        error: AuthErrorType::InvalidRequest,
                        description: Some("Invalid redirect url"),
                    }
                })?,
            state: request.state.clone(),
        })
    }

    /// Constructs a new `OauthError`
    pub fn new_error(&self, error: AuthErrorType, description: &'static str) -> OauthError {
        OauthError {
            redirect_uri: self.redirect_uri.clone(),
            state: self.state.clone(),
            error,
            description: Some(description),
        }
    }

    /// Constructs a closure wrapping a `rorm::Error`
    ///
    /// The returned closure will emit an error log message.
    #[track_caller]
    pub fn map_rorm_error(&self) -> impl Fn(rorm::Error) -> OauthError {
        let location = Location::caller();
        |error: rorm::Error| {
            ApiError {
                code: ApiStatusCode::InternalServerError,
                context: None,
                location,
                source: Some(error.into()),
            }
            .emit_tracing_event();
            self.new_error(AuthErrorType::ServerError, "Internal server error")
        }
    }
}

pub type OauthResult<T> = Result<T, OauthError>;

pub struct OauthError {
    redirect_uri: Option<Url>,
    state: Option<String>,

    error: AuthErrorType,
    description: Option<&'static str>,
}

impl IntoResponse for OauthError {
    fn into_response(self) -> Response {
        if let Some(mut redirect_uri) = self.redirect_uri {
            // Add query parameters to `redirect_uri`
            AuthError {
                error: self.error,
                state: self.state,
                error_description: self.description,
            }
            .serialize(serde_urlencoded::Serializer::new(
                &mut redirect_uri.query_pairs_mut(),
            ))
            .unwrap_or_else(|_| unreachable!("The AuthError struct should always be serializable"));

            Redirect::temporary(redirect_uri.as_str()).into_response()
        } else {
            ApiJson(AuthError {
                error: self.error,
                state: self.state,
                error_description: self.description,
            })
            .into_response()
        }
    }
}

impl ShouldBeResponseBody for OauthError {}

impl ResponseBody for OauthError {
    fn body(generator: &mut SchemaGenerator) -> Vec<(StatusCode, Option<(Mime, Option<Schema>)>)> {
        let mut body = ApiJson::<AuthError>::body(generator);
        body.insert(0, (StatusCode::TEMPORARY_REDIRECT, None));
        body
    }
}
