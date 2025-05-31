# RoomMD

_Creating building (wiring) diagrams should be as easy as writing markdown._

View simple ascii sketches of a house as 3D models. Each utf-8 character corresponts to an certain object you can describe if you want. If two rooms contain the same characters like _D_ for _D_oor or _S_ for _Staircase_ e.g., the software will put these two rooms together as shown in the example. Use whatever characters you like. It is also possible to display wires that to through multiple rooms as there is no size limits to the ascii sketches.

Use https://772.github.io/roommd/!

![grafik](https://github.com/user-attachments/assets/0318a859-19bf-4014-a5c6-1356c452b9d6)

## How to update the wasm branch in this repository

```
cargo b
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
git add -A && git commit -m "Update" && git push
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --target web --out-dir ./ --out-name "roommd" ./target/wasm32-unknown-unknown/release/roommd.wasm
git checkout --no-overlay wasm
git add roommd.js roommd_bg.wasm index.html example.md
git commit -m "Update wasm files."
git push
git checkout main
```
