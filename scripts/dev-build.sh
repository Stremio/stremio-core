#!/bin/sh
set -ex
wasm-pack build --no-typescript --no-pack --out-dir wasm_build --dev --target web
mv ./wasm_build/stremio_core_web_bg.wasm stremio_core_web_bg.wasm
npx babel wasm_build/stremio_core_web.js --config-file ./.babelrc --out-file stremio_core_web.js
npx babel src/bridge.js --config-file ./.babelrc --out-file bridge.js
npx babel src/worker.js --config-file ./.babelrc --out-file worker.js
