# Contract classes

A contract class is a collection of state variable declarations, and related unconstrained, private, and public functions. Contract classes don't have any initialized state, they just define code. A contract class cannot be called; only a contract instance can be called.

## Rationale

Contract classes simplify the process of reusing code by enshrining implementations as a first-class citizen at the protocol. Given multiple [contract instances](./instances.md) that rely on the same class, the class needs to be declared only once, reducing the deployment cost for all contract instances. Classes also simplify the process of upgradeability; classes decouple state from code, making it easier for an instance to switch to different code while retaining its state.

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
| `version` | `u8` | Version identifier. Initially one, bumped for any changes to the contract class struct. |
| `artifact_hash` | `Field` | Hash of the contract artifact. The specification of this hash is not enforced by the protocol. Should include commitments to unconstrained code and compilation metadata. Intended to be used by clients to verify that an off-chain fetched artifact matches a registered class. |
| `private_functions` | [`PrivateFunction[]`](#private-function) | List of individual private functions, constructors included. |
| `packed_public_bytecode` | `Field[]` | [Packed bytecode representation](../public-vm/bytecode-validation-circuit.md#packed-bytecode-representation) of the AVM bytecode for all public functions in this contract. |

Note that individual public functions are not first-class citizens in the protocol, so the contract entire public function bytecode is stored in the class, unlike private or unconstrained functions which are differentiated individual circuits recognized by the protocol.

As for unconstrained functions, these are not used standalone within the protocol. They are either inlined within private functions, or called from a PXE as _getters_ for a contract. Calling from a private function to an unconstrained one in a different contract is forbidden, since the caller would have no guarantee of the code run by the callee. Considering this, unconstrained functions are not part of a contract class at the protocol level.

### Class Identifier

Also known as `contract_class_id`, the Class Identifier is both a unique identifier and a commitment to the struct contents. It is computed as:

```
private_function_leaves = private_functions.map(fn => pedersen([fn.function_selector as Field, fn.vk_hash], GENERATOR__FUNCTION_LEAF))
private_functions_root = merkleize(private_function_leaves)
public_bytecode_commitment = calculate_commitment(packed_public_bytecode)
contract_class_id = pedersen([artifact_hash, private_functions_root, public_bytecode_commitment], GENERATOR__CLASS_IDENTIFIER_V1)
```

Private Functions are hashed into Function Leaves before being merkleized into a tree of height `FUNCTION_TREE_HEIGHT=5`. Empty leaves have value `0`. A poseidon hash is used. The AVM public bytecode commitment is calculated as [defined in the Public VM section](../public-vm/bytecode-validation-circuit.md#committed-representation).

### Private Function

The structure of each private function within the protocol is the following:

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `function_selector` | `u32` | Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. |
| `vk_hash` | `Field` | Hash of the verification key associated to this private function. |

Note the lack of visibility modifiers. Internal functions are specified as a macro, and the check is handled at the application circuit level by verifying that the `context.msg_sender` equals the contract current address.

Also note the lack of commitment to the function compilation artifact. Even though a commitment to a function is required so that the PXE can verify the execution of correct unconstrained Brillig code embedded within private functions, this is handled entirely out of protocol. As such, PXEs are expected to verify it against the `artifact_hash` in the containing contract class.

### Artifact Hash

Even though not enforced by the protocol, it is suggested for the `artifact_hash` to follow this general structure, in order to be compatible with the definition of the [`broadcast` function below](#broadcast).

```
private_functions_artifact_leaves = artifact.private_functions.map(fn => 
  sha256(fn.selector, fn.metadata_hash, sha256(fn.private_bytecode))
)
private_functions_artifact_tree_root = merkleize(private_functions_artifact_leaves)

unconstrained_functions_artifact_leaves = artifact.unconstrained_functions.map(fn => 
  sha256(fn.selector, fn.metadata_hash, sha256(fn.unconstrained_bytecode))
)
unconstrained_functions_artifact_tree_root = merkleize(unconstrained_functions_artifact_leaves)

artifact_hash = sha256(
  private_functions_artifact_tree_root,
  unconstrained_functions_artifact_tree_root, 
  artifact_metadata,
)
```

For the artifact hash merkleization and hashing is done using sha256, since it is computed and verified outside of circuits and does not need to be SNARK friendly. Fields are left-padded with zeros to 256 bits before being hashed. Function leaves are sorted in ascending order before being merkleized, according to their function selectors. Note that a tree with dynamic height is built instead of having a tree with a fixed height, since the merkleization is done out of a circuit.

Bytecode for private functions is a mix of ACIR and Brillig, whereas unconstrained function bytecode is Brillig exclusively, as described on the [bytecode section](../bytecode/index.md).

The metadata hash for each function is suggested to be computed as the sha256 of all JSON-serialized fields in the function struct of the compilation artifact, except for bytecode and debug symbols. The metadata is JSON-serialized using no spaces, and sorting ascending all keys in objects before serializing them.

```
function_metadata = omit(function, "bytecode", "debug_symbols")
function_metadata_hash = sha256(json_serialize(function_metadata))
```

The artifact metadata stores all data that is not contained within the contract functions and is not debug specific. This includes the compiler version identifier, events interface, and name. Metadata is JSON-serialized in the same fashion as the function metadata.

```
artifact_metadata = omit(artifact, "functions", "file_map")
artifact_metadata_hash = sha256(json_serialize(artifact_metadata))
```

### Versioning

A contract class has an implicit `version` field that identifies the schema of the struct. This allows to change the shape of a contract class in future upgrades to the protocol to include new fields or change existing ones, while preserving the structure for existing classes. Supporting new types of contract classes would require introducing new kernel circuits, and a transaction proof may require switching between different kernel circuits depending on the version of the contract class used for each function call.

Note that the version field is not directly used when computing the contract class id, but is implicit in the generator index. Bumping the version of a contract class struct would involve using a different generator index for computing its id.

## Canonical Contract Class Registerer

A contract class is registered by calling a private `register` function in a canonical `ContractClassRegisterer` contract, which will emit a Registration Nullifier. The Registration Nullifier is defined as the `contract_class_id` itself of the class being registered. Note that the Private Kernel circuit will [silo](../circuits/private-kernel-tail.md#siloing-values) this value with the contract address of the `ContractClassRegisterer`, effectively storing the hash of the `contract_class_id` and `ContractClassRegisterer` address in the nullifier tree. As such, proving that a given contract class has been registered requires checking existence of this siloed nullifier.

The rationale for the Registerer contract is to guarantee that the public bytecode for a contract class is publicly available. This is a requirement for publicly [deploying a contract instance](./instances.md#publicly_deployed), which ultimately prevents a sequencer from executing a public function for which other nodes in the network may not have the code.

### Register Function

The `register` function receives the artifact hash, private functions tree root, and packed public bytecode of a `ContractClass` struct as [defined above](#structure), and performs the following steps:

- Assert that `packed_public_bytecode` is valid according to the definition in the [Public VM section](../public-vm/bytecode-validation-circuit.md#packed-bytecode-representation).
- Computes the `contract_class_id` as [defined above](#class-identifier).
- Emits the resulting `contract_class_id` as a nullifier to prevent the same class from being registered again.
- Emits an unencrypted event `ContractClassRegistered` with the contents of the contract class.

In pseudocode:

```
function register(
  artifact_hash: Field,
  private_functions_root: Field,
  packed_public_bytecode: Field[],
) 
  assert is_valid_packed_public_bytecode(packed_public_bytecode)

  version = 1
  bytecode_commitment = calculate_commitment(packed_public_bytecode)
  contract_class_id = pedersen([version, artifact_hash, private_functions_root, bytecode_commitment], GENERATOR__CLASS_IDENTIFIER)

  emit_nullifier contract_class_id
  emit_unencrypted_event ContractClassRegistered(contract_class_id, version, artifact_hash, private_functions_root, packed_public_bytecode)
```

Upon seeing a `ContractClassRegistered` event in a mined transaction, nodes are expected to store the contract class, so they can retrieve it when executing a public function for that class. Note that a class may be used for deploying a contract within the same transaction in which it is registered.

Note that emitting the `contract_class_id` as a nullifier (the `contract_class_id_nullifier`), instead of as an entry in the note hashes tree, allows nodes to prove non-existence of a class. This is needed so a sequencer can provably revert a transaction that includes a call to an unregistered class.

### Genesis

The `ContractClassRegisterer` will need to exist from the genesis of the Aztec Network, otherwise nothing will ever be publicly deployable to the network. The Class Nullifier for the `ContractClassRegisterer` contract will be pre-inserted into the genesis nullifier tree at leaf index `GENESIS_NULLIFIER_LEAF_INDEX_OF_CONTRACT_CLASS_REGISTERER_CLASS_ID_NULLIFIER=1`. The canonical instance will be deployed at `CONTRACT_CLASS_REGISTERER_ADDRESS=0x10000`, and its Deployment Nullifier will be inserted at `GENESIS_NULLIFIER_LEAF_INDEX_OF_CONTRACT_CLASS_REGISTERER_DEPLOYMENT_NULLIFIER=2`.

<!-- TODO: perhaps we need a page of constants? -->

<!-- TODO(cryptography): How do we convince the world that there's 'nothing up our sleeves'? What could be the consequences of a cunningly-chosen nullifier being pre-inserted into the nullifier tree? -->

### Broadcast

The `ContractClassRegisterer` has an additional private `broadcast` functions that can be used for broadcasting on-chain the bytecode, both ACIR and Brillig, for private functions and unconstrained in the contract. Any user can freely call this function. Given that ACIR and Brillig [do not have a circuit-friendly commitment](../bytecode/index.md), it is left up to nodes to perform this check. 

Broadcasted contract artifacts that do not match with their corresponding `artifact_hash`, or that reference a `contract_class_id` that has not been broadcasted, can be safely discarded.

```
function broadcast_all_private_functions(
  contract_class_id: Field,
  artifact_metadata: Field,
  unconstrained_functions_artifact_tree_root: Field,
  functions: { selector: Field, metadata: Field, vk_hash: Field, bytecode: Field[] }[],
)
  emit_unencrypted_event ClassPrivateFunctionsBroadcasted(
    contract_class_id,
    artifact_metadata,
    unconstrained_functions_artifact_tree_root,
    functions,
  )
```

```
function broadcast_all_unconstrained_functions(
  contract_class_id: Field,
  artifact_metadata: Field,
  private_functions_artifact_tree_root: Field,
  functions:{ selector: Field, metadata: Field, bytecode: Field[] }[],
)
  emit_unencrypted_event ClassUnconstrainedFunctionsBroadcasted(
    contract_class_id,
    artifact_metadata,
    unconstrained_functions_artifact_tree_root,
    functions,
  )
```

<!-- TODO: What representation of bytecode can we use here?  -->

The broadcast functions are split between private and unconstrained to allow for private bytecode to be broadcasted, which is valuable for composability purposes, without having to also include unconstrained functions, which could be costly to do due to data broadcasting costs. Additionally, note that each broadcast function must include enough information to reconstruct the `artifact_hash` from the Contract Class, so nodes can verify it against the one previously registered.

The `ContractClassRegisterer` contract also allows broadcasting individual functions, in case not every function needs to be put on-chain. This requires providing a Merkle membership proof for the function within its tree, that nodes can validate.

```
function broadcast_private_function(
  contract_class_id: Field,
  artifact_metadata: Field,
  unconstrained_functions_artifact_tree_root: Field,
  function_leaf_sibling_path: Field,
  function: { selector: Field, metadata: Field, vk_hash: Field, bytecode: Field[] },
)
  emit_unencrypted_event ClassPrivateFunctionBroadcasted(
    contract_class_id,
    artifact_metadata,
    unconstrained_functions_artifact_tree_root,
    function_leaf_sibling_path,
    function,
  )
```

```
function broadcast_unconstrained_function(
  contract_class_id: Field,
  artifact_metadata: Field,
  private_functions_artifact_tree_root: Field,
  function_leaf_sibling_path: Field,
  function: { selector: Field, metadata: Field, bytecode: Field[] }[],
)
  emit_unencrypted_event ClassUnconstrainedFunctionBroadcasted(
    contract_class_id,
    artifact_metadata,
    unconstrained_functions_artifact_tree_root,
    function_leaf_sibling_path: Field,
    function,
  )
```

It is strongly recommended for developers registering new classes to broadcast the code for `compute_hash_and_nullifier`, so any private message recipients have the code available to process their incoming notes. However, the `ContractClassRegisterer` contract does not enforce this during registration, since it is difficult to check the multiple signatures for `compute_hash_and_nullifier` as they may evolve over time to account for new note sizes.
