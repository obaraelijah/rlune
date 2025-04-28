use std::collections::HashMap;

use rorm::fields::types::Json;
use rorm::internal::field::Field;
use rorm::internal::field::FieldProxy;
use rorm::Database;
use rorm::Model;
use rorm::Patch;
use schemars::_serde_json::Value;
use tower_sessions::cookie::time::Duration;
use tower_sessions::cookie::time::OffsetDateTime;
use tower_sessions::cookie::SameSite;
use tower_sessions::Expiry;
pub use tower_sessions::Session;
use tower_sessions::SessionManagerLayer;
use tower_sessions_rorm_store::RormStore;
use tower_sessions_rorm_store::SessionModel;

use crate::Module;

pub fn layer() -> SessionManagerLayer<RormStore<RluneSession>> {
    SessionManagerLayer::new(RormStore::<RluneSession>::new(Database::global().clone()))
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)))
        .with_same_site(SameSite::Lax)
}

#[derive(Model)]
pub struct RluneSession {
    #[rorm(primary_key, max_length = 255)]
    id: String,
    expires_at: OffsetDateTime,
    data: Json<HashMap<String, Value>>,
}

impl SessionModel for RluneSession {
    fn get_expires_at_field() -> FieldProxy<impl Field<Type = OffsetDateTime, Model = Self>, Self> {
        RluneSession::F.expires_at
    }

    fn get_data_field(
    ) -> FieldProxy<impl Field<Type = Json<HashMap<String, Value>>, Model = Self>, Self> {
        RluneSession::F.data
    }

    fn get_insert_patch(
        id: String,
        expires_at: OffsetDateTime,
        data: Json<HashMap<String, Value>>,
    ) -> impl Patch<Model = Self> + Send + Sync + 'static {
        RluneSession {
            id,
            data,
            expires_at,
        }
    }

    fn get_session_data(&self) -> (String, OffsetDateTime, Json<HashMap<String, Value>>) {
        (self.id.clone(), self.expires_at.clone(), self.data.clone())
    }
}
