use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use futures_concurrency::future::Join;
use futures_lite::future;
use tokio::task::JoinError;
use tokio::task::JoinHandle;

use crate::module;
use crate::module::registry::module_set::OwnedModulesSet;
use crate::module::registry::DynModule;
use crate::module::registry::ModuleDependencies;
use crate::module::registry::Registry;
use crate::module::Module;

pub struct RegistryBuilder {
    modules: HashMap<TypeId, UninitModule>,
}

impl RegistryBuilder {
    /// Constructs a new `RegistryBuilder`
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Adds a new module to the `RegistryBuilder`
    ///
    /// Calling this method twice with the same `T` is not an error but will only add it once.
    pub fn register_module<T: Module>(&mut self) -> &mut Self {
        if let Entry::Vacant(entry) = self.modules.entry(TypeId::of::<T>()) {
            entry.insert(Box::new(|| {
                tokio::spawn(async {
                    let pre_init = T::pre_init().await?;
                    Ok(BoxDynFnOnce::new(move |mut modules: OwnedModulesSet| {
                        Box::pin(async move {
                            let mut dependencies =
                                <T::Dependencies as ModuleDependencies>::take(&mut modules);
                            let t = T::init(pre_init, &mut dependencies).await?;
                            <T::Dependencies as ModuleDependencies>::put_back(
                                dependencies,
                                &mut modules,
                            );
                            modules.insert(t);
                            Ok(modules)
                        }) as future::Boxed<_>
                    }))
                })
            }) as UninitModule);
            <T::Dependencies as ModuleDependencies>::register(self);
        }
        self
    }

    /// Initialized all registered modules
    ///
    /// and makes the registry available through [`Registry::global`].
    pub async fn init(&mut self) -> Result<(), InitError> {
        let pre_init_modules = process_join_results(
            self.modules
                .drain()
                .map(|(_, x)| x())
                .collect::<Vec<_>>()
                .join()
                .await,
        )
        .map_err(InitError::PreInit)?;

        let mut modules = OwnedModulesSet::new();
        for pre_init_module in pre_init_modules {
            modules = pre_init_module
                .call(modules)
                .await
                .map_err(InitError::Init)?;
        }

        let registry = {
            let global = Registry::raw_global();
            if global
                .set(Registry {
                    modules: modules.leak(),
                })
                .is_err()
            {
                panic!("The module registry has already been initialized once");
            }
            global
                .get()
                .unwrap_or_else(|| unreachable!("The OnceLock has just been set"))
        };

        process_join_results(
            registry
                .modules
                .iter()
                .map(|init_module| init_module.post_init())
                .collect::<Vec<_>>()
                .join()
                .await,
        )
        .map_err(InitError::PostInit)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum InitError {
    PreInit(Vec<module::PreInitError>),
    Init(module::InitError),
    PostInit(Vec<module::PostInitError>),
}

/// An uninitialised module waiting to be pre-initialised
type UninitModule = Box<dyn Fn() -> JoinHandle<Result<PreInitModule, module::PreInitError>>>;

/// A pre-initialised modules waiting to be initialised
type PreInitModule =
    BoxDynFnOnce<OwnedModulesSet, future::Boxed<Result<OwnedModulesSet, module::InitError>>>;

impl<M: Module> DynModule for M {
    fn post_init(&'static self) -> JoinHandle<Result<(), module::PostInitError>> {
        tokio::spawn(Module::post_init(self))
    }
}

/// Helper mimicking a `Box<dyn FnOnce>` which doesn't exist because `FnOnce` isn't object safe.
struct BoxDynFnOnce<Arg, Ret>(Box<dyn FnMut(Arg) -> Ret + Send>);
impl<Arg: 'static, Ret: 'static> BoxDynFnOnce<Arg, Ret> {
    /// Constructs a new `BoxDynFnOnce`
    pub fn new(f: impl FnOnce(Arg) -> Ret + Send + 'static) -> Self {
        let mut f = Some(f);
        Self(Box::new(move |arg| {
            let f = f
                .take()
                .unwrap_or_else(|| unreachable!("The BoxDynFnOnce can only be called once"));
            f(arg)
        }))
    }

    /// Calls the contained `FnOnce`
    pub fn call(mut self, arg: Arg) -> Ret {
        (self.0)(arg)
    }
}

fn process_join_results<T, E: From<String>>(
    vec: Vec<Result<Result<T, E>, JoinError>>,
) -> Result<Vec<T>, Vec<E>> {
    let mut ts = Vec::new();
    let mut errors = Vec::new();
    for join_result in vec {
        let result = join_result.unwrap_or_else(|join_error| {
            Err(E::from(
                join_error
                    .try_into_panic()
                    .map(|panic| {
                        format!(
                            "Module panicked: {}",
                            if let Some(string) = panic.downcast_ref::<String>() {
                                string.as_str()
                            } else if let Some(string) = panic.downcast_ref::<&'static str>() {
                                string
                            } else {
                                "Box<dyn Any>"
                            }
                        )
                    })
                    .unwrap_or_else(|join_error| format!("Couldn't join: {join_error}")),
            ))
        });

        match result {
            Ok(t) => ts.push(t),
            Err(error) => errors.push(error),
        }
    }

    if errors.is_empty() {
        Ok(ts)
    } else {
        Err(errors)
    }
}
