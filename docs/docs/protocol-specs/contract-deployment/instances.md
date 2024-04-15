# Contract instances

<!-- Consider adding a diagram which shows how all of this data interrelates. Similar to the outdated ones I drew. You know I love my diagrams. -->

A contract instance is a concrete deployment of a [contract class](./classes.md). A contract instance always references a contract class, which dictates what code it executes when called. A contract instance has state (both private and public), as well as an address that acts as its identifier. A contract instance can be called into.

## Requirements

- Users must be able to precompute the address of a given contract instance. This allows users to precompute their account contract addresses and receive funds before interacting with the chain, and also allows counterfactual deployments.
- An address must be linkable to its deployer address. This allows simple diversified and stealth account contracts. Related, a precomputed deployment may or may not be restricted to be executed by a given address.
- A user calling into an address must be able to prove that it has not been deployed. This allows the executor to prove that a given call in a transaction is unsatisfiable and revert accordingly.
- A user should be able to privately call into a contract without publicly deploying it. This allows private applications to deploy contracts without leaking information about their usage.

## `ContractInstance`

The structure of a contract instance is defined as:

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `version` | `u8` | Version identifier. Initially one, bumped for any changes to the contract instance struct. |
| `salt` | `Field` | User-generated pseudorandom value for uniqueness. |
| `deployer` | `AztecAddress` | Optional address of the deployer of the contract. |
| `contract_class_id` | `Field` | Identifier of the contract class for this instance. |
| `initialization_hash` | `Field` | Hash of the selector and arguments to the constructor. |
| `portal_contract_address` | `EthereumAddress` | Optional address of the L1 portal contract. |
| `public_keys_hash` | `Field` | Optional hash of the struct of public keys used for encryption and nullifying by this contract. |

<!-- TODO: define the `initialization_hash` derivation more explicitly. -->

<!-- Note: Always ensure the spec above matches the one described in Addresses and Keys. -->

### Versioning

Contract instances have a `version` field that identifies the schema of the instance, allowing for changes to the struct in future versions of the protocol, same as the contract class [version](./classes.md#versioning).

### Address

The address of the contract instance is computed as the hash of the elements in the structure above, as defined in [the addresses and keys section](../addresses-and-keys/address.md#address). This computation is deterministic, which allows any user to precompute the expected deployment address of their contract, including account contracts.

### Deployer

The `deployer` address of a contract instance is used to restrict who can initialize the contract (ie call its constructor) and who can publicly deploy it. Note that neither of these checks are enforced by the protocol: the initialization is checked by the constructor itself, and the deployment by the `ContractInstanceDeployer` (described below). Furthermore, a contract class may choose to not enforce this restriction by removing the check from the constructor.

The `deployer` address can be set to zero to signal that anyone can initialize or publicly deploy an instance.

## Initialization

A contract instance at a given address can be either Initialized or not. An address by default is not initialized, and it is considered to be Initialized once it emits an Initialization Nullifier, meaning it can only be initialized once.

### Uninitialized

The default state for any given address is to be uninitialized, meaning its constructor has not been called. A user who knows the preimage of the address can still issue a private call into a function in the contract, as long as that function does not assert that the contract has been initialized by checking the Initialization Nullifier.

All function calls to an Uninitialized contract that depend on the contract being initialized should fail, to prevent the contract from being used in an invalid state.

This state allows using a contract privately before it has been initialized or deployed, which is used in [diversified and stealth accounts](../addresses-and-keys/diversified-and-stealth.md).

### Initialized

An instance is Initialized when a constructor for the instance has been invoked, and the constructor has emitted the instance's Initialization Nullifier. All private functions that require the contract to be initialized by checking the existence of the Initialization Nullifier can now be called by any user who knows the address preimage.

The Initialization Nullifier is defined as the contract address itself. Note that the nullifier later gets [siloed by the Private Kernel Circuit](../circuits/private-kernel-tail.md#siloing-values) before it gets broadcasted in a transaction.

:::warning
It may be the case that it is not possible to read a nullifier in the same transaction that it was emitted due to protocol limitations. That would lead to a contract not being callable in the same transaction as it is initialized. To work around this, we can emit an Initialization Commitment along with the Initialization Nullifier, which _can_ be read in the same transaction as it is emitted. If needed, the Initialization Commitment is defined exactly as the Initialization Nullifier.
:::

### Constructors

Contract constructors are not enshrined in the protocol, but handled at the application circuit level. Constructors are methods used for initializing a contract, either private or public, and a contract may have more than a single constructor. A contract must ensure the following requirements are met:

- A contract may be initialized at most once
- A contract must be initialized using the method and arguments defined in its address preimage
- A contract must be initialized by its `deployer` (if it's non-zero)
- All functions that depend on contract initialization cannot be invoked until the contract is initialized

These checks are embedded in the application circuits themselves. The constructor emits an Initialization Nullifier when it is invoked, which prevents it from being called more than once. The constructor code must also check that its own selector and the arguments for the call match the ones in the address preimage, which are supplied via an oracle call.

All non-constructor functions in the contract should require a merkle membership proof for the Initialization Nullifier, to prevent them from being called before the constructor is invoked. Nevertheless, a contract may choose to allow some functions to be called before initialization, such as in the case of [Diversified and Stealth account contracts](../addresses-and-keys/diversified-and-stealth.md).

Removing constructors from the protocol itself simplifies the kernel circuit, and decoupling Initialization from Public Deployments allows users to keep contract instances private if they wish to do so.

## Public Deployment

A Contract Instance is considered to be Publicly Deployed when it has been broadcasted to the network via a canonical `ContractInstanceDeployer` contract, which also emits a Deployment Nullifier associated to the deployed instance.

All public function calls to an Undeployed address _must_ fail, since the Contract Class for it is not known to the network. If the Class is not known to the network, then an Aztec Node, whether it is the elected sequencer or a full node following the chain, may not be able to execute the bytecode for a public function call, which is undesirable.

The failing of public function calls to Undeployed addresses is enforced by having the Public Kernel Circuit check that the Deployment Nullifier for the instance has been emitted. Note that makes Public Deployment a protocol-level concern, whereas Initialization is purely an application-level concern. Also, note that this requires hardcoding the address of the `ContractInstanceDeployer` contract in a protocol circuit.

The Deployment Nullifier is defined as the address of the contract being deployed. Note that it later gets [siloed](../circuits/private-kernel-tail.md#siloing-values) using the `ContractInstanceDeployer` address by the Kernel Circuit, so this nullifier is effectively the hash of the deployed contract address and the `ContractInstanceDeployer` address.

### Canonical Contract Instance Deployer

A new contract instance can be _Publicly Deployed_ by calling a `deploy` function in a canonical `ContractInstanceDeployer` contract. This function receives the arguments for a `ContractInstance` struct as described [above](#contractinstance-structure):

- Validates the referenced `contract_class_id` exists. This can be done via either a call to the `ClassRegisterer` contract, or by directly reading the corresponding nullifier.
- Set `deployer` to zero or `msg_sender` depending on whether the `universal_deploy` flag is set.
- Computes the resulting `new_contract_address`.
- Emits the resulting address as the Deployment Nullifier to signal the public deployment, so callers can prove that the contract has or has not been publicly deployed.
- Emits an unencrypted event `ContractInstanceDeployed` with the address preimage.

The pseudocode for the process described above is the following:

<!-- Is `version` needed? If so, please update the address.md to include it -->

```rust
fn deploy (
  salt: Field,
  contract_class_id: Field,
  initialization_hash: Field,
  portal_contract_address: Field,
  public_keys_hash: Field,
  universal_deploy?: boolean,
)
  let contract_class_registerer: Contract = ContractClassRegisterer::at(CONTRACT_CLASS_REGISTERER_ADDRESS);

  assert(nullifier_exists(silo(contract_class_id, contract_class_registerer.address)));

  assert(is_valid_eth_address(portal_contract_address));

  let deployer: Address = if universal_deploy { 0 } else { msg_sender };
  let version: Field = 1;

  let address = address_crh(
    version,
    salt,
    deployer,
    contract_class_id,
    initialization_hash,
    portal_contract_address,
    public_keys_hash
  );

  emit_nullifier(address);

  emit_unencrypted_event(ContractInstanceDeployed::new(address, version, salt, contract_class_id, initialization_hash, portal_contract_address, public_keys_hash));
```

> See [address](../addresses-and-keys/address.md) for `address_crh`.

Upon seeing a `ContractInstanceDeployed` event from the canonical `ContractInstanceDeployer` contract, nodes are expected to store the address and preimage, so they can verify executed code during public code execution as described in the next section.

The `ContractInstanceDeployer` contract provides two implementations of the `deploy` function: a private and a public one.

### Genesis

The `ContractInstanceDeployer` will need to exist from the genesis of the Aztec Network, otherwise nothing will ever be deployable to the network. The Class Nullifier for the `ContractInstanceDeployer` contract will be pre-inserted into the genesis nullifier tree at leaf index [`GENESIS_NULLIFIER_LEAF_INDEX_OF_CONTRACT_INSTANCE_DEPLOYER_CLASS_ID_NULLIFIER`](../constants.md#genesis-constants). The canonical instance will be deployed at [`CONTRACT_INSTANCE_DEPLOYER_ADDRESS`](../constants.md#genesis-constants), and its Deployment Nullifier will be inserted at [`GENESIS_NULLIFIER_LEAF_INDEX_OF_CONTRACT_INSTANCE_DEPLOYER_DEPLOYMENT_NULLIFIER`](../constants.md#genesis-constants).

<!-- TODO(cryptography): How do we convince the world that there's 'nothing up our sleeves'? What could be the consequences of a cunningly-chosen nullifier being pre-inserted into the nullifier tree? -->

## Verification of Executed Code

The Kernel Circuit, both private and public, is responsible for verifying that the code loaded for a given function execution matches the expected one. This requires the following checks:

- The `contract_class_id` of the address called is the expected one, verified by hashing the address preimage that includes the `contract_class_id`.
- The [function selector](./classes.md#private-function) being executed is part of the `contract_class_id`, verified via a Merkle membership proof of the selector in the functions tree of the Contract Class.

Specific to private functions:

- The hash of the `verification_key` matches the `vk_hash` defined in the corresponding [Private Function](./classes.md#private-function) for the Contract Class. Note that the `verification_key` must include an identifier of the proving system used to compute it.

<!-- TODO: Define a format for encoding the proving system into the verification key or the vk_hash preimage. -->

Specific to public functions:

- The bytecode loaded by the [AVM](../public-vm/intro) for the contract matches the `bytecode_commitment` in the contract class, verified using the [bytecode validation circuit](../public-vm/bytecode-validation-circuit).
- The contract Deployment Nullifier has been emitted, or prove that it hasn't, in which case the transaction is expected to revert. This check is done via a merkle (non-)membership proof of the Deployment Nullifier. Note that a public function should be callable in the same transaction in which its contract Deployment Nullifier was emitted.

Note that, since constructors are handled at the application level, the kernel circuit is not required to check the Initialization Nullifier before executing code.

### Verifying Brillig in Private Functions

Private functions may have unconstrained code, inlined as Brillig bytecode. While unconstrained code, as it name implies, is not constrained within the protocol, a user PXE still needs a mechanism to verify that the code it has been delivered off-chain for a given function is correct.

This verification is done via the [contract class `artifact_hash`](./classes.md#structure), which contains a commitment to all bytecode in the contract. The PXE should receive the entire contract artifact, or at least the relevant sections to execute along with the commitments for the others to reconstruct the original `artifact_hash`, and verify that the resulting `artifact_hash` matches the one declared on-chain for the class of the contract being run.

## Discarded Approaches

### Contracts Tree

Earlier versions of the protocol relied on a dedicated contract tree, which required dedicated kernel code to process deployments, which had to be enshrined as new outputs from the application circuits. By abstracting contract deployment and storing deployments as nullifiers, the interface between the application and kernel circuits is simplified, and the kernel circuit has far fewer responsibilities. Furthermore, multiple contract deployments within a single transaction are now possible.

### Requiring initialization for Public Deployment

An earlier version of this draft required contracts to be Initialized in order to be Publicly Deployed. While this was useful for removing the initialization check in public functions, it caused a mix of concerns where the `ContractInstanceDeployer` needed to read a nullifier emitted from another contract. It also coupled the `ContractInstanceDeployer` to the convention decided for Initialization Nullifiers, and forced every contract to have a constructor in order to be publicly deployed even if they didn't need one. Furthermore, it required public constructors to be called via the `ContractInstanceDeployer` only.

Fully separating Initialization and Public Deployment leads to a cleaner `ContractInstanceDeployer`, and allows more flexibility to applications in handling their own initialization. The main downsides are that this opens the door for a contract to be simultaneously Publicly Deployed and Uninitialized, which is a state that does not seem to map to a valid use case. And it requires public functions to check the Initialization Nullifier on every call, which in the current approach is not needed as the presence of the Deployment Nullifier checked by the Public Kernel is enough of a guarantee that the contract was initialized.

### Execute Initialization during Public Deployment only

While it is appealing to allow a user to privately create a new contract instance and not reveal it to the world, we have not yet validated this use case. We could simplify deployment by relying on a single nullifier to track Initialization, and couple it with Public Deployment. Private functions can check initialization via the Deployment Nullifier emitted by the `ContractInstanceDeployer`.

This approach requires that constructors are only invoked as part of Public Deployment, so constructors would require an additional check for `msg_sender` being the canonical `ContractInstanceDeployer`. Furthermore, to ensure that an instance constructor is properly run, the `ContractInstanceDeployer` would need to know the selector for the instance constructor, which now needs to be part of the Contract Class, re-enshrining it into the protocol. Last, being able to keep agreements (contracts) private among their parties is commonplace in the traditional world, so there is a compelling argument for keeping this requirement.

Alternatively, we could remove constructor abstraction altogether, and have the Private Kernel Circuit check for the Deployment Nullifier, much like the Public Kernel Circuit does. However, this hurts Diversified and Stealth account contracts, which now require an explicit deployment and cannot be used directly.
