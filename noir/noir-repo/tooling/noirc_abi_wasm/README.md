# Noir Lang ABI JavaScript Package

This JavaScript package enables users to ABI encode inputs to a Noir program, i.e. generating an initial witness.

## Building from source

Outside of the [noir repo](https://github.com/noir-lang/noir), this package can be built using the command below:

```bash
nix build -L github:noir-lang/noir/master#abi_wasm
```

If you are within the noir repo and would like to build local changes, you can use:

```bash
nix build -L #abi_wasm
```
