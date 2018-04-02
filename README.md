# Motor

## Overview

*(This is very early development version. Nothing works but patches are welcome!)*

Motor is a [WebAssembly](http://webassembly.org/) runtime, which aims for secure and efficient execution of WebAssembly programs. The runtime is implemented in the [Rust](https://www.rust-lang.org/en-US/) programming language.

## Getting Started

To run a simple WebAssembly module, type:

```bash
$ cargo run test/start.wasm
```

### Building WebAssembly Modules

To build a WebAssembly module, use any of the existing compilers out there. The `test` directory contains some modules, which were translated from the WebAssembly text format (`.wat`) to the binary format (`.wasm`) with the `wat2wasm` tool provided by the [WABT](https://github.com/WebAssembly/wabt) toolkit.

## Documentation

* [WebAssembly Specification](https://webassembly.github.io/spec/)

## Licensing

Motor is licensed under the Apache 2.0 license. See the file [LICENSE](LICENSE) for details.
