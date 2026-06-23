# wasmtime-testing-helper

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
You can pass anything you want into the `wasmtime::component::bindgen!` macro, this is just an
example.

In your tests you can arrange by calling `let mut harness = harness();` and then using the
`mock` and `stub` functions.

To mock a WIT implementation with logic, intended for if you change the output based on the
input parameter values given. You can do like so:
```rust
let mut harness = harness();
harness.mock(
    "namespace:package/interface",
    "function",
    |_context, (size,): (u32,)| Ok(("A".repeat(size as usize),)),
);
```

To stub a WIT implementation with set logic, intended for if you always give the same output
no matter the input parameter values given. You can do like so:
```rust
let mut harness = harness();
harness.stub::<(u32,), (String,)>(
    "namespace:package/interface",
    "function",
    ("AAAAAAAA".to_string(),),
);
```
This requires a turbofish to know the function parameter types. The first tuple is the
function parameter types, and the second tuple is the return type.

After arranging your mocks and stubs you can then act by calling `instantiate` on your
component testing environment like so `let mut component = instantiate(harness);`.
Then to invoke your component you can do:
```rust
let interface = component.component.namespace_interface_function();
    let result = interface
    .call_function(&mut component.store)
    .expect("failed to call function");
```
Where this `namespace_interface_function` function to fetch the interface for calling your
function is determined by the WIT namespace, interface and then function name.
And the `call_function` is just `call_` before your function name.

You can also get the amount of times mocked or stubbed function is called by using
`component.call_count("namespace_interface_function", "function")`.

## Not implemented yet
Easy composition for integration testing two WASM components talking to one another is not yet
implemented.
