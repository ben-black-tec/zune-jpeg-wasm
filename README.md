
## build

Run  
```
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=3 -C llvm-args=-ffast-math" wasm-pack  build --release --target web
```