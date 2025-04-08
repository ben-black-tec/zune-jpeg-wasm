
## build

Run  
```
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=3 -C llvm-args=-ffast-math" wasm-pack  build --release --target web
```

Or to build/test faster
```
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=3 -C llvm-args=-ffast-math" cargo  build --release --target wasm32-unknown-unknown
```