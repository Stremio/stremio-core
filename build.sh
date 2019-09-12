#!/bin/sh
set -ex
wasm-pack build --no-typescript --out-dir wasm_build --release --target web
mv ./wasm_build/stremio_core_web_bg.wasm ./static/stremio_core_web.wasm
sed -i -e 's/import.meta.url/""/g' wasm_build/stremio_core_web.js
./node_modules/.bin/babel wasm_build/stremio_core_web.js --out-file build/stremio_core_web.js --presets=@babel/preset-env
./node_modules/.bin/babel src/lib.js --out-file build/lib.js --presets=@babel/preset-env
