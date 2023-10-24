# Stremio Core Web

[![npm](https://img.shields.io/npm/v/@stremio/stremio-core-web?style=flat-square)](https://www.npmjs.com/package/@stremio/stremio-core-web)

Bridge between [stremio-core](https://github.com/stremio/stremio-core) and [stremio-web](https://github.com/stremio/stremio-web)


## Build

Builds a production wasm package and prepares the rest of the dependencies for the npm package.

```
npm install
npm run build
```

### Development

Building the package using [`./scripts/build.sh`](./scripts/build.sh) with `--dev` would allow you to see more logging messages being emitted, this is intended **only** for debugging as it will log messages with sensitive information!

```
./scripts/build.sh --dev
```

## Publishing

1. Update version to the next minor/major/patch version in Cargo (`Cargo.toml` and `Cargo.lock`) and npm (`package.json` and `package-lock.json`), e.g. from `0.44.13` to `0.44.14`.
2. Commit the change with the new version as a message, e.g. `0.44.14`
3. Wait for CI to build successfully
4. Push a new tag starting with `v`, e.g. `git tag v0.44.14` `git push origin v0.44.14`
5. Create a [new Release](https://github.com/Stremio/stremio-core-web/releases/new) with the created tag and the tag name as a title, e.g. `v0.44.14`
6. Publish the Release
7. CI will automatically build and release the `npm` package to the registry