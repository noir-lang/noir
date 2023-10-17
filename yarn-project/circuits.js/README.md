# Circuits.js

Javascript bindings for the aztec3 circuits WASM.
High-level bindings to the raw C API to our core circuit logic.

## To run:

`yarn && yarn test`

## Updating Snapshots

The tests will fail if you've made changes to things like the Public Inputs for the rollup/kernel circuits or the accumulators,
logic, or hashes. This is because snapshot data in places like `src/structs/__snapshots__` or `src/abis/__snapshots__` will need to be updated for the tests.

You can update the snapshot data by running

```bash
yarn test -u
```

and committing the updated snapshot files.

## To rebundle local dependencies from aztec3-packages

Currently relies on dependencies locally linked from `aztec3-packages`.
Run `yarn bundle-deps` to rebundle them (committed to the repo for simplicity).
Run `yarn dev-deps` if you have ../../.. as the `aztec3-packages` path.

TODO worker API
