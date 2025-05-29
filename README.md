# RoomMD

_Creating building (wiring) diagrams should be as easy as writing markdown._

View simple ascii sketches of a house as 3D models. Each utf-8 character corresponts to an certain object you can describe if you want. If two rooms contain the same characters like _D_ for _D_oor or _S_ for _Staircase_ e.g., the software will put these two rooms together as shown in the example. Use whatever characters you like. It is also possible to display wires that to through multiple rooms as there is no size limits to the ascii sketches.

Use https://772.github.io/roommd/!

## How to update the wasm branch in this repository

```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --target web --out-dir ./ --out-name "roommd" ./target/wasm32-unknown-unknown/release/roommd.wasm
git checkout pages
git add roommd.js roommd_bg.wasm index.html
git commit -m "Update wasm files."
git push
git checkout main
```

If you haven`t used wasm so far, use this first:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```
