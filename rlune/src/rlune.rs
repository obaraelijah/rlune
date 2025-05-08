use crate::core::Module;
use crate::error::RluneError;
use rlune_core::re_exports::rorm::Database;
use rlune_core::registry::builder::RegistryBuilder;
use rlune_core::router::MutHandlerMeta;
use rlune_core::session;
use rlune_core::RluneRouter;
use std::mem;
use std::net::SocketAddr;
use std::sync::RwLock;
use tokio::net::TcpListener;
use tracing::info;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[non_exhaustive]
pub struct Rlune;

impl Rlune {
    pub fn new() -> ModuleBuilder {
        ModuleBuilder::new()
    }
}

#[derive(Default)]
pub struct ModuleBuilder {
    modules: RegistryBuilder,
}

impl ModuleBuilder {
    fn new() -> ModuleBuilder {
        let registry = tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(Level::INFO.as_str())))
            .with(tracing_subscriber::fmt::layer());

        registry.init();

        let mut this = ModuleBuilder::default();
        this.register_module::<Database>();
        this
    }

    /// Register a module
    pub fn register_module<T: Module>(&mut self) -> &mut Self {
        self.modules.register_module::<T>();
        self
    }

    pub async fn init_modules(&mut self) -> Result<RouterBuilder, RluneError> {
        self.modules.init().await?;
        Ok(RouterBuilder {
            routes: RluneRouter::new(),
        })
    }
}

pub struct RouterBuilder {
    routes: RluneRouter,
}

impl RouterBuilder {
    /// Adds a router to the builder
    pub fn add_routes(&mut self, router: RluneRouter) -> &mut Self {
        let this = mem::take(&mut self.routes);
        self.routes = this.merge(router);
        self
    }

    /// Starts the webserver
    pub async fn start(&mut self, socket_addr: SocketAddr) -> Result<(), RluneError> {
        let (router, handlers) = mem::take(&mut self.routes).finish();

        *HANDLERS.write().unwrap() = handlers.leak();

        let socket = TcpListener::bind(socket_addr).await?;

        info!("Starting to serve webserver on http://{socket_addr}");
        axum::serve(socket, router.layer(session::layer())).await?;

        Ok(())
    }
}


static HANDLERS: RwLock<&'static [MutHandlerMeta]> = RwLock::new(&[]);


/// Quick and dirty solution to expose the registered handlers after startup
#[doc(hidden)]
pub fn get_routes() -> &'static [MutHandlerMeta] {
    *HANDLERS.read().unwrap()
}