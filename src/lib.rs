use wasmtime::component::{
    Component, ComponentNamedList, Instance, Lift, Linker, Lower, ResourceTable,
};
use wasmtime::{Engine, Result, Store, StoreContextMut};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

/// Holds the state for the component(s) we are testing.
pub struct ComponentState {
    wasi_context: WasiCtx,
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

/// Holds the component and the linking that has been mocked and stubbed for it.
pub struct ComponentCompositionBuilder {
    engine: Engine,
    component: Component,
    linker: Linker<ComponentState>,
}

impl ComponentCompositionBuilder {
    /// Creates a new ComponentCompositionBuilder object to test a component with. It is intended
    /// you use the `harness` function to build the ComponentCompositionBuilder instead.
    pub fn new(wasm_path: &str) -> Self {
        let engine = Engine::default();
        let component =
            Component::from_file(&engine, wasm_path).expect("failed to load WASM component");

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("failed to add WASI to linker");

        ComponentCompositionBuilder {
            engine,
            component,
            linker,
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
        self.linker
            .instance(interface)
            .expect("failed to get linker instance")
            .func_wrap(function, handler)
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
        self.mock(interface, function, move |_context, _parameters: Parameters| {
            Ok(value.clone())
        })
    }

    /// Gives you a typed instantiated component to call functions on.
    pub fn instantiate<T>(
        self,
        wrap: impl FnOnce(&mut Store<ComponentState>, &Instance) -> T,
    ) -> InstantiatedComponent<T> {
        let state = ComponentState {
            wasi_context: WasiCtxBuilder::new().build(),
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
        }
    }
}

pub struct InstantiatedComponent<T> {
    pub store: Store<ComponentState>,
    pub component: T,
}

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
            component_composition_builder.instantiate(|store, instance| {
                $bindings::Main::new(store, instance)
                    .expect("failed to create typed component wrapper")
            })
        }
    };
}
