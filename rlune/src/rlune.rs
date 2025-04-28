use std::net::SocketAddr;
use std::{io, mem};

use crate::core::Module;
use crate::error::RluneError;
use axum::Router;
use rlune_core::registry::builder::RegistryBuilder;
use rlune_core::RluneRouter;
use tokio::net::TcpListener;
use tracing::info;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Default)]
pub struct Rlune {
    modules: RegistryBuilder,
    routes: RluneRouter,
}

impl Rlune {
    pub fn init() -> Self {
        let registry = tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(Level::INFO.as_str())))
            .with(tracing_subscriber::fmt::layer());

        registry.init();

        Self::default()
    }

    /// Register a module
    pub fn register_module<T: Module>(&mut self) -> &mut Self {
        self.modules.register_module::<T>();
        self
    }

    /// Initializes all modules and start the webserver
    pub async fn start(&mut self, socket_addr: SocketAddr) -> Result<(), RluneError> {
        self.modules.init().await.map_err(io::Error::other)?;

        let router = Router::from(mem::take(&mut self.routes));

        let socket = TcpListener::bind(socket_addr).await?;

        info!("Starting to serve webserver on http://{socket_addr}");
        axum::serve(socket, router).await?;

        Ok(())
    }
}