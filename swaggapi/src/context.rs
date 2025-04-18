use std::convert::Infallible;
use std::ops::Deref;

use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::Route;
use axum::routing::Router;
use tower::Layer;
use tower::Service;

use crate::handler::Handler;
use crate::handler::HandlerMeta;
use crate::internals::ptrset::PtrSet;
use crate::SwaggapiPage;
use crate::PAGE_OF_EVERYTHING;

/// An `ApiContext` combines several [`SwaggapiHandler`] under a common path.
///
/// It is also responsible for adding them to [`SwaggapiPage`]s once mounted to your application.
#[derive(Debug)]
pub struct ApiContext {
    /* The same collection of handlers in swaggapi and framework specific representation */
    /// The contained handlers
    handlers: Vec<ContextHandler>,
    /// The underlying axum router
    router: Router,

    /* Parameters added to new handlers */
    /// A base path all handlers are routed under
    ///
    /// This is effectively remembers the argument actix' `Scope` was created with.
    /// Since `Router` doesn't take a path, this will always be empty for axum.
    path: String,

    /// Changes have to be applied to already existing `handlers` manually
    pages: Vec<&'static SwaggapiPage>,

    /// Changes have to be applied to already existing `handlers` manually
    tags: Vec<&'static str>,
}

impl ApiContext {
    /// Create a new context
    ///
    /// It wraps an axum [`Router`] internally and should be added to your application's router using [`Router::merge`]:
    /// ```rust
    /// # use axum::Router;
    /// # use swaggapi::ApiContext;
    /// let app = Router::new().merge(ApiContext::new("/api"));
    /// ```
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            router: Router::new(),
            path: String::new(),
            pages: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Create a new context with a tag
    ///
    /// (Shorthand for `ApiContext::new().tag(...)`)
    pub fn with_tag(tag: &'static str) -> Self {
        Self::new().tag(tag)
    }

    /// Add a handler to the context
    pub fn handler(mut self, handler: impl Handler) -> Self {
        self.push_handler(ContextHandler::new(handler.meta()));
        self.router = self
            .router
            .route(&handler.meta().path, handler.method_router());
        self
    }

    /// Attach a [`SwaggapiPage`] this context's handlers will be added to
    pub fn page(mut self, page: &'static SwaggapiPage) -> Self {
        self.pages.push(page);
        for handler in &mut self.handlers {
            handler.pages.insert(page);
        }
        self
    }

    /// Add a tag to all of this context's handlers
    pub fn tag(mut self, tag: &'static str) -> Self {
        self.tags.push(tag);
        for handler in &mut self.handlers {
            handler.tags.insert(tag);
        }
        self
    }

    /// Adds a [`ContextHandler`] after adding this context's `path`, `tags` and `pages` to it
    fn push_handler(&mut self, mut handler: ContextHandler) {
        if !self.path.is_empty() {
            handler.path = format!("{}{}", self.path, handler.path);
        }
        handler.tags.extend(self.tags.iter().copied());
        handler.pages.extend(self.pages.iter().copied());
        self.handlers.push(handler);
    }

    /// Adds the handlers to their api pages and returns the contained framework impl
    fn finish(self) -> Router {
        for mut handler in self.handlers {
            handler.path = framework_path_to_openapi(handler.path);

            PAGE_OF_EVERYTHING.add_handler(&handler);
            for page in handler.pages.iter() {
                page.add_handler(&handler);
            }
        }
        return self.router;

        /// Converts the framework's syntax for path parameters into openapi's

        fn framework_path_to_openapi(framework_path: String) -> String {
            use std::borrow::Cow;
            use std::sync::OnceLock;

            use regex::Regex;

            static RE: OnceLock<Regex> = OnceLock::new();

            let regex = RE.get_or_init(|| Regex::new(":([^/]*)").unwrap());
            match regex.replace_all(&framework_path, "{$1}") {
                Cow::Borrowed(_) => framework_path,
                Cow::Owned(new_path) => new_path,
            }
        }
    }

    /// Calls [`Router::nest`] while preserving api information
    pub fn nest(mut self, path: &str, other: ApiContext) -> Self {
        for mut handler in other.handlers {
            // Code taken from `path_for_nested_route` in `axum/src/routing/path_router.rs`
            handler.path = if path.ends_with('/') {
                format!("{path}{}", handler.path.trim_start_matches('/'))
            } else if handler.path == "/" {
                path.into()
            } else {
                format!("{path}{}", handler.path)
            };

            self.push_handler(handler);
        }
        self.router = self.router.nest(path, other.router);
        self
    }

    /// Calls [`Router::merge`] while preserving api information
    pub fn merge(mut self, other: ApiContext) -> Self {
        for handler in other.handlers {
            self.push_handler(handler);
        }
        self.router = self.router.merge(other.router);
        self
    }

    /// Apply a [`tower::Layer`] to all routes in the context.
    ///
    /// See [`Router::layer`] for more details.
    pub fn layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.router = self.router.layer(layer);
        self
    }

    /// Apply a [`tower::Layer`] to the context that will only run if the request matches a route.
    ///
    /// See [`Router::route_layer`] for more details.
    pub fn route_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.router = self.router.route_layer(layer);
        self
    }
}

impl From<ApiContext> for Router {
    fn from(context: ApiContext) -> Self {
        context.finish()
    }
}

/// A wrapped [`HandlerMeta`] used inside [`ApiContext`] to allow modifications.
#[derive(Debug)]
pub(crate) struct ContextHandler {
    /// The original unmodified [`HandlerMeta`]
    pub original: HandlerMeta,

    /// The handler's modified path
    pub path: String,

    /// The handler's modified path
    pub tags: PtrSet<'static, str>,

    /// The pages the handler should be added to
    pub pages: PtrSet<'static, SwaggapiPage>,
}
impl ContextHandler {
    /// Constructs a new `ContextHandler`
    pub fn new(original: HandlerMeta) -> Self {
        Self {
            path: original.path.to_string(),
            tags: PtrSet::from_iter(original.tags.iter().copied()),
            pages: PtrSet::new(),
            original,
        }
    }
}
impl Deref for ContextHandler {
    type Target = HandlerMeta;

    fn deref(&self) -> &Self::Target {
        &self.original
    }
}
