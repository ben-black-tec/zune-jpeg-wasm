
## build

Run
```
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=3 -C llvm-args=-ffast-math" wasm-pack  build --release --target web
```

Or to build/test faster
```
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=3 -C llvm-args=-ffast-math" cargo  build --release --target wasm32-unknown-unknown
```


### Chrome overhead notes

57s total time

10ms dft
10ms color
5ms upsampling
15ms huffman

### Firefox overhead notes

35ms

15ms huffman
2ms dft
3ms color convert
1ms upsample

(alt): scalar DFT 8ms