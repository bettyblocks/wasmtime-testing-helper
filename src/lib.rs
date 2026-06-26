//! Helper library for integration testing WASM components without making separate crates for helper
//! WASM components.
//!
//! # Installation
//! Add this dev-dependency to your `Cargo.toml` like so:
//! ```TOML
//! [dev-dependencies]
//! wasmtime-testing-helper = { git = "https://github.com/bettyblocks/wasmtime-testing-helper" }
//! ```
//! The `wasmtime-testing-helper` exposes `wasmtime` through `wasmtime_testing_helper::wasmtime`,
//! but only with the features `["component-model", "cranelift", "runtime", "std"]`. If you want to
//! use more features, add wasmtime as a dev-dependency to your own crate and enable them and use
//! `wasmtime` instead of `wastime_testing_helper::wasmtime`.
//!
//! # Usage
//! Use the [`bindgen!`] macro to build the WIT interfaces for your WASM
//! component and then use the [`setup!`] macro to build the [`harness`](setup!) and
//! [`instantiate`](setup!) functions which build a testing harness for your specific WASM component using
//! the macro expansion of [wasmtime::component::bindgen]. Our
//! [`:bindgen!`] just wraps this so the macro expansion uses the re-export of wasmtime we provide,
//! so that you don't have to add wasmtime as a dependency yourself.
//! ```ignore
//! mod bindings {
//!     wasmtime_testing_helper::bindgen!("main");
//!
//!     wasmtime_testing_helper::setup!(Main);
//! }
//! ```
//! You can pass anything you want into the `wasmtime_testing_helper::bindgen!` macro, this is just an
//! example. If you pass a string like here it will look for a world in your WIT with the given
//! name. So for us it will look in `wit/world.wit` for `world main { ... }`. And then wasmtime
//! will give us an struct named after the world in PascalCase, so `Main`.
//!
//! In your tests you can arrange by calling `let mut harness = bindings::harness();` and then
//! using the [`ComponentCompositionBuilder::mock`], [`ComponentCompositionBuilder::stub`],
//! [`ComponentCompositionBuilder::wasi_context_builder_mut`] functions. You can also mock and stub
//! for resources with [`ComponentCompositionBuilder::mock_constructor`],
//! [`ComponentCompositionBuilder::stub_constructor`], [`ComponentCompositionBuilder::mock_method`],
//! and [`ComponentCompositionBuilder::stub_method`].
//!
//! To mock a WIT implementation with logic, intended for if you change the output based on the
//! input parameter values given. You can do like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
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
//! You can also mock a WIT resource constructor definition like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 resource %resource {
//! #                     constructor(initial: u32);
//! #                     increment: func(amount: u32) -> u32;
//! #                 }
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
//! struct Counter { count: u32 }
//! let mut harness = bindings::harness();
//! harness.mock_constructor(
//!     "namespace:package/interface",
//!     "resource",
//!     |(initial,): (u32,)| Ok(Counter { count: initial + 1 }),
//! );
//! ```
//!
//! Or a WIT resource method definition like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 resource %resource {
//! #                     constructor(initial: u32);
//! #                     increment: func(amount: u32) -> u32;
//! #                 }
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
//! struct Counter { count: u32 }
//! let mut harness = bindings::harness();
//! harness.mock_method(
//!     "namespace:package/interface",
//!     "resource",
//!     "increment",
//!     |backing: &mut Counter, (amount,): (u32,)| {
//!         backing.count += amount + 1;
//!         Ok((backing.count,))
//!     },
//! );
//! ```
//!
//! To stub a WIT implementation with a set value, intended for if you always give the same output
//! no matter the input parameter values given. You can do like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
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
//! You can also stub a WIT resource constructor definition like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 resource %resource {
//! #                     constructor(initial: u32);
//! #                     increment: func(amount: u32) -> u32;
//! #                 }
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
//! #[derive(Clone)]
//! struct Counter { count: u32 }
//! let mut harness = bindings::harness();
//! harness.stub_constructor::<Counter, (u32,)>(
//!     "namespace:package/interface",
//!     "resource",
//!     Counter { count: 0 },
//! );
//! ```
//! Note that your struct needs to implement [`Clone`]. You could add
//! `#[cfg_attr(test, derive(Clone))]` above your struct definition if it is in your source code.
//! Or you could try implementing [`Clone`] in your integration test module like so:
//! ```no_run
//! # mod other_module {
//! #   pub struct Counter { pub count: u32 }
//! # }
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 resource %resource {
//! #                     constructor(initial: u32);
//! #                     increment: func(amount: u32) -> u32;
//! #                 }
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
//! use other_module::Counter;
//! impl Clone for Counter {
//!     fn clone(&self) -> Self {
//!         Counter { count: self.count }
//!     }
//! }
//!
//! let mut harness = bindings::harness();
//! harness.stub_constructor::<Counter, (u32,)>(
//!     "namespace:package/interface",
//!     "resource",
//!     Counter { count: 0 },
//! );
//! ```
//!
//! Or a WIT resource method definition like so:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
//! #         inline: r"
//! #             package namespace:%package;
//! #
//! #             interface %interface {
//! #                 resource %resource {
//! #                     constructor(initial: u32);
//! #                     increment: func(amount: u32) -> u32;
//! #                 }
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
//! struct Counter { count: u32 }
//! let mut harness = bindings::harness();
//! harness.stub_method::<Counter, (u32,), (u32,)>(
//!     "namespace:package/interface",
//!     "resource",
//!     "increment",
//!     (22,),
//! );
//! ```
//!
//! After arranging your mocks and stubs you can then act by calling [`instantiate`](setup!) on your
//! component testing environment like so `let mut component = bindings::instantiate(harness);`.
//! Then to invoke your component you can do:
//! ```no_run
//! # mod bindings {
//! #     wasmtime_testing_helper::bindgen!({
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
//! It is possible to mock and stub outgoing http requests as well as long as the [http] feature is enabled.
//!
//! # Example
//! For the examples we use the inline option, but this would normally go in `wit/world.wit`
//! instead.
//!
//! Here's an example showing how you would test `function` on `%interface`. Which might call the
//! functions from `other-interface`.
//! ```no_run
//! mod bindings {
//!     wasmtime_testing_helper::bindgen!({
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
//! ).stub::<(String,), (String,)>(
//!     "namespace:package/other-interface",
//!     "another-function",
//!     (String::from("stubbed"),),
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
//! Here's an example with two resources where we mock one constructor and stub the other, and
//! mock one method and stub another method:
//! ```no_run
//! mod bindings {
//!     wasmtime_testing_helper::bindgen!({
//!         inline: r"
//!             package namespace:%package;
//!
//!             interface %interface {
//!                 resource counter {
//!                     constructor(initial: u32);
//!                     get-value: func() -> u32;
//!                 }
//!
//!                 resource multiplier {
//!                     constructor(factor: u32);
//!                     apply: func(value: u32) -> u32;
//!                 }
//!             }
//!
//!             world main {
//!                 export %interface;
//!             }
//!         "
//!     });
//!
//!     wasmtime_testing_helper::setup!(Main);
//! }
//! struct Counter { value: u32 }
//! #[derive(Clone)]
//! struct Multiplier { factor: u32 }
//! let mut harness = bindings::harness();
//! harness
//!     .mock_constructor(
//!         "namespace:package/interface",
//!         "counter",
//!         |(initial,): (u32,)| Ok(Counter { value: initial }),
//!     )
//!     .stub_constructor::<Multiplier, (u32,)>(
//!         "namespace:package/interface",
//!         "multiplier",
//!         Multiplier { factor: 2 },
//!     )
//!     .mock_method(
//!         "namespace:package/interface",
//!         "counter",
//!         "get-value",
//!         |backing: &mut Counter, (): ()| Ok((backing.value,)),
//!     )
//!     .stub_method::<Multiplier, (u32,), (u32,)>(
//!         "namespace:package/interface",
//!         "multiplier",
//!         "apply",
//!         (42,),
//!     );
//! ```
//!
//! # Not implemented yet
//! Easy composition for integration testing two WASM components talking to one another is not yet
//! implemented.

pub extern crate wasmtime;

/// Wraps [`wasmtime::component::bindgen!`] and automatically sets `wasmtime_crate` to the
/// re-exported [`wasmtime`] so consuming crates do not need `wasmtime` as a direct dependency.
///
/// Accepts the same arguments as [`wasmtime::component::bindgen!`]: either a world name string
/// or a configuration block.
/// ```ignore
/// mod bindings {
///     wasmtime_testing_helper::bindgen!("main");
///
///     wasmtime_testing_helper::setup!(Main);
/// }
/// ```
///
/// You could also do this manually like so.
/// ```ignore
/// mod bindings {
///     wasmtime_testing_helper::wasmtime::component::bindgen!({
///         world: "main",
///         wasmtime_crate: wasmtime_testing_helper::wasmtime,
///     });
///
///     wasmtime_testing_helper::setup!(Main);
/// }
/// ```
#[macro_export]
macro_rules! bindgen {
    ($name:literal) => {
        $crate::wasmtime::component::bindgen!({
            world: $name,
            wasmtime_crate: wasmtime_testing_helper::wasmtime,
        });
    };
    ({ $($config:tt)* }) => {
        $crate::wasmtime::component::bindgen!({
            wasmtime_crate: wasmtime_testing_helper::wasmtime,
            $($config)*
        });
    };
}

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use wasmtime::component::{
    Component, ComponentNamedList, Instance, Lift, Linker, LinkerInstance, Lower, Resource,
    ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Result, Store, StoreContextMut, Strategy};

use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

#[cfg(feature = "http")]
pub mod http;

static COMPONENT_CACHE: OnceLock<(Engine, Component)> = OnceLock::new();

/// Splits a method's full parameter tuple (which includes the resource handle as the first
/// element) into the handle and the remaining parameters. This allows `mock_method` closures to
/// receive `&mut BackingType` directly instead of working with raw resource handles.
pub trait MethodParameters<BackingType: 'static>: Sized + 'static {
    type WithSelf: ComponentNamedList + Lift + 'static;
    fn split_from(with_self: Self::WithSelf) -> (Resource<BackingType>, Self);
}

macro_rules! impl_method_parameters {
    ($($type:ident),*) => {
        #[allow(non_snake_case)]
        impl<BackingType: 'static, $($type: 'static),*> MethodParameters<BackingType>
            for ($($type,)*)
        where
            (Resource<BackingType>, $($type,)*): ComponentNamedList + Lift,
        {
            type WithSelf = (Resource<BackingType>, $($type,)*);
            fn split_from(
                (handle, $($type,)*): (Resource<BackingType>, $($type,)*)
            ) -> (Resource<BackingType>, ($($type,)*)) {
                (handle, ($($type,)*))
            }
        }
    };
}

// Blame wasmtime.
// https://docs.rs/crate/wasmtime/46.0.0/source/src/runtime/func.rs#294
// Just copying their implementation as it seems to be the easiests and the least boilerplate. This
// is just and issue with Rust.
impl_method_parameters!();
impl_method_parameters!(A1);
impl_method_parameters!(A1, A2);
impl_method_parameters!(A1, A2, A3);
impl_method_parameters!(A1, A2, A3, A4);
impl_method_parameters!(A1, A2, A3, A4, A5);
impl_method_parameters!(A1, A2, A3, A4, A5, A6);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
impl_method_parameters!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
impl_method_parameters!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15
);
impl_method_parameters!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16
);
impl_method_parameters!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17
);

/// Holds the state for the component(s) we are testing.
pub struct ComponentState {
    wasi_context: WasiCtx,
    resource_table: ResourceTable,
    #[cfg(feature = "http")]
    wasi_http_context: wasmtime_wasi_http::WasiHttpCtx,
    #[cfg(feature = "http")]
    hooks: Box<dyn wasmtime_wasi_http::p2::WasiHttpHooks>,
}

impl WasiView for ComponentState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_context,
            table: &mut self.resource_table,
        }
    }
}

#[cfg(feature = "http")]
impl wasmtime_wasi_http::p2::WasiHttpView for ComponentState {
    fn http(&mut self) -> wasmtime_wasi_http::p2::WasiHttpCtxView<'_> {
        wasmtime_wasi_http::p2::WasiHttpCtxView {
            ctx: &mut self.wasi_http_context,
            table: &mut self.resource_table,
            hooks: self.hooks.as_mut(),
        }
    }
}

/// We need this [`ResourceFunctionDefinition`] to be able to define the `Parameters` and `Return`
/// typings with the resource mocks and stubs. This is both for method and constructors.
/// This simply wraps the given closure for the mock or stub with our utility like call_count and
/// contains logic for adding it to the given LinkerInstance.
type ResourceFunctionDefinition =
    Box<dyn for<'a> FnOnce(&mut LinkerInstance<'a, ComponentState>) -> Result<()> + Send>;

/// Holds the component and the linking that has been mocked and stubbed for it.
pub struct ComponentCompositionBuilder {
    engine: Engine,
    component: Component,
    linker: Linker<ComponentState>,
    // The call counters are set up as the mocks and stubs are made, so they have to already exist
    // here.
    call_counters: HashMap<String, Arc<AtomicUsize>>,
    wasi_context_builder: WasiCtxBuilder,
    // We need to batch resource definitions per interface because `instance.resource` call locks
    // the interface name in the linker. Making it so that we can't define any more resources for
    // that interface.
    // The outer hashmap is the interface name and the inner hashmap is the resource name.
    pending_resource_definitions:
        HashMap<String, HashMap<String, (ResourceType, Vec<ResourceFunctionDefinition>)>>,
    #[cfg(feature = "http")]
    mock_hooks: http::HttpHooks,
}

impl ComponentCompositionBuilder {
    /// Creates a new [`ComponentCompositionBuilder`] object to test a component with. It is intended
    /// you use the [`harness`](setup!) function from the [`setup!`] macro to build one instead.
    ///
    /// The engine and compiled component are cached for the lifetime of the test binary so
    /// compilation only happens once regardless of how many tests call this.
    pub fn new(wasm_path: &str) -> Self {
        let (engine, component) = COMPONENT_CACHE
            .get_or_init(|| {
                let mut config = Config::new();
                config.strategy(Strategy::Winch);

                let engine =
                    Engine::new(&config).expect("failed to create engine with Winch strategy");
                let component = Component::from_file(&engine, wasm_path)
                    .expect("failed to load WASM component");
                (engine, component)
            })
            .clone();

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("failed to add WASI to linker");
        #[cfg(feature = "http")]
        wasmtime_wasi_http::p2::add_only_http_to_linker_sync(&mut linker)
            .expect("failed to add WASI HTTP to linker");

        ComponentCompositionBuilder {
            engine,
            component,
            linker,
            call_counters: HashMap::new(),
            wasi_context_builder: WasiCtxBuilder::new(),
            pending_resource_definitions: HashMap::new(),
            #[cfg(feature = "http")]
            mock_hooks: http::HttpHooks::new(),
        }
    }

    /// Mock a WIT implementation with logic. Intended for if you change the output based on the
    /// input parameter values given.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::bindgen!({
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
        let counter = self.register_call_counter(interface, function);

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
    /// #     wasmtime_testing_helper::bindgen!({
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
    /// #     wasmtime_testing_helper::bindgen!({
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

    /// Mocks the http handler in the wasmtime testing helpers.
    /// All outgoing http requests will be handled by this mock.
    /// This can be used to dynamically send responses to outgoing http requests.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::wasmtime::component::bindgen!({
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
    /// harness.mock_http_handler(
    ///     Box::new(|request, config| {
    ///        Box::pin(async move {
    ///             Ok(hyper::Response::new(request.into_body().await.unwrap()))
    ///         })
    ///    })
    /// );
    /// ```
    #[cfg(feature = "http")]
    pub fn mock_http_handler(&mut self, request_handler: http::HttpHandler) {
        self.mock_hooks.set_request_handler(request_handler)
    }

    /// Stubs the http handler in the wasmtime testing helpers.
    /// All outgoing http requests will be handled by this stub.
    /// This can be used to send a static response to outgoing http requests.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::wasmtime::component::bindgen!({
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
    /// harness.stub_http_handler(
    ///     Ok(hyper::Response::new(String::from("All good!")))
    /// );
    /// ```
    #[cfg(feature = "http")]
    pub fn stub_http_handler<T: Into<hyper::body::Bytes> + Clone + Send + Sync + 'static>(
        &mut self,
        response: Result<hyper::Response<T>, http::ErrorCode>,
    ) {
        let bytes_response = response.map(|response| {
            let (parts, bytes) = response.into_parts();
            hyper::Response::from_parts(parts, bytes.into())
        });
        self.mock_hooks.set_request_handler(Box::new(move |_, _| {
            let bytes_response_clone = bytes_response.clone();
            Box::pin(async move { bytes_response_clone })
        }))
    }

    /// Pushes a resource function definition into the `pending_resource_definitions` for the given
    /// interface and resource, declaring the resource type if it has not been declared yet.
    fn push_resource_definition<BackingType: Send + Sync + 'static>(
        &mut self,
        interface: &str,
        resource_name: &str,
        definition: ResourceFunctionDefinition,
    ) {
        self.pending_resource_definitions
            .entry(String::from(interface))
            .or_default()
            .entry(String::from(resource_name))
            .or_insert_with(|| (ResourceType::host::<BackingType>(), Vec::new()))
            .1
            .push(definition);
    }

    /// Registers a new call counter for the mocked or stubbed function.
    fn register_call_counter(&mut self, interface: &str, function: &str) -> Arc<AtomicUsize> {
        let counter = Arc::new(AtomicUsize::new(0));
        self.call_counters
            .insert(format!("{}.{}", interface, function), counter.clone());
        counter
    }

    /// Mock a WIT resource constructor implementation with logic. Intended for if you change the
    /// built struct based on the input parameter values given.
    /// You are free to use any struct for mocking, as the WIT does not define fields the struct
    /// that the resource must adhere to, only your existing logic.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 resource %resource {
    /// #                     constructor(initial: u32);
    /// #                     increment: func(amount: u32) -> u32;
    /// #                 }
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
    /// struct Counter { count: u32 }
    /// let mut harness = bindings::harness();
    /// harness.mock_constructor(
    ///     "namespace:package/interface",
    ///     "resource",
    ///     |(initial,): (u32,)| Ok(Counter { count: initial + 1 }),
    /// );
    /// ```
    pub fn mock_constructor<BackingType, Parameters>(
        &mut self,
        interface: &str,
        resource_name: &str,
        handler: impl Fn(Parameters) -> Result<BackingType> + Send + Sync + 'static,
    ) -> &mut Self
    where
        BackingType: Send + Sync + 'static,
        Parameters: ComponentNamedList + Lift + 'static,
    {
        let constructor_name = format!("[constructor]{}", resource_name);
        let counter = self.register_call_counter(interface, &constructor_name);

        let resource_name = String::from(resource_name);
        let definition: ResourceFunctionDefinition = Box::new(move |instance| {
            instance
                .func_wrap(
                    &constructor_name,
                    move |mut context: StoreContextMut<'_, ComponentState>,
                          parameters: Parameters| {
                        counter.fetch_add(1, Ordering::Relaxed);
                        let backing_value = handler(parameters)?;
                        let resource = context.data_mut().resource_table.push(backing_value)?;
                        Ok((resource,))
                    },
                )
                .expect("failed to register constructor mock");
            Ok(())
        });
        self.push_resource_definition::<BackingType>(interface, &resource_name, definition);
        self
    }

    /// Stub a WIT resource constructor with a set value. Intended for if you always give the same
    /// struct no matter the input parameter values given.
    /// This requires a turbofish to know the backing type and function parameter types. The first
    /// is the backing struct type and the second tuple is the constructor parameter types.
    /// You are free to use any struct for stubbing, as the WIT does not define fields the struct
    /// that the resource must adhere to, only your existing logic.
    /// The struct used for stubbing the resource needs to derive [`Clone`].
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 resource %resource {
    /// #                     constructor(initial: u32);
    /// #                     increment: func(amount: u32) -> u32;
    /// #                 }
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
    /// #[derive(Clone)]
    /// struct Counter { count: u32 }
    /// let mut harness = bindings::harness();
    /// harness.stub_constructor::<Counter, (u32,)>(
    ///     "namespace:package/interface",
    ///     "resource",
    ///     Counter { count: 0 },
    /// );
    /// ```
    pub fn stub_constructor<BackingType, Parameters>(
        &mut self,
        interface: &str,
        resource_name: &str,
        value: BackingType,
    ) -> &mut Self
    where
        BackingType: Clone + Send + Sync + 'static,
        Parameters: ComponentNamedList + Lift + 'static,
    {
        self.mock_constructor::<BackingType, Parameters>(
            interface,
            resource_name,
            move |_parameters: Parameters| Ok(value.clone()),
        )
    }

    /// Mock a WIT resource method implementation with logic. Intended for if you change the
    /// return value based on the input parameter values given or the resource state.
    /// You are free to use any struct for mocking, as the WIT does not define fields the struct
    /// that the resource must adhere to, only your existing logic.
    /// The handler receives a mutable reference to the backing Rust value and the method
    /// parameters.
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 resource %resource {
    /// #                     constructor(initial: u32);
    /// #                     increment: func(amount: u32) -> u32;
    /// #                 }
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
    /// struct Counter { count: u32 }
    /// let mut harness = bindings::harness();
    /// harness.mock_method(
    ///     "namespace:package/interface",
    ///     "resource",
    ///     "increment",
    ///     |backing: &mut Counter, (amount,): (u32,)| {
    ///         backing.count += amount + 1;
    ///         Ok((backing.count,))
    ///     },
    /// );
    /// ```
    pub fn mock_method<BackingType, MethodParams, Return>(
        &mut self,
        interface: &str,
        resource_name: &str,
        method_name: &str,
        handler: impl Fn(&mut BackingType, MethodParams) -> Result<Return> + Send + Sync + 'static,
    ) -> &mut Self
    where
        BackingType: Send + Sync + 'static,
        MethodParams: MethodParameters<BackingType>,
        MethodParams::WithSelf: ComponentNamedList + Lift + 'static,
        Return: ComponentNamedList + Lower + 'static,
    {
        let method_function_name = format!("[method]{}.{}", resource_name, method_name);
        let counter = self.register_call_counter(interface, &method_function_name);

        let resource_name = String::from(resource_name);
        let definition: ResourceFunctionDefinition = Box::new(move |instance| {
            instance
                .func_wrap(
                    &method_function_name,
                    move |mut context: StoreContextMut<'_, ComponentState>,
                          parameters: MethodParams::WithSelf| {
                        counter.fetch_add(1, Ordering::Relaxed);

                        let (resource_handle, method_parameters) =
                            MethodParams::split_from(parameters);
                        let backing_value = context
                            .data_mut()
                            .resource_table
                            .get_mut::<BackingType>(&resource_handle)?;

                        handler(backing_value, method_parameters)
                    },
                )
                .expect("failed to register method mock");
            Ok(())
        });
        self.push_resource_definition::<BackingType>(interface, &resource_name, definition);
        self
    }

    /// Stub a WIT resource method with a set value. Intended for if you always return the same
    /// value no matter the resource state or method parameters.
    /// This requires a turbofish to know the backing type and function parameter types. The first
    /// is the backing struct type, the second tuple is the method parameter types, and the third
    /// tuple is the return type.
    /// The return value needs to derive [`Clone`].
    /// ```no_run
    /// # mod bindings {
    /// #     wasmtime_testing_helper::bindgen!({
    /// #         inline: r"
    /// #             package namespace:%package;
    /// #
    /// #             interface %interface {
    /// #                 resource %resource {
    /// #                     constructor(initial: u32);
    /// #                     increment: func(amount: u32) -> u32;
    /// #                 }
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
    /// struct Counter { count: u32 }
    /// let mut harness = bindings::harness();
    /// harness.stub_method::<Counter, (u32,), (u32,)>(
    ///     "namespace:package/interface",
    ///     "resource",
    ///     "increment",
    ///     (22,),
    /// );
    /// ```
    pub fn stub_method<BackingType, MethodParams, Return>(
        &mut self,
        interface: &str,
        resource_name: &str,
        method_name: &str,
        value: Return,
    ) -> &mut Self
    where
        BackingType: Send + Sync + 'static,
        MethodParams: MethodParameters<BackingType>,
        MethodParams::WithSelf: ComponentNamedList + Lift + 'static,
        Return: ComponentNamedList + Lower + Clone + Send + Sync + 'static,
    {
        self.mock_method::<BackingType, MethodParams, Return>(
            interface,
            resource_name,
            method_name,
            move |_backing: &mut BackingType, _method_parameters: MethodParams| Ok(value.clone()),
        )
    }

    /// Gives you a typed instantiated component to call functions on. It is intended you use
    /// [`instantiate`](setup!) from the [`setup!`] macro to build an [`InstantiatedComponent`] instead.
    pub fn instantiate<T>(
        self,
        wrap: impl FnOnce(&mut Store<ComponentState>, &Instance) -> T,
    ) -> InstantiatedComponent<T> {
        let ComponentCompositionBuilder {
            engine,
            component,
            mut linker,
            call_counters,
            mut wasi_context_builder,
            pending_resource_definitions,
            #[cfg(feature = "http")]
            mock_hooks,
        } = self;

        for (interface, resource_map) in pending_resource_definitions {
            let mut instance = linker
                .instance(&interface)
                .expect("failed to get linker instance");
            for (resource_name, (resource_type, definitions)) in resource_map {
                instance
                    .resource(&resource_name, resource_type, |_, _| Ok(()))
                    .expect("failed to register resource type");
                for definition in definitions {
                    definition(&mut instance).expect("failed to apply resource definition");
                }
            }
        }

        let state = ComponentState {
            wasi_context: wasi_context_builder.build(),
            resource_table: ResourceTable::new(),
            #[cfg(feature = "http")]
            wasi_http_context: wasmtime_wasi_http::WasiHttpCtx::new(),
            #[cfg(feature = "http")]
            hooks: Box::new(mock_hooks),
        };
        let mut store = Store::new(&engine, state);
        let instance = linker
            .instantiate(&mut store, &component)
            .expect("failed to instantiate component");
        let component = wrap(&mut store, &instance);

        InstantiatedComponent {
            store,
            component,
            call_counters,
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
    /// For resource constructors, pass `"[constructor]resource-name"` as the function.
    /// For resource methods, pass `"[method]resource-name.method-name"` as the function.
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
///     wasmtime_testing_helper::bindgen!("main");
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

            // CARGO_TARGET_TMPDIR is only set by Cargo for integration tests. This works with the
            // `no_run` in doctests and the allow lets it work with `cargo clippy`.
            #[allow(clippy::option_env_unwrap)]
            let cargo_target_tmpdir = option_env!("CARGO_TARGET_TMPDIR")
                .expect("CARGO_TARGET_TMPDIR not set. Will be set with `cargo test`");

            let target_directory = std::path::Path::new(cargo_target_tmpdir)
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
