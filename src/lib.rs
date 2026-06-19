use wasmtime::component::{
    Component, ComponentNamedList, Instance, Lift, Linker, Lower, ResourceTable,
};
use wasmtime::{Engine, Result, Store, StoreContextMut};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

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

pub struct ComponentCompositionBuilder {
    engine: Engine,
    component: Component,
    linker: Linker<ComponentState>,
}

impl ComponentCompositionBuilder {
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
