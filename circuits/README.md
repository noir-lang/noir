# Aztec 3 Circuit Onboarding

###### tags: `aztec-3, circuits`

## Contributing

See [CODING_STANDARD.md](./CODING_STANDARD.md)\*\* before contributing!

## Repository Overview

The [`aztec3-circuits`](https://github.com/AztecProtocol/aztec3-packages) circuits folder contains circuits and related C++ code (`cpp/`) for Aztec3 along with Typescript wrappers (`ts/`).

### Dependencies

- cmake >= 3.24
- Ninja (used by the presets as the default generator)
- clang15
- clang-format
- wasm-opt (part of the [Binaryen](https://github.com/WebAssembly/binaryen) toolkit)
- [wasmtime](https://docs.wasmtime.dev/cli-install.html) for running tests in a wasm environment

### Resources

- [Circuits project board](https://github.com/orgs/AztecProtocol/projects/22/views/2)
- [[DO NOT EDIT] Diagram with pseudocode for circuits](https://miro.com/app/board/uXjVPlafJWM=/)
  - This diagram's contents are likely more up-to-date than code and than the other links below
- Kernel circuits
  - [Slides - Dive into Kernel Circuits](https://drive.google.com/file/d/1BaspihHDUgny6MHAKMtTkWKvfah7PYtv/view?usp=share_link)
    - [Recording of presentation](https://drive.google.com/file/d/1Uh-vLdc1_rsMUHL_c4HZ93jrjpuDsqD3/view?usp=share_link)
  - [Kernel circuit mentioned in M1.1 Contract Deployment document](https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Kernel-Circuit-functionality)
  - [More info on the kernel circuit for Contract Creation](https://hackmd.io/@aztec-network/rkyRaXqPj#Kernel-Circuit-Logic)
  - _Note:_ [Base rollup circuits in Aztec Connect](https://github.com/AztecProtocol/aztec-connect-cpp/tree/defi-bridge-project/src/rollup/proofs/rollup) are relevant to kernel circuits
- Rollup circuits
  - [Rollup circuit mentioned in M1.1 Contract Deployment document](https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Rollup-Circuit-functionality)
  - [More info on the rollup circuit for Contract Creation](https://hackmd.io/@aztec-network/rkyRaXqPj#Rollup-Circuit-Logic)
  - [Rollup circuits in Aztec Connect](https://github.com/AztecProtocol/aztec-connect-cpp/tree/defi-bridge-project/src/rollup/proofs)
  - [[DO NOT EDIT] Diagram with different options for merkle insertions in rollup circuits](https://miro.com/app/board/uXjVMfITC3c=/)
- [Outdated specs](https://github.com/AztecProtocol/aztec2-internal/blob/3.0/markdown/specs/aztec3/src/SUMMARY.md)
- [Explanation of Indexed Merkle Tree (for nullifiers)](https://hackmd.io/AjR1uGh8SzSc7k3Gcu02mQ)

### Getting Started with C++

Clone the repo and build the C++:

```
git clone git@github.com:AztecProtocol/aztec3-packages.git
cd circuits
git submodule update --init --recursive
cd cpp
./bootstrap.sh
```

Here is an example of rapidly rebuilding and running all tests for `x86_64`:

```
./bootstrap.sh
./scripts/run_tests_local x86_64 glob
```

> **WARNING:** the `x86_64` (and `wasm` used below) as well as the keyword `glob` **MUST BE LOWERCASE**!

Here is an example of rapidly rebuilding and running only the abis tests for `wasm`:

```
./bootstrap.sh aztec3_circuits_abis_tests
./scripts/run_tests_local wasm aztec3_circuits_abis_tests
```

> _Note:_ to run wasm tests you must first follow the [instructions here](https://docs.wasmtime.dev/cli-install.html) to install `wasmtime`.

You can choose which tests will run via a gtest filter. This one below runs only tests that _omit_ the string '.circuit':

```
./scripts/run_tests_local wasm aztec3_circuits_abis_tests -*.circuit*
```

---

Here's a list of the tests currently available (conveniently combined with the command to build, then execute them on `x86_64`, for easy copy-pasta):

- `aztec3_circuits_abis_tests`

  - `./bootstrap.sh aztec3_circuits_abis_tests && ./scripts/run_tests_local x86_64 aztec3_circuits_abis_tests`

- `aztec3_circuits_apps_tests`

  - `./bootstrap.sh aztec3_circuits_apps_tests && ./scripts/run_tests_local x86_64 aztec3_circuits_apps_tests`

- `aztec3_circuits_kernel_tests`

  - `./bootstrap.sh aztec3_circuits_kernel_tests && ./scripts/run_tests_local x86_64 aztec3_circuits_kernel_tests`

- `aztec3_circuits_recursion_tests`

  - `./bootstrap.sh aztec3_circuits_recursion_tests && ./scripts/run_tests_local x86_64 aztec3_circuits_recursion_tests`

- `aztec3_circuits_rollup_tests`
  - `./bootstrap.sh aztec3_circuits_rollup_tests && ./scripts/run_tests_local x86_64 aztec3_circuits_rollup_tests`

---

#### Using docker to replicate CI failures

You can also run tests in docker. This is useful for replicating CI failures that you can't replicate with your standard local environment.

To build and run all tests in an `x86_64` docker image:

```
./bootstrap.sh
./scripts/build_run_tests_docker_local 1 x86_64 glob
```

You can choose `wasm` instead of `x86_64`. You can also specify individual test executables instead of `glob` and can use gtest filters exactly as described for `run_tests_local`.

> At this time, it is common to run wasm tests with the filter `-*.circuit*` as there are circuit issues in wasm.

> The `build_run_tests_docker_local` script builds the chosen docker image (`x86_64` or `wasm`) and then launches a container from that image to run the `run_tests_local` script (used above).

#### Generating code coverage reports

You can generate coverage reports for your tests.
To build and run coverage on all tests:

```
./bootstrap.sh
./scripts/run_coverage
```

Producing coverage reports is computationally intensive
You can select a specific test suite to run coverage on by supplying it as an argument to the `run_coverage`. For example, to only compile and produce

```
./bootstrap.sh
./scripts/run_coverage aztec3_circuits_abis_tests
```

**Toggles**
Running with the `CLEAN` environment variable set will delete the existing `build-coverage` folder.
Running with the `CLEAR_COV` environment variable will delete any existing `lcov.info` file.

#### Viewing coverage reports

Once a report has been generated, you can view them within the `build-coverage` folder in html format. If you ran coverage with any tests in mind, the report will exist in a folder prefixed with its name, otherwise they can be found in one labelled all_tests.

#### Viewing coverage reports inside vscode

It may be useful to view coverage information from within vscode. The `./scripts/run_coverage` will produce an `lcov.info` file that should automatically be picked up by the `coverage-gutters` vscode extension.

---

#### Using the VSCode debugger

> **WARNING:** to debug in WASM (to use the `-g` option to `wasmtime`) you will unfortunately need to revert to `wasmtime` version `1.0.0` until [this bug](https://github.com/bytecodealliance/wasmtime/issues/3999) is fixed. To install that version, remove the `~/.wasmtime` directory and run `curl https://wasmtime.dev/install.sh -sSf | bash /dev/stdin --version v1.0.0`

1. Make sure you have the recommended plugins installed
   - Open the command pallete (`Ctrl+Shift+P`)
     - `Cmd+Shift+P` on Macs
   - Type and select "Extensions: Show Recommended Extensions"
   - Install any plugins shown not already installed
1. Configure CMake for whichever preset you'd like to use
   - Open the command pallete (`Ctrl+Shift+P`)
   - Type and select "CMake: Select Configure Preset"
   - Choose a debug preset such as:
     - "Debugging build with Clang-15"
     - "Debugging build for WASM"
   - Redo this step later to switch between Clang-15/native and wasm
1. Go to the "Run and Debug" panel
   - Button (usually on left) that looks like a play button with a bug
   - Or `Ctrl+Shift+D`
1. Select the proper launch option at the top of the "Run and Debug" panel
   - "Launch native"
   - "Launch in WASM"
1. Select the test executable to debug
   - Open the command pallete (`Ctrl+Shift+P`)
   - Type and select "CMake: Set Debug Target"
   - Select executable to debug like `aztec3_circuits_abis_tests`
1. Check output for progress
1. [OPTIONAL] change `gtest_filter` args to filter specific test cases
   - In `circuits.code-workspace`'s `launch->configurations-><native or wasm>`
   - Don't commit these changes
1. [OPTIONAL] set breakpoints in C++ files

> _Note:_ redo steps 3-5 to switch between debugging Clang-15/native test executables and WASM test executables

---

### C++ Repository Layout

This repository submodules [`barretenberg`](https://github.com/AztecProtocol/barretenberg) as a C++ library containing proving systems and utilities at `cpp/barretenberg/`.

The core Aztec 3 C++ code lives in `cpp/src/aztec3/`, and is split into the following subdirectories/files:

- `constants.hpp`: top-level constants relevant to Aztec 3
- `circuits`: circuits and their types and interfaces
  - `apps`: infrastructure and early prototypes for application circuits ([more here](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/apps))
  - `abis`: types, interfaces, and cbinds for representing/constructing outputs of application circuits that will be fed into kernel circuits ([more here](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/abis))
  - `kernel`: kernel circuits, their interfaces, and their tests
  - `rollup`: rollup circuits, their interfaces, and thier tests
  - `recursion`: types and examples for aggregation of recursive proof objects
  - `mock`: mock circuits
- `oracle`: used to fetch external information (like private data notes) and inject them as inputs into the circuit during execution of circuit logic ([more here](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/oracle))
- `dbs`: database infrastructure (_e.g._ PrivateStateDb)

### Typescript

All typescript code was moved from here into `aztec3-packages/yarn-project/circuits.js`.

## Private Kernel Circuit

The private kernel circuit validates that a particular private function was correctly executed by the user. Therefore, the private kernel circuit is going to be run on the user's device. A private function execution can involve calls to other private functions from the same contract or private functions from other contracts. Each call to another private function needs to be proven that the execution was correct. Therefore, each nested call to another private function will have its own circuit execution proof, and that proof must then be validated by the private kernel circuit. The proof generated by the private kernel circuit will be submitted to the transaction pool, from where rollup providers will include those private kernel proofs in their L2 blocks.

The private kernel circuit in Aztec 3 is implemented in [`circuit/kernel/private`](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/kernel/private) directory. The input and output interface of the private kernel circuit is written in [`circuits/abis/private_kernel`](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/abis/private_kernel).

See pseudocode in [this diagram](https://miro.com/app/board/uXjVPlafJWM=/) and the slides [here](https://drive.google.com/file/d/1BaspihHDUgny6MHAKMtTkWKvfah7PYtv/view?usp=share_link) for a deeper dive into the private kernel circuit.

## Public Kernel Circuit

The public kernel circuit performs many of the functions of the private kernel circuit but for public functions. Public functions differ from public functions in that they they are exeucted by the sequencer and can modify the public state tree. Like private functions, public functions can include calls to other public functions and these calls are validated within the circuit.

The public kernel circuit in Aztec 3 is implemented in [`circuit/kernel/public`](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/kernel/publlic) directory. The input and output interface of the private kernel circuit is written in [`circuits/abis/public_kernel`](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/abis/public_kernel).

## Circuit Overviews

### Base Rollup Circuit

The rollup providers (or sequencers - nomenclature is yet to be decided) pull up transactions from the pool to be rolled up. Each of these transactions is essentially a proof generated by the private kernel circuit including its public inputs. To accumulate several such transactions in a single L2 block, the rollup provider needs to aggregate the private kernel proofs. The base rollup circuit is meant to aggregate these private kernel proofs. In simple words, the rollup circuit validates that the private kernel circuit was correctly executed.

In our design, we allow the rollup providers to only aggregate _two_ private kernel proofs at once. This would mean that if a rollup provider wishes to roll-up 1024 transactions in one L2 block, for example, he would need $\frac{1024}{2} = 512$ invocations of the base rollup circuit. This hard-limit on the number of private kernel proofs that one can aggregate is enable generating rollup proofs on commodity hardware, effectively reducing the entry-barrier for common users to become rollup providers.

:::info
At a very high-level, the rollup circuits need to perform the following checks:

1. Aggregate the inner proofs (most expensive part taking up appx 75% circuit size)
2. Merkle membership checks (second most expensive part taking up appx 15% circuit size)
3. Public input hashing using SHA-256 (third most expensive part, appx 10%, can blow up if you need to hash tons of public inputs)

We have a limit on the first point: you can aggregate a maximum of two proofs per rollup circuit. We still need to decide if we wish to put any limits on the second step. Mike has an idea of splitting up the Merkle membership checks across multiple rollup circuits so that all of the Merkle membership computation doesn't need to be performed by a single circuit. Similarly, we need to ensure that a rollup circuit doesn't need to hash huge amounts of data in one stage.
:::

:::warning
**Note**: For the first milestone, we do not include public function execution in our circuit design.
:::

See pseudocode in [this diagram](https://miro.com/app/board/uXjVPlafJWM=/) for a deep dive into the Base Rollup Circuit's functionality.

### Merge Rollup Circuit

The base rollup proofs further need to be validated if they were executed correctly. The merge rollup circuit is supposed to verify that the base rollup proofs are correct. In principle, the design of the merge rollup circuit would be very similar to the base rollup circuit except the change in the ABIs. As with the base rollup circuit, every merge rollup circuit validates two base rollup proofs.

Furthermore, we can use the same merge rollup circuit to verify two merge rollup proofs to further compress the proof validation. This leads to a formation of a tree-like structure to create an L2 block.

See pseudocode in [this diagram](https://miro.com/app/board/uXjVPlafJWM=/) for a deep dive into the Merge Rollup Circuit's functionality.

:::warning
**Note**: We might not need the merge rollup circuit for the offsite but its anyway going to be very similar to the base rollup circuit.
:::

### Root Rollup Circuit

The root rollup circuit is the final circuit execution layer before the proof is sent to L1 for on-chain verification. The root rollup circuit verifies that the final merge rollup circuit was correctly executed. The proof from the root rollup circuit is verified by the rollup contract on Ethereum. So effectively, verifying that one proof on-chain gives a final green flag to whatever number of transactions that were included in that particular L2 block.

It is interesting to note that the root rollup circuit takes _one_ proof and outputs _one_ proof. The reason we do this is to switch the types of the proof: i.e. ultra-plonk/honk to standard-plonk. This is because standard-plonk proofs are cheaper to verify on-chain (in terms of gas costs).

See pseudocode in [this diagram](https://miro.com/app/board/uXjVPlafJWM=/) for a deep dive into the Root Rollup Circuit's functionality.

## Circuit Logic

### Private Kernel Circuit Logic

The private kernel circuit is recursive in that it will perform validation of a single function call, and then recursively verify its previous iteration along with the next function call.

Below is a list of the private kernel circuit's high-level responsibilities:

1. For the first iteration:
   - **[M1.1]** Validate the signature of a signed tx object
   - Validate the function in that tx object matches the one currently being processed
2. For all subsequent iterations:
   - Pop an item off of the kernel's dynamic callstack
   - Validate that this function matches the one currently being processed
3. Verify a 'previous' kernel circuit (mock for first iteration, always mocked for **[M1.1]**)
4. Verify the proof of execution for the function (or constructor **[M1.1]**) currently being processed
5. **[M1.1]** If this is a contract deployment, check the contract deployment logic
   - _[After M1.1]_ Includes checks for private circuit execution logic
6. **[M1.1]** Generate `contract_address` and its nullifier (inserted in later circuit)
7. **[M1.1]** Generate `new_contract_data` which is the contract tree leaf preimage
8. Copy the current function's callstack into kernel's dynamic callstack
9. Validate the function data against `function_tree_root`
   - Includes a membership check of the function leaf
10. Validate the contract data against `contract_tree_root`
    - Includes a membership check of the contract leaf
11. Perform membership checks for commitments accessed by this function
12. Collect new commitments, nullifiers, contracts, and messages to L1
13. Add recursion byproducts to `aggregation_object`
14. TODO: L1 messages
15. Section in progress...
