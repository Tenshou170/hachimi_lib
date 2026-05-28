<div align="center">
  <h1>hachimi_lib</h1>
  <p><b>High-performance, tag-aware text wrapping and text isolation core engine in Rust.</b></p>

  <p>
    <img src="https://img.shields.io/badge/rust-2021-000000?logo=rust&logoColor=white&style=for-the-badge" alt="Rust 2021 Edition">
    <img src="https://img.shields.io/badge/wasm--pack-compatible-blue?logo=webassembly&logoColor=white&style=for-the-badge" alt="wasm-pack buildable">
  </p>
  <p>
    <a href="https://github.com/Tenshou170/hachimi_lib_js"><img src="https://img.shields.io/badge/JS%2FWASM-Bindings-orange?logo=javascript&style=flat-square" alt="JS/WASM bindings"></a>
    <a href="https://github.com/Tenshou170/hachimi_lib/blob/main/LICENSE"><img src="https://img.shields.io/github/license/Tenshou170/hachimi_lib?color=orange&style=flat-square" alt="MIT License"></a>
  </p>
</div>

---

`hachimi_lib` is the core text-processing engine for the Hachimi ecosystem. Written in optimized Rust, it provides advanced text isolation and optimal-fit wrapping logic that is fully aware of both Unity-style formatting tags (e.g. `<size=...>`, `<color=...>`) and template expressions (e.g. `$(...)`).

This crate is designed for dual use:
1.  As a Rust library dependency in injection frameworks (like [Hachimi-Edge](https://github.com/Tenshou170/Hachimi-Edge)).
2.  As a WebAssembly module compiled for TypeScript/JavaScript webviews (like [ZokuZoku-Edge](https://github.com/Tenshou170/ZokuZoku-Edge)).

---

## ✨ Features

*   🏷️ **Tag & Expression Awareness**: Parses and isolates tags (`<...>` style) and template expressions (`$(...)` style) dynamically so they are excluded from physical layout wrap calculations.
*   🚀 **Optimal Line-Fit wrapping**: Uses Knuth-style penalty parameters to compute visually balanced line endings.
*   🌐 **Unicode Support**: Leverages standard segmenters and Unicode properties to maintain clean line breaks across multiple scripts.
*   ⚡ **Ultra-Fast WebAssembly Compiles**: Designed for zero-overhead, synchronous bindings inside JS engines.

---

## 📦 Usage (Rust)

Add the git dependency to your `Cargo.toml`:

```toml
[dependencies]
hachimi_lib = { git = "https://github.com/Tenshou170/hachimi_lib.git" }
```

Use `wrap_text` in your code:

```rust
use hachimi_lib::wrap_text;

let text = "Hello <color=red>world</color>! This is $(name).";
let wrapped = wrap_text(text, 20, 1.0);
for line in wrapped {
    println!("{}", line);
}
```

---

## 🌀 WebAssembly Compilation

Compile to a web-compatible WebAssembly module using `wasm-pack`:

```bash
wasm-pack build --target web --out-dir ../hachimi_lib_js
```

---

## 🔗 Bindings
*   **JavaScript/WASM**: [`hachimi_lib_js`](https://github.com/Tenshou170/hachimi_lib_js) (distribution package for web and VS Code webviews).

---

## ⚖️ License

Distributed under the **MIT** License. See the [LICENSE](LICENSE) file for details.
