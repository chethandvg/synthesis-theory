# Synthesis theory: WebAssembly bindings

WebAssembly build of [synthesis-theory](https://github.com/chethandvg/synthesis-theory) for browsers and Node. Function names match the Rust crate; signals come back as `Float64Array`, and sample rates, seeds and counts must be whole numbers. WebAssembly has its own maths library, so results that use exp, log or sin can differ from the native build in the last bit; CI checks every result against the shared golden vectors within their tolerances.

```bash
npm install
npm run build   # pkg/web for browsers, pkg/node for Node
npm test
```

Building needs Rust with the `wasm32-unknown-unknown` target and the `wasm-bindgen` CLI at the version pinned in `Cargo.toml`.
