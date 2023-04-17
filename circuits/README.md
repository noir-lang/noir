# Aztec 3 Circuit Onboarding

###### tags: `aztec-3, circuits`

## Repository Overview

The [`aztec3-circpuits`](https://github.com/AztecProtocol/aztec3-packages) circuits folder contains circuits and related C++ code (`cpp/`) for Aztec3 along with Typescript wrappers (`ts/`).

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

Here is an example of rapidly rebuilding and running all tests in `src/aztec3/circuits/abis/tests`:

```
cmake --preset clang15
cmake --build --preset clang15 --target aztec3_circuits_abis_tests
(cd build && ./bin/aztec3_circuits_abis_tests)
```

You can also limit it to run only specific test cases using a gtest filter:

```
(cd build && ./bin/aztec3_circuits_abis_tests --gtest_filter=*hash_tx_request*)
```

---

Here's a list of the tests currently available (conveniently combined with the command to build, then execute them, for easy copy-pasta):

- aztec3_circuits_abis_tests

  - `cmake --build --preset clang15 --target aztec3_circuits_abis_tests && (cd build && ./bin/aztec3_circuits_abis_tests --gtest_filter=*)`

- aztec3_circuits_apps_tests

  - `cmake --build --preset clang15 --target aztec3_circuits_apps_tests && (cd build && ./bin/aztec3_circuits_apps_tests --gtest_filter=*)`

- aztec3_circuits_kernel_tests

  - `cmake --build --preset clang15 --target aztec3_circuits_kernel_tests && (cd build && ./bin/aztec3_circuits_kernel_tests --gtest_filter=*)`

- aztec3_circuits_recursion_tests

  - `cmake --build --preset clang15 --target aztec3_circuits_recursion_tests && (cd build && ./bin/aztec3_circuits_recursion_tests --gtest_filter=*)`

- aztec3_circuits_rollup_tests
  - `cmake --build --preset clang15 --target aztec3_circuits_rollup_tests && (cd build && ./bin/aztec3_circuits_rollup_tests --gtest_filter=*)`

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
- `oracle`: used to fetch external information (like private data notes) and inject them as inputs into the circuit "Composer" during execution of circuit logic ([more here](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/oracle))
- `dbs`: database infrastructure (_e.g._ PrivateStateDb)

### Typescript

All typescript code was moved from here into `aztec3-packages/yarn-project/circuits.js`.

## Private Kernel Circuit

The private kernel circuit validates that a particular private function was correctly executed by the user. Therefore, the private kernel circuit is going to be run on the user's device. A private function execution can involve calls to other private functions from the same contract or private functions from other contracts. Each call to another private function needs to be proven that the execution was correct. Therefore, each nested call to another private function will have its own circuit execution proof, and that proof must then be validated by the private kernel circuit. The proof generated by the private kernel circuit will be submitted to the transaction pool, from where rollup providers will include those private kernel proofs in their L2 blocks.

The private kernel circuit in Aztec 3 is implemented in [`circuit/kernel/private`](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/kernel/private) directory. The input and output interface of the private kernel circuit is written in [`circuits/abis/private_kernel`](https://github.com/AztecProtocol/aztec3-packages/tree/master/circuits/cpp/src/aztec3/circuits/abis/private_kernel).

See pseudocode in [this diagram](https://miro.com/app/board/uXjVPlafJWM=/) and the slides [here](https://drive.google.com/file/d/1BaspihHDUgny6MHAKMtTkWKvfah7PYtv/view?usp=share_link) for a deeper dive into the private kernel circuit.

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
