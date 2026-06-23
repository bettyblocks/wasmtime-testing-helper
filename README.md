# wasmtime-testing-helper

Helper library for integration testing WASM components without making separate crates for helper
WASM components.

## Installation
Add these dev-dependencies to your `Cargo.toml` like so:
```TOML
[dev-dependencies]
wasmtime = { version = "46", default-features = false, features = ["component-model", "cranelift", "runtime", "std"] }
wasmtime-testing-helper = { git = "https://github.com/bettyblocks/wasmtime-testing-helper" }
```

## Usage
Use the [wasmtime::component::bindgen!](https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html) macro to build the WIT interfaces for your WASM
component and then use the [`setup!`] macro to build the [`harness`](setup!) and
[`instantiate`](setup!) functions which build a testing harness for your specific WASM component using
the macro expansion of [wasmtime::component::bindgen!](https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html).
```rust
mod bindings {
    wasmtime::component::bindgen!("main");

    wasmtime_testing_helper::setup!(Main);
}
```
You can pass anything you want into the `wasmtime::component::bindgen!` macro, this is just an
example. If you pass a string like here it will look for a world in your WIT with the given
name. So for us it will look in `wit/world.wit` for `world main { ... }`. And then wasmtime
will give us an struct named after the world in PascalCase, so `Main`.

In your tests you can arrange by calling `let mut harness = bindings::harness();` and then
using the [`ComponentCompositionBuilder::mock`], [`ComponentCompositionBuilder::stub`] and
[`ComponentCompositionBuilder::wasi_context_builder_mut`] functions.

To mock a WIT implementation with logic, intended for if you change the output based on the
input parameter values given. You can do like so:
```rust
#
#
#
let mut harness = bindings::harness();
harness.mock(
    "namespace:package/interface",
    "function",
    |_context, (size,): (u32,)| Ok(("A".repeat(size as usize),)),
);
```

To stub a WIT implementation with set logic, intended for if you always give the same output
no matter the input parameter values given. You can do like so:
```rust
#
#
#
let mut harness = bindings::harness();
harness.stub::<(u32,), (String,)>(
    "namespace:package/interface",
    "function",
    ("AAAAAAAA".to_string(),),
);
```
This requires a turbofish to know the function parameter types. The first tuple is the
function parameter types, and the second tuple is the return type.

After arranging your mocks and stubs you can then act by calling [`instantiate`](setup!) on your
component testing environment like so `let mut component = bindings::instantiate(harness);`.
Then to invoke your component you can do:
```rust
#
#
#
let mut harness = bindings::harness();
let mut component = bindings::instantiate(harness);
let interface = component.component.namespace_package_interface();
let result = interface
    .call_function(&mut component.store, 0)
    .expect("failed to call function");
```
Where this `namespace_package_interface` function to fetch the interface for calling your
function is determined by the WIT namespace, package, and interface name.
And the `call_function` is just `call_` before your function name.

You can also get the amount of times a mocked or stubbed function is called by using
[`InstantiatedComponent::call_count`].

## Example
For the example we use the inline option, but this would normally go in `wit/world.wit`
instead.
```rust
mod bindings {
    wasmtime::component::bindgen!({
        inline: r"
            package namespace:%package;

            interface %interface {
                function: func(length: u32) -> string;
            }

            interface other-interface {
                other-function: func(value: string) -> string;
                another-function: func(value: string) -> string;
            }

            world main {
                import other-interface;
                export %interface;
            }
        "
    });

    wasmtime_testing_helper::setup!(Main);
}
let mut harness = bindings::harness();

harness.mock(
    "namespace:package/other-interface",
    "other-function",
    |_context, (value,): (String,)| Ok((value.to_uppercase(),)),
);
harness.stub::<(String,), (String,)>(
    "namespace:package/other-interface",
    "another-function",
    ("stubbed".to_string(),),
);

let mut component = bindings::instantiate(harness);
let interface = component.component.namespace_package_interface();
let result = interface
    .call_function(&mut component.store, 8)
    .expect("failed to call function");
assert_eq!(result, "STUBBED");
assert_eq!(
    component.call_count("namespace:package/other-interface", "other-function"),
    1,
);
assert_eq!(
    component.call_count("namespace:package/other-interface", "another-function"),
    1,
);
```

## Not implemented yet
Easy composition for integration testing two WASM components talking to one another is not yet
implemented.
