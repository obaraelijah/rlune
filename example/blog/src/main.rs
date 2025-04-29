mod auth_models;

use std::any::type_name;
use std::net::SocketAddr;
use std::str::FromStr;

use rlune::contrib::auth::AuthModule;
use rlune::{get, Rlune};
use std::any::Any;
use std::marker::PhantomData;
use std::panic;
use std::panic::Location;

use crate::auth_models::AuthModels;
use rlune::contrib::auth;
use rlune::core::RluneRouter;
use tracing::error;

#[get("/index")]
async fn test<const N: usize, T: 'static>() -> String {
    format!("<{N}, {}>", type_name::<T>())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_panic_hook();

    Rlune::init()
        .register_module::<AuthModule>()
        .add_routes(
            RluneRouter::new().nest(
                "/auth",
                RluneRouter::new()
                    .handler(auth::handler::get_login_flow::<AuthModels>(PhantomData))
                    .handler(auth::handler::login_oidc::<AuthModels>(PhantomData))
                    .handler(auth::handler::finish_login_oidc::<AuthModels>(PhantomData))
                    .handler(auth::handler::login_local_webauthn::<AuthModels>(
                        PhantomData,
                    ))
                    .handler(auth::handler::finish_login_local_webauthn::<AuthModels>(
                        PhantomData,
                    ))
                    .handler(auth::handler::login_local_password::<AuthModels>(
                        PhantomData,
                    ))
                    .handler(auth::handler::logout(PhantomData)),
            ),
        )
        .start(SocketAddr::from_str("127.0.0.1:8080")?)
        .await?;

    Ok(())
}

/// Initializes the global panic hook to output tracing events instead of writing to stdout
pub fn init_tracing_panic_hook() {
    panic::set_hook(Box::new(panic_hook))
}

/// The panic hook set by [`init_tracing_panic_hook`]
fn panic_hook(info: &panic::PanicHookInfo) {
    let msg = payload_as_str(info.payload());
    let location = info.location();

    error!(
        panic.file = location.map(Location::file),
        panic.line = location.map(Location::line),
        panic.column = location.map(Location::column),
        panic.msg = msg,
        "Panic"
    );
}

/// Copied from the std's default hook (v1.81.0)
fn payload_as_str(payload: &dyn Any) -> &str {
    if let Some(&s) = payload.downcast_ref::<&'static str>() {
        s
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.as_str()
    } else {
        "Box<dyn Any>"
    }
}