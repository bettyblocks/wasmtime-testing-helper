# wasmtime-testing-helper

## wasmtime testing helper

Helper library for integration testing WASM components without making separate crates for helper
WASM components.

## Usage
Use the `wasmtime::component::bindgen!` macro to build the WIT interfaces for your WASM
component and then use the `wasmtime_testing_helper::setup!` macro to build the `harness` and
`instantiate` functions which build a testing harness for your specific WASM component using
the macro expansion of `wasmtime::component::bindgen!`.
```rust
mod bindings {
    wasmtime::component::bindgen!({ path: "wit", world: "main" });
}

wasmtime_testing_helper::setup!(bindings);
```

The in your tests you can arrange by calling `let mut harness = harness();` and then using
the `mock` and `stub` functions. And then act by calling instantiating your component testing
environment with `let mut component = instantiate(harness);` And invoking your component with
```rust
let interface = component.component.namespace_interface_function();
    let result = interface
    .call_function(&mut component.store)
    .expect("failed to call function");
```
Where this `namespace_interface_function` function to fetch the interface for calling your
function is determined by the WIT namespace, interface and then function name.
And the `call_function` is just `call_` before your function name.

## Not implemented yet
Easy composition for integration testing two WASM components talking to one another is not yet
implemented.
