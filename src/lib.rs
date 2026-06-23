//! Helper library for integration testing WASM components without making separate crates for helper
//! WASM components.
//!
//! # Usage
//! Use the `wasmtime::component::bindgen!` macro to build the WIT interfaces for your WASM
//! component and then use the `wasmtime_testing_helper::setup!` macro to build the `harness` and
//! `instantiate` functions which build a testing harness for your specific WASM component using
//! the macro expansion of `wasmtime::component::bindgen!`.
//! ```ignore
//! mod bindings {
//!     wasmtime::component::bindgen!({ path: "wit", world: "main" });
//! }
//!
//! wasmtime_testing_helper::setup!(bindings);
//! ```
//!
//! The in your tests you can arrange by calling `let mut harness = harness();` and then using
//! the `mock` and `stub` functions. And then act by calling instantiating your component testing
//! environment with `let mut component = instantiate(harness);` And invoking your component with
//! ```
//! let interface = component.component.namespace_interface_function();
//!     let result = interface
//!     .call_function(&mut component.store)
//!     .expect("failed to call function");
//! ```
//! Where this `namespace_interface_function` function to fetch the interface for calling your
//! function is determined by the WIT namespace, interface and then function name.
//! And the `call_function` is just `call_` before your function name.
//!
//! You can also get the amount of times mocked or stubbed function is called by using
//! `component.call_count("namespace_interface_function", "function")`.
//!
//! # Not implemented yet
//! Easy composition for integration testing two WASM components talking to one another is not yet
//! implemented.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use wasmtime::component::{
    Component, ComponentNamedList, Instance, Lift, Linker, Lower, ResourceTable,
};
use wasmtime::{Engine, Result, Store, StoreContextMut};
pub use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};
use wasmtime_wasi_http::{
    WasiHttpCtx,
    p2::{WasiHttpCtxView, WasiHttpView, default_hooks},
};

/// Holds the state for the component(s) we are testing.
pub struct ComponentState {
    wasi_context: WasiCtx,
    wasi_http_context: WasiHttpCtx,
    resource_table: ResourceTable,
}

impl WasiView for ComponentState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_context,
            table: &mut self.resource_table,
        }
    }
}

impl WasiHttpView for ComponentState {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.wasi_http_context,
            table: &mut self.resource_table,
            hooks: default_hooks(),
        }
    }
}

/// Holds the component and the linking that has been mocked and stubbed for it.
pub struct ComponentCompositionBuilder {
    engine: Engine,
    component: Component,
    linker: Linker<ComponentState>,
    call_counts: HashMap<String, Arc<AtomicUsize>>,
}

impl ComponentCompositionBuilder {
    /// Creates a new ComponentCompositionBuilder object to test a component with. It is intended
    /// you use the `harness` function from the `setup!` macro to build the ComponentCompositionBuilder instead.
    pub fn new(wasm_path: &str) -> Self {
        let engine = Engine::default();
        let component =
            Component::from_file(&engine, wasm_path).expect("failed to load WASM component");

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("failed to add WASI to linker");
        wasmtime_wasi_http::p2::add_only_http_to_linker_sync(&mut linker)
            .expect("failed to add WASI HTTP to linker");

        ComponentCompositionBuilder {
            engine,
            component,
            linker,
            call_counts: HashMap::new(),
        }
    }

    /// Mock a WIT implementation with logic. Intended for if you change the output based on the
    /// input parameter values given.
    /// ```ignore
    /// let mut harness = harness();
    /// harness.mock(
    ///     "namespace:package/interface"
    ///     "function"
    ///     |_context, (size,): (u32,)| Ok(("A".repeat(size as usize),)),
    /// );
    /// ```
    pub fn mock<Parameters, Return>(
        &mut self,
        interface: &str,
        function: &str,
        handler: impl Fn(StoreContextMut<'_, ComponentState>, Parameters) -> Result<Return>
        + Send
        + Sync
        + 'static,
    ) -> &mut Self
    where
        Parameters: ComponentNamedList + Lift + 'static,
        Return: ComponentNamedList + Lower + 'static,
    {
        let counter = Arc::new(AtomicUsize::new(0));
        self.call_counts
            .insert(format!("{}.{}", interface, function), counter.clone());

        self.linker
            .instance(interface)
            .expect("failed to get linker instance")
            .func_wrap(function, move |context, parameters: Parameters| {
                counter.fetch_add(1, Ordering::Relaxed);
                handler(context, parameters)
            })
            .expect("failed to register mock function");
        self
    }

    /// Mock a WIT implementation with logic. Intended for if you always give the same output no
    /// matter the input parameter values given.
    /// This requires a turbofish to know the function parameter types. The first tuple is the
    /// function parameter types, and the second tuple is the return type.
    /// ```ignore
    /// let mut harness = harness();
    /// harness.stub::<(u32,), (String,)>(
    ///     "namespace:package/interface"
    ///     "function"
    ///     ("AAAAAAAA".to_string(),),
    /// );
    /// ```
    pub fn stub<Parameters, Return>(
        &mut self,
        interface: &str,
        function: &str,
        value: Return,
    ) -> &mut Self
    where
        Parameters: ComponentNamedList + Lift + 'static,
        Return: ComponentNamedList + Lower + Clone + Send + Sync + 'static,
    {
        self.mock(
            interface,
            function,
            move |_context, _parameters: Parameters| Ok(value.clone()),
        )
    }

    /// Gives you a typed instantiated component to call functions on. It is intended you use the
    /// `instantiate` from the `setup!` macro to build the InstantiatedComponent instead.
    pub fn instantiate<T>(
        self,
        wasi_context: WasiCtx,
        wrap: impl FnOnce(&mut Store<ComponentState>, &Instance) -> T,
    ) -> InstantiatedComponent<T> {
        let state = ComponentState {
            wasi_context,
            wasi_http_context: WasiHttpCtx::new(),
            resource_table: ResourceTable::new(),
        };
        let mut store = Store::new(&self.engine, state);
        let instance = self
            .linker
            .instantiate(&mut store, &self.component)
            .expect("failed to instantiate component");
        let component = wrap(&mut store, &instance);

        InstantiatedComponent {
            store,
            component,
            call_counts: self.call_counts,
        }
    }
}

/// The instantiated component lives in the component field along with the store field storing the
/// state of the component.
pub struct InstantiatedComponent<T> {
    pub store: Store<ComponentState>,
    pub component: T,
    call_counts: HashMap<String, Arc<AtomicUsize>>,
}

impl<T> InstantiatedComponent<T> {
    /// Gets the amount of times a function for a specific interface has been called.
    pub fn call_count(&self, interface: &str, function: &str) -> usize {
        let key = format!("{}.{}", interface, function);
        self.call_counts
            .get(&key)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}

/// Intended to be used like so to set up project specific helpers which automatically route to the
/// WASM file artifact made by building with `cargo build --target=wasm32-wasip2 --release`. It is
/// expected that this build is ran before testing to ensure up-to-date state.
/// ```ignore
/// mod bindings {
///     wasmtime::component::bindgen!({ path: "wit", world: "main" });
/// }
///
/// wasmtime_testing_helper::setup!(bindings);
/// ```
#[macro_export]
macro_rules! setup {
    ($bindings:ident) => {
        fn harness() -> $crate::ComponentCompositionBuilder {
            let package_name = env!("CARGO_PKG_NAME").replace('-', "_");
            let wasm_path = format!("{}/{}.wasm", env!("CARGO_MANIFEST_DIR"), package_name,);
            $crate::ComponentCompositionBuilder::new(&wasm_path)
        }

        fn instantiate(
            component_composition_builder: $crate::ComponentCompositionBuilder,
        ) -> $crate::InstantiatedComponent<$bindings::Main> {
            component_composition_builder.instantiate(
                wasmtime_testing_helper::WasiCtxBuilder::new().build(),
                |store, instance| {
                    $bindings::Main::new(store, instance)
                        .expect("failed to create typed component wrapper")
                },
            )
        }
    };
}
