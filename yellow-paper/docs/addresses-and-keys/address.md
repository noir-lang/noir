---
title: Address
---

An address is computed as the hash of the following fields:

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `salt` | `Field` | User-generated pseudorandom value for uniqueness. |
| `deployer` | `AztecAddress` | Optional address of the deployer of the contract. |
| `contract_class_id` | `Field` | Identifier of the contract class for this instance. |
| `initialization_hash` | `Field` | Hash of the selector and arguments to the constructor. |
| `portal_contract_address` | `EthereumAddress` | Address of the L1 portal contract, zero if none. |
| `public_keys_hash` | `Field` | Hash of the struct of public keys used for encryption and nullifying by this contract, zero if no public keys. |

Storing these fields in the address preimage allows any part of the protocol to check them by recomputing the hash and verifying that the address matches. Examples of these checks are:

- Sending an encrypted note to an undeployed account, which requires the sender app to check the recipient's public key given their address. This scenario also requires the recipient to share with the sender their public key and rest of preimage.
- Having the kernel circuit verify that the code executed at a given address matches the one from the class.
- Asserting that the initialization hash matches the function call in the contract constructor.
- Checking the portal contract address when sending a cross-chain message.

:::warning
We may remove the `portal_contract_address` as a first-class citizen.
:::

The hashing scheme for the address should then ensure that checks that are more frequent can be done cheaply, and that data shared out of band is kept manageable. We define the hash to be computed as follows:

```
salted_initialization_hash = pedersen([salt, initialization_hash, deployer as Field, portal_contract_address as Field], GENERATOR__SALTED_INITIALIZATION_HASH)
partial_address = pedersen([contract_class_id, salted_initialization_hash], GENERATOR__CONTRACT_PARTIAL_ADDRESS_V1)
address = pedersen([public_keys_hash, partial_address], GENERATOR__CONTRACT_ADDRESS_V1)
```

The `public_keys` array can vary depending on the format of keys used by the address, but it is suggested it includes the master keys defined in the [keys section](./keys.md).

```
public_keys_hash = pedersen([
  nullifier_pubkey.x, nullifier_pubkey.y,
  tagging_pubkey.x, tagging_pubkey.y,
  incoming_view_pubkey.x, incoming_view_pubkey.y,
  outgoing_view_pubkey.x, outgoing_view_pubkey.y
], GENERATOR__PUBLIC_KEYS)
```

This recommended hash format is compatible with the [encryption precompiles](./precompiles.md#encryption-and-tagging-precompiles) initially defined in the protocol and advertised in the canonical [registry](../pre-compiled-contracts/registry.md) for private message delivery. An address that chooses to use a different format for its keys will not be compatible with apps that rely on the registry for note encryption. Nevertheless, new precompiles introduced in future versions of the protocol could use different public keys formats.

<!-- TODO(cryptography): Can we restrict "x" components of public keys to all be the same sign, so we don't need to encode "y"'s signs? -->
