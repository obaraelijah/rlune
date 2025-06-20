use std::any::type_name;
use std::net::SocketAddr;
use std::str::FromStr;

use rlune::contrib::auth::AuthModule;
use rlune::core::re_exports::axum::response::IntoResponse;
use rlune::core::re_exports::axum::response::Response;
use rlune::core::re_exports::axum::Json;
use rlune::core::Module;
use rlune::core::RluneRouter;
use rlune::get;
use rlune::openapi::OpenapiRouterExt;
use rlune::rorm::Database;
use rlune::Rlune;

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
    Rlune::new()
        .register_module::<Database>(Default::default())
        .register_module::<AuthModule>(Default::default())
        .init_modules()
        .await?
        .add_routes(
            RluneRouter::with_openapi_tag("Auth Module")
                .nest("/auth", AuthModule::global().handler.as_router()),
        )
        .add_routes(
            RluneRouter::new()
                .openapi_tag("Main")
                .handler(test::<1337, ()>)
                .handler(openapi),
        )
        .start(SocketAddr::from_str("127.0.0.1:8080")?)
        .await?;

    Ok(())
}
