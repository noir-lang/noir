# Noir wasm

## Dependencies

In order to build the wasm package, the following must be installed:

- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [jq](https://github.com/stedolan/jq)

## Build

The wasm package can be built using the command below:

```bash
./build-wasm
```

Using `wasm-pack` directly isn't recommended as it doesn't generate a complete `package.json` file, resulting in files being omitted from the published package.
