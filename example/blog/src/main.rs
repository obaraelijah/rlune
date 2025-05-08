use std::any::type_name;
use std::any::Any;
use std::net::SocketAddr;
use std::panic;
use std::panic::Location;
use std::str::FromStr;

use rlune::contrib::auth::AuthModule;
use rlune::core::re_exports::axum::response::IntoResponse;
use rlune::core::re_exports::axum::response::Response;
use rlune::core::re_exports::axum::Json;
use rlune::core::Module;
use rlune::core::RluneRouter;
use rlune::get;
use rlune::Rlune;
use tracing::error;

#[get("/index")]
async fn test<const N: usize, T: 'static>() -> String {
    format!("<{N}, {}>", type_name::<T>())
}

#[get("/openapi")]
async fn openapi() -> Response {
    Json(rlune::openapi::get_openapi()).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_panic_hook();

    Rlune::new()
        .register_module::<AuthModule>()
        .init_modules()
        .await?
        .add_routes(RluneRouter::new().nest("/auth", AuthModule::global().handler.as_router()))
        .add_routes(
            RluneRouter::new()
                .handler(test::<1337, ()>)
                .handler(openapi),
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
