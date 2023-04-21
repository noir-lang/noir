# Noir contracts

## Building

- Install [noirup](https://github.com/noir-lang/noirup)
  ```
  curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
  ```
- Install the noir branch needed for A3 backend (`joss/ssa-2-brillig-plus-hacks` at the moment)
  ```
  noirup --branch arv/call_stack_item_oracle
  ```
- Move to the circuit you want to build
  ```
  cd src/contracts/test_contract
  ```
- Build using nargo
  ```
  nargo compile main --contracts
  ```

### Examples folder

The examples folder has friendly ABIs to be consumed from aztec3-client. They can be generated from the nargo build output using the `scripts/copy_output.ts`.
