# Circuits.js

Javascript types and helper functions for the aztec circuits.

## To run:

`yarn && yarn test`

## Updating Snapshots

The tests will fail if you've made changes to things like the Public Inputs for the rollup/kernel circuits or the accumulators,
logic, or hashes. This is because snapshot data in places like `src/abis/__snapshots__` will need to be updated for the tests.

You can update the snapshot data by running

```bash
yarn test -u
```

and committing the updated snapshot files.
