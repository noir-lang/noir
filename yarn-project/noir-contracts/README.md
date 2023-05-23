# Noir contracts

This package contains the source code and the Aztec ABIs for the example contracts used in tests.

## Building the contracts

- Install [noirup](https://github.com/noir-lang/noirup)
  ```
  curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
  ```
- Install the noir branch needed for A3 backend (`aztec3-hacky` at the moment)
  ```
  noirup --branch aztec3-hacky
  ```
- Use the `noir:build:all` script to compile the contracts you want and prepare the ABI for consumption
  ```
  yarn noir:build:all
  ```

Alternatively you can run `yarn noir:build CONTRACT1 CONTRACT2...` to build a subset of contracts:
```
yarn noir:build zk_token public_token
```

To view compilation output, including errors, run with the `VERBOSE=1` flag:
```
VERBOSE=1 yarn noir:build zk_token public_token
```
