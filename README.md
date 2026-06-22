# Ferret

```
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⢄⣀⠀⡀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡤⠴⠖⠒⠶⠤⢄⣀⠀⠀⠀⠀⠀⠀⢀⠔⢹⣿⡯⠓⠈⠓⢼⠂⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣾⠟⠋⠁⠀⠀⠀⠀⠀⠀⠉⠒⣤⠤⢤⠔⠋⠀⠱⠇⠀⠀⣶⠶⠈⢆⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣯⣄⡀⠈⢢⡀⢀⣀⠀⠀⠀⠀⠈⠀⠀⠀⠀⡀⣰⣴⣷⣷⠾⣽⠿⠤⠟⠁
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⣿⣿⣿⠀⣸⡿⠿⣿⣿⣶⣶⣾⠗⠁⠀⣰⣾⣾⣿⠿⠋⠀⠨⠊⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⣀⣠⣶⣿⢯⣾⣿⣿⡿⣋⣴⣷⣶⣿⡿⠙⠻⠟⣿⡠⣶⣿⣿⣿⣿⣅⡀⠀⠀⠀⠀⠀⠀⠀⠀
⢀⣠⣶⣾⣿⣿⣿⣿⡿⠟⠛⠁⢸⣿⣏⡉⠉⠀⠿⣿⣏⣉⡀⠀⠀⠀⠙⠻⣿⣧⣈⣉⢹⣿⠇⠀⠀⠀⠀⠀⠀⠀⠀
⠋⠉⠉⠉⠉⠉⠉⠁⠀⠀⠀⠀⠈⠛⠛⠛⠀⠀⠀⠈⠉⠉⠉⠀⠀⠀⠀⠀⠈⠛⠛⠛⠃⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
```

Ferret is a free, open source Lua 5.4 VM obfuscator. It compiles supported Lua
source into a custom register bytecode format, encrypts bytecode and constants,
and emits a standalone Lua VM runtime. 

## Project

Ferret is developed and maintained by [luacrypt.com](https://luacrypt.com).

This project is not a commercial product sold by LuaCrypt. It is the open-source predecessor and core foundation of our upcoming Capybara project.

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) if you want to
help build the best OSS Lua obfuscator together.

## Use

```sh
cargo run -p ferret-cli -- obfuscate input.lua -o output.lua
```

For deterministic output:

```sh
cargo run -p ferret-cli -- obfuscate input.lua -o output.lua --seed 7
```

For metadata:

```sh
cargo run -p ferret-cli -- obfuscate input.lua -o output.lua --metadata meta.json
```

To benchmark the OSS obfuscator over Lua corpora:

```sh
cargo run -p ferret-cli -- bench tests/fixtures tests/scripts tests/lua54_conformance
```

To include native Lua vs generated VM runtime overhead:

```sh
cargo run --release -p ferret-cli -- bench --runtime-overhead --runtime-runs 3 tests/bench/runtime_overhead.lua
```

For repeated corpus timing:

```sh
cargo run --release -p ferret-cli -- bench --runtime-overhead --runtime-runs 3 --runtime-inner-runs 200 --runtime-known-mismatches tests/bench/runtime_mismatch_exclusions.txt tests/fixtures tests/scripts tests/lua54_conformance
```

To build the WebAssembly package:

```sh
cargo build -p ferret-wasm --target wasm32-unknown-unknown
```

## Development

All public obfuscation is VM-only. Unsupported Lua features fail with a clear
diagnostic instead of falling back to source reconstruction.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```
