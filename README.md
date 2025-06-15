_Creating building (wiring) diagrams should be as easy as writing markdown._

## What is RoomMD?

View simple ascii sketches of a house as 3D models with this web application. Each utf-8 character corresponts to an certain object you can describe if you want. If two rooms contain the same characters like _D_ for _D_oor or _S_ for _Staircase_ e.g., the software will put these two rooms together as shown in the example. Use whatever characters you like. It is also possible to display wires that to through multiple rooms as there is no size limits to the ascii sketches.

## Usage

Open [RoomMD](https://772.github.io/roommd/).

## Info

- Minimalistic code with less than 600 lines. Not optimized, yet.
- Programmed via safe Rust and the [Bevy Engine](https://bevyengine.org/). This is an example of using [bevy::render::render_resource::Face::Front](https://docs.rs/bevy/latest/bevy/render/render_resource/enum.Face.html).

## How to update the wasm branch in this repository

Note that after the WebAssembly branch was initially created, I deleted all files in it.

```
cargo b
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
git add -A && git commit -m "Update."
git push
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --no-typescript --target web --out-dir ./../ --out-name "wasm" ./target/wasm32-unknown-unknown/release/*.wasm
cp index.html ..
cp example.md ..
git checkout wasm
mv ../index.html .
mv ../example.md .
mv ../wasm.js .
mv ../wasm_bg.wasm .
git add wasm.js wasm_bg.wasm index.html example.md
git commit -m "Update wasm files."
git push
git checkout main
```

## TODO

- Texture for rooms. Then update screenshot.
- Stop using the url parameter as input since its size is too limited. Then add more rooms to the example.
- When placing rooms near to other rooms, the normalized position should be used which will lead to more hits. Then use an offset to place use the correct position. Rooms shouldn't need to have the same height/depth/width.
- Better HTML editor that warns if rooms have inconsitent data.
