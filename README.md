# RoomMD

_Creating building (wiring) diagrams should be as easy as writing markdown._

View simple ascii sketches of a house as 3D models. Each utf-8 character corresponts to an certain object you can describe if you want. If two rooms contain the same characters like _D_ for _D_oor or _S_ for _Staircase_ e.g., the software will put these two rooms together as shown in the example. Use whatever characters you like. It is also possible to display wires that to through multiple rooms as there is no size limits to the ascii sketches.

## How to update the wasm in this repository

```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --target web --out-dir ./ --out-name "roommd" ./target/wasm32-unknown-unknown/release/roommd.wasm
git add -A && git commit -m "Update." && git push
```
