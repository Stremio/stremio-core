#!/bin/sh
set -ex
wasm-pack build --no-typescript --out-dir wasm_build --release --target web
mv ./wasm_build/state_container_web_bg.wasm ./static/state_container_web.wasm
./node_modules/.bin/babel src/lib.js --out-file build/lib.js --presets=@babel/preset-env
./node_modules/.bin/babel wasm_build/state_container_web.js --out-file build/state_container_web.js --presets=@babel/preset-env
