# Contract instances

A contract instance is a concrete deployment of a [contract class](./classes.md). A contract instance has state, both private and public, as well as an address that acts as identifier, and can be called into. A contract instance always references a contract class, that dictates what code it executes when called.

## Requirements

- Users must be able to precompute the address of a given deployment. This allows users to precompute their account contract addresses and receive funds before interacting with the chain, and also allows counterfactual deployments.
- An address must be linked to its deployer address. This allows simple diversified and stealth account contracts. Related, a precomputed deployment may or may not be restricted to be executed by a given address.
- A user calling into an address must be able to prove that it has not been deployed. This allows the executor to prove that a given call in a transaction is unsatisfiable and revert accordingly.
- A user should be able to privately call into a contract without publicly deploying it. This allows private applications to deploy contracts without leaking information about their usage.

## Structure

The structure of a contract instance is defined as:

| Field | Type | Description |
|----------|----------|----------|
| version | u8 | Version identifier. Initially one, bumped for any changes to the contract instance struct. |
| deployer_address | AztecAddress | Address of the canonical deployer contract that creates this instance. |
| salt | Field | User-generated pseudorandom value for uniqueness. |
| contract_class_id | Field | Identifier of the contract class for this instance. |
| contract_args_hash | Field | Hash of the arguments to the constructor. |
| portal_contract_address | EthereumAddress | Optional address of the L1 portal contract. |
| public_keys_hash | Field | Optional hash of the struct of public keys used for encryption and nullifying by this contract. |

<!-- TODO: Ensure the spec above matches the one described in Addresses and Keys. -->

## Address

The address of the contract instance is computed as the hash of all elements in the structure above, as defined in the addresses and keys section. This computation is deterministic, which allows any user to precompute the expected deployment address of their contract, including account contracts.

## Statuses

A contract instance at a given address can be in any of the following statuses:
- **Undeployed**: The instance has not yet been deployed. A user who knows the preimage of the address can issue a private call into the contract, as long as it does not require initialization. Public function calls to this address will fail.
- **Privately deployed**: The instance constructor has been executed, but its class identifier has not been broadcasted. A user who knows the preimage of the address can issue a private call into the contract. Public function calls to this address will fail. Private deployments are signalled by emitting an initialization nullifier when the constructor runs.
- **Publicly deployed**: The instance constructor has been executed, and the address preimage has been broadcasted. All function calls to the address, private or public, are valid. Public deployments are signalled by emitting a public deployment nullifier.

<!-- TODO: Validate the need for private deployments. They seem cool, and do not incur in extra protocol complexity, but they require having two nullifiers per contract: one to signal initialization, and one to signal broadcasting of the deployment. -->

## Constructors

Contract constructors are not enshrined in the protocol, but handled at the application circuit level. A contract must satisfy the following requirements:
- The constructor must be invoked exactly once
- The constructor must be invoked with the arguments in the address preimage
- Functions that depend on contract initialization cannot be invoked until the constructor is run

These checks can be embedded in the application circuit itself. The constructor emits a standardized initialization nullifier when it is invoked, which prevents it from being called more than once. The constructor code must also check that the arguments hash it received matches the ones in the address preimage, supplied via an oracle call. All other functions in the contract must include a merkle membership proof for the nullifier, to prevent them from being called before the constructor is invoked. Note that a contract may choose to allow some functions to be called before initialization.

The checks listed above should not be manually implemented by a contract developer, but rather included as part of the Aztec macros for Noir.

Constructors may be either private or public functions. Contracts with private constructors can be privately or publicly deployed, contracts with public constructors can only be publicly deployed.

Removing constructors from the protocol itself simplifies the kernel circuit. Separating initialization from public deployment also allows to implement private deployments, since a private deployment is equivalent to just invoking the constructor function at a given address.

## Public Deployment

A new contract instance can be _publicly deployed_ by calling a `deploy` function in a canonical `Deployer` contract. This function receives the arguments for a `ContractInstance` struct as described above, except for the `deployer_address` which is the deployer's own address, and executes the following actions:

- Validates the referenced `contract_class_id` exists.
- Mixes in the `msg_sender` with the user-provided `salt` by hashing them together, ensuring that the deployment is unique for the requester.
- Computes the resulting contract address.
- Emits the resulting address as a nullifier to signal the public deployment, so callers can prove that the contract has or has not been publicly deployed.
- Emits an unencrypted event with the address preimage, excluding the `deployer_address` which is already part of the log.
- Either proves the corresponding class identifier has no constructor, or calls the constructor with the preimage of the supplied `contract_args_hash`, or proves that the constructor nullifier has already been emitted so a previously privately-deployed contract can be publicly deployed.

Upon seeing a new contract deployed event from the canonical deployer contract, nodes are expected to store the address and preimage, to verify executed code during public code execution as described in the next section.

The `Deployer` contract provides two implementations of the `deploy` function: a private and a public one. Contracts with a private constructor are expected to use the former, and contracts with public constructors expected to use the latter. Contracts with no constructors or that have already been privately-deployed can use either.

Additionally, the `Deployer` contract provides two `universal_deploy` functions, a private and a public one, with the same arguments as the `deploy` one, that just forwards the call to the `deploy` contract. This makes `msg_sender` in the `deploy` contract to be the `Deployer` contract itself, and allows for universal deployments that are semantically decoupled from their deployer, and can be permissionlessly invoked by any user who knows the address preimage.

## Verification of Executed Code

The kernel circuit, both private and public, is responsible for verifying that the code loaded for a given function execution matches the expected one. This requires the following checks:
- The contract class identifier of the address called is the expected one, verified by hashing the address preimage that includes the class id.
- The function identifier being executed is part of the class id, verified via a merkle membership proof.
- The function code executed matches the commitment in the function identifier, verified via a merkle membership proof and a bytecode commitment proof.

Note that, since constructors are handled at the application level, the kernel circuit is not required to check that a constructor has been run before executing code.

The public kernel circuit, additionally, needs to check that the corresponding contract instance has been publicly deployed, or prove that it hasn't. This is done via a merkle (non-)membership proof of the public deployment nullifier.

## Discarded Approaches

Earlier versions of the protocol relied on a dedicated contract tree, which required dedicated kernel code to process deployments, which had to be enshrined as new outputs from the application circuits. By abstracting contract deployment and storing deployments as nullifiers, the interface between the application and kernel circuits is simplified, and the kernel circuit has fewer responsibilities.
