_Creating building (wiring) diagrams should be as easy as writing markdown._

[![License: GPL3](https://img.shields.io/badge/License-GNU%20GPL-blue)](https://opensource.org/license/gpl-3-0)
[![Crate](https://img.shields.io/crates/v/roommd.svg)](https://crates.io/crates/roommd)

## What is RoomMD?

View simple ascii sketches of a house as 3D models with this web application. Each utf-8 character corresponts to an certain object you can describe if you want. If two rooms contain the same characters like _D_ for _D_oor or _S_ for _Staircase_ e.g., the software will put these two rooms together as shown in the example. Use whatever characters you like. It is also possible to display wires that to through multiple rooms as there is no size limits to the ascii sketches.

## Usage

- **Web**: [RoomMD WebAssembly](https://772.github.io/roommd/).
- **Installed**: `roommd example.md` (install via ```cargo install roommd```).
- **From source**: ```cargo run example.md``` (after cloning this repository).

https://github.com/user-attachments/assets/cc425997-2444-4089-b27f-a17cc8623284

## Info

- Programmed via safe Rust and the [Bevy Engine](https://bevyengine.org/). This is an example of using [bevy::render::render_resource::Face::Front](https://docs.rs/bevy/latest/bevy/render/render_resource/enum.Face.html).
- Make sure the width of the back wall is matching with the sides of the ceiling and so on. The software won't start if there is any error.

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
cp ../wasm.js .
cp ../wasm_bg.wasm .
git checkout wasm
mv ../index.html .
mv ../example.md .
mv ../wasm.js .
mv ../wasm_bg.wasm .
git add wasm.js wasm_bg.wasm index.html example.md
git commit -m "Update wasm files."
git push -f
git checkout main
```

