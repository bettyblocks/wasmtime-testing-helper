//! Helper library for integration testing WASM components without making separate crates for helper
//! WASM components.
//!
//! # Installation
//! Add these dev-dependencies to your `Cargo.toml` like so:
//! ```TOML
//! [dev-dependencies]
//! wasmtime = { version = "46", default-features = false, features = ["component-model", "cranelift", "runtime", "std"] }
//! wasmtime-testing-helper = { git = "https://github.com/bettyblocks/wasmtime-testing-helper" }
//! ```
//!
//! # Usage
//! Use the [wasmtime::component::bindgen!] macro to build the WIT interfaces for your WASM
//! component and then use the [`setup!`] macro to build the [`harness`](setup!) and
//! [`instantiate`](setup!) functions which build a testing harness for your specific WASM component using
//! the macro expansion of [wasmtime::component::bindgen!].
//! ```ignore
//! mod bindings {
//!     wasmtime::component::bindgen!("main");
//!
//!     wasmtime_testing_helper::setup!(Main);
//! }
//! ```
//! You can pass anything you want into the `wasmtime::component::bindgen!` macro, this is just an
//! example. If you pass a string like here it will look for a world in your WIT with the given
//! name. So for us it will look in `wit/world.wit` for `world main { ... }`. And then wasmtime
//! will give us an struct named after the world in PascalCase, so `Main`.
//!
//! In your tests you can arrange by calling `let mut harness = bindings::harness();` and then
//! using the [`ComponentCompositionBuilder::mock`], [`ComponentCompositionBuilder::stub`] and
//! [`ComponentCompositionBuilder::wasi_context_builder_mut`] functions.
//!
//! To mock a WIT implementation with logic, intended for if you change the output based on the
//! input parameter values given. You can do like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime::component::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 function: func(length: u32) -> string;
//! #             }
//! #
//! #             world main {
//! #                 export %interface;
//! #             }
//! #         "
//! #     });
//! #
//! #     wasmtime_testing_helper::setup!(Main);
//! # }
//! let mut harness = bindings::harness();
//! harness.mock(
//!     "namespace:package/interface",
//!     "function",
//!     |_context, (size,): (u32,)| Ok(("A".repeat(size as usize),)),
//! );
//! ```
//!
//! To stub a WIT implementation with a set value, intended for if you always give the same output
//! no matter the input parameter values given. You can do like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime::component::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 function: func(length: u32) -> string;
//! #             }
//! #
//! #             world main {
//! #                 export %interface;
//! #             }
//! #         "
//! #     });
//! #
//! #     wasmtime_testing_helper::setup!(Main);
//! # }
//! let mut harness = bindings::harness();
//! harness.stub::<(u32,), (String,)>(
//!     "namespace:package/interface",
//!     "function",
//!     (String::from("AAAAAAAA"),),
//! );
//! ```
//! This requires a turbofish to know the function parameter types. The first tuple is the
//! function parameter types, and the second tuple is the return type.
//!
//! After arranging your mocks and stubs you can then act by calling [`instantiate`](setup!) on your
//! component testing environment like so `let mut component = bindings::instantiate(harness);`.
//! Then to invoke your component you can do:
//! ```no_run
//! # mod bindings {
//! #     wasmtime::component::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 function: func(length: u32) -> string;
//! #             }
//! #
//! #             world main {
//! #                 export %interface;
//! #             }
//! #         "
//! #     });
//! #
//! #     wasmtime_testing_helper::setup!(Main);
//! # }
//! let mut harness = bindings::harness();
//! let mut component = bindings::instantiate(harness);
//! let interface = component.component.namespace_package_interface();
//! let result = interface
//!     .call_function(&mut component.store, 0)
//!     .expect("failed to call function");
//! ```
//! Where this `namespace_package_interface` function to fetch the interface for calling your
//! function is determined by the WIT namespace, package, and interface name.
//! And the `call_function` is just `call_` before your function name.
//!
//! You can also get the amount of times a mocked or stubbed function is called by using
//! [`InstantiatedComponent::call_count`].
//!
//! # Example
//! For the example we use the inline option, but this would normally go in `wit/world.wit`
//! instead.
//! ```no_run
//! mod bindings {
//!     wasmtime::component::bindgen!({
//!         inline: r"
//!             package namespace:%package;
//!
//!             interface %interface {
//!                 function: func(length: u32) -> string;
//!             }
//!
//!             interface other-interface {
//!                 other-function: func(value: string) -> string;
//!                 another-function: func(value: string) -> string;
//!             }
//!
//!             world main {
//!                 import other-interface;
//!                 export %interface;
//!             }
//!         "
//!     });
//!
//!     wasmtime_testing_helper::setup!(Main);
//! }
//! let mut harness = bindings::harness();
//!
//! harness.mock(
//!     "namespace:package/other-interface",
//!     "other-function",
//!     |_context, (value,): (String,)| Ok((value.to_uppercase(),)),
//! );
//! harness.stub::<(String,), (String,)>(
//!     "namespace:package/other-interface",
//!     "another-function",
//!     (String::from("stubbed",),
//! );
//!
//! let mut component = bindings::instantiate(harness);
//! let interface = component.component.namespace_package_interface();
//! let result = interface
//!     .call_function(&mut component.store, 8)
//!     .expect("failed to call function");
//! assert_eq!(result, "STUBBED");
//! assert_eq!(
//!     component.call_count("namespace:package/other-interface", "other-function"),
//!     1,
//! );
//! assert_eq!(
//!     component.call_count("namespace:package/other-interface", "another-function"),
//!     1,
//! );
//! ```
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
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
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
    // The call counters are set up as the mocks and stubs are made, so they have to already exist
    // here.
    call_counters: HashMap<String, Arc<AtomicUsize>>,
    wasi_context_builder: WasiCtxBuilder,
}

impl ComponentCompositionBuilder {
    /// Creates a new [`ComponentCompositionBuilder`] object to test a component with. It is intended
    /// you use the [`harness`](setup!) function from the [`setup!`] macro to build one instead.
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
            call_counters: HashMap::new(),
            wasi_context_builder: WasiCtxBuilder::new(),
        }
    }

    /// Mock a WIT implementation with logic. Intended for if you change the output based on the
    /// input parameter values given.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime::component::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 function: func(length: u32) -> string;
    /// #             }
    /// #
    /// #             world main {
    /// #                 export %interface;
    /// #             }
    /// #         "
    /// #     });
    /// #
    /// #     wasmtime_testing_helper::setup!(Main);
    /// # }
    /// let mut harness = bindings::harness();
    /// harness.mock(
    ///     "namespace:package/interface",
    ///     "function",
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
        self.call_counters
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

    /// Stub a WIT implementation with a set value. Intended for if you always give the same output
    /// no matter the input parameter values given.
    /// This requires a turbofish to know the function parameter types. The first tuple is the
    /// function parameter types, and the second tuple is the return type.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime::component::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 function: func(length: u32) -> string;
    /// #             }
    /// #
    /// #             world main {
    /// #                 export %interface;
    /// #             }
    /// #         "
    /// #     });
    /// #
    /// #     wasmtime_testing_helper::setup!(Main);
    /// # }
    /// let mut harness = bindings::harness();
    /// harness.stub::<(u32,), (String,)>(
    ///     "namespace:package/interface",
    ///     "function",
    ///     (String::from("AAAAAAAA"),),
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

    /// Returns a mutable reference to the wasi context builder.
    /// This can be used to for example set environment variables.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime::component::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 function: func(length: u32) -> string;
    /// #             }
    /// #
    /// #             world main {
    /// #                 export %interface;
    /// #             }
    /// #         "
    /// #     });
    /// #
    /// #     wasmtime_testing_helper::setup!(Main);
    /// # }
    /// let mut harness = bindings::harness();
    /// harness.wasi_context_builder_mut().env("ENVIRONMENT_VAR", "Exists");
    /// ```
    pub fn wasi_context_builder_mut(&mut self) -> &'_ mut WasiCtxBuilder {
        &mut self.wasi_context_builder
    }

    /// Gives you a typed instantiated component to call functions on. It is intended you use
    /// [`instantiate`](setup!) from the [`setup!`] macro to build an [`InstantiatedComponent`] instead.
    pub fn instantiate<T>(
        mut self,
        wrap: impl FnOnce(&mut Store<ComponentState>, &Instance) -> T,
    ) -> InstantiatedComponent<T> {
        let state = ComponentState {
            wasi_context: self.wasi_context_builder.build(),
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
            call_counters: self.call_counters,
        }
    }
}

/// The instantiated component lives in the component field along with the store field storing the
/// state of the component.
pub struct InstantiatedComponent<T> {
    pub store: Store<ComponentState>,
    pub component: T,
    call_counters: HashMap<String, Arc<AtomicUsize>>,
}

impl<T> InstantiatedComponent<T> {
    /// Gets the amount of times a function for a specific interface has been called.
    pub fn call_count(&self, interface: &str, function: &str) -> usize {
        let key = format!("{}.{}", interface, function);
        self.call_counters
            .get(&key)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}

/// Intended to be used like so to set up project specific helpers which automatically route to the
/// WASM file artifact made by building with `cargo build --target=wasm32-wasip2 --release`. It is
/// expected that this build is run before testing to ensure up-to-date state.
/// ```ignore
/// mod bindings {
///     wasmtime::component::bindgen!("main");
///
///     wasmtime_testing_helper::setup!(Main);
/// }
/// ```
#[macro_export]
macro_rules! setup {
    ($bindings:path) => {
        /// Builds an instance of ComponentCompositionBuilder that uses the .wasm file built using
        /// `cargo build --target=wasm32-wasip2 --release`.
        pub fn harness() -> $crate::ComponentCompositionBuilder {
            let package_name = env!("CARGO_PKG_NAME").replace('-', "_");

            // CARGO_TARGET_TMPDIR is set by Cargo at runtime during integration tests.
            // We use std::env::var instead of env!() so this compiles in the doctest context.
            let cargo_target_tmpdir = std::env::var("CARGO_TARGET_TMPDIR")
                .expect("CARGO_TARGET_TMPDIR not set; run tests via `cargo test`");
            let target_directory = std::path::Path::new(&cargo_target_tmpdir)
                .parent()
                .expect("CARGO_TARGET_TMPDIR has no parent directory")
                .to_path_buf();
            let wasm_path = format!(
                "{}/wasm32-wasip2/release/{}.wasm",
                target_directory.display(),
                package_name
            );
            $crate::ComponentCompositionBuilder::new(&wasm_path)
        }

        /// Instantiates your testing environment using the definitions of your built .wasm file
        /// and the mocks and stubs possibly added.
        pub fn instantiate(
            component_composition_builder: $crate::ComponentCompositionBuilder,
        ) -> $crate::InstantiatedComponent<$bindings> {
            component_composition_builder.instantiate(|store, instance| {
                <$bindings>::new(store, instance).expect("failed to create typed component wrapper")
            })
        }
    };
}
