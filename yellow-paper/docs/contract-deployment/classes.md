# Contract classes

A contract class is a collection of related unconstrained, private, and public functions. Contract classes don't have state, they just define code.

## Rationale

Contract classes simplify the process of reusing code by enshrining implementations as a first-class citizen at the protocol. Given multiple contract instances that rely on the same class, the class needs to be declared only once, reducing the deployment cost for all contract instances. Classes also simplify the process of upgradeability. Classes decouple state from code, making it easier for an instance to switch to different code while retaining its state.

:::info
Read the following discussions for additional context:

- [Abstracting contract deployment](https://forum.aztec.network/t/proposal-abstracting-contract-deployment/2576)
- [Implementing contract upgrades](https://forum.aztec.network/t/implementing-contract-upgrades/2570)
- [Contract classes, upgrades, and default accounts](https://forum.aztec.network/t/contract-classes-upgrades-and-default-accounts/433)
:::

## Structure

The structure of a contract class is defined as:

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| version | u8 | Version identifier. Initially one, bumped for any changes to the contract class struct. |
| registerer_address | AztecAddress | Address of the canonical contract used for registering this class. |
| artifact_hash | Field | Hash of the entire contract artifact, including compiler information, proving backend, and all generated ACIR and Brillig. The specification of this hash is left to the compiler and not enforced by the protocol. |
| constructor_function | PrivateFunction | PublicFunction | Constructor for instances of this class. |
| private_functions | PrivateFunction[] | List of private functions. |
| public_functions | PublicFunction[] | List of public functions. |
| unconstrained_functions | UnconstrainedFunction[] | List of unconstrained functions. |

<!-- TODO: Do we need the artifact hash, if we're including the artifact hash of each individual function? -->
<!-- NOTE: I'm deliberately omitting the portal bytecode hash here. -->

### Private Function

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| function_selector | u32 | Selector of the function. Calculated as the hash of the method name and arguments. |
| vk_hash | Field | Hash of the verification key associated to this private function. |
| salt | Field | Optional value for salting the bytecode of a function. |
| bytecode_hash | Field | Hash of the compiled function artifact, including all generated ACIR and Brillig. |
| optional_bytecode | Buffer | Optional bytecode for the function. Private function bytecode can be kept private if desired and only broadcasted to participants of the contract. |

### Public and Unconstrained Function

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| function_selector | u32 | Selector of the function. Calculated as the hash of the method name and arguments. |
| bytecode_hash | Field | Hash of the compiled function artifact, including all generated Brillig code. |
| bytecode | Buffer | Full bytecode for the function. Must hash to the `artifact_hash`. |

<!-- TODO: Expand on the bytecode commitment scheme and bytecode_hash, both here and for private fns. -->

### Class Identifier

The class identifier is computed by merkleizing the lists of private, public, and unconstrained functions separately, replacing the functions lists in the contract class struct with their respective tree roots, and then hashing the resulting struct.

## Registration

A contract class is registered by calling a private `register` function in a canonical `ClassRegisterer` contract. The `register` function receives a `ContractClass` struct as defined above, except for the `registerer_address`, and performs the following checks:

- `version` is 1 for the initial release
- `bytecode` for each function hashes to the `bytecode_hash`

The `register` function then:

- Emits the `ContractClass` struct as unencrypted events.
- Computes the class identifier as the hash of the `ContractClass` object.
- Emits the computed class identifier as a nullifier.

Upon seeing a new contract class registration event in a mined transaction, nodes are expected to store the contract class, so they can retrieve it when executing a public function for that class.

Note that emitting the class identifier as a nullifier, instead of as an entry in the note hashes tree, allows public functions to prove non-existence of a class, which is required to support public contract instance deployments.
