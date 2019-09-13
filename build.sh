#!/bin/sh
set -ex
wasm-pack build --no-typescript --out-dir wasm_build --release --target web
mv ./wasm_build/stremio_core_web_bg.wasm ./static/stremio_core_web.wasm
npx babel wasm_build/stremio_core_web.js --config-file ./.babelrc --out-file build/stremio_core_web.js
npx babel src/lib.js --config-file ./.babelrc --out-file build/lib.js
