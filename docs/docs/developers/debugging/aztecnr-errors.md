---
title: Aztec.nr Errors
---

This section contains some errors that you may encounter when writing and compiling contracts in Aztec.nr. If you run into an error that is not listed here, please [create an issue](https://github.com/AztecProtocol/aztec-packages/issues/new).

#### `Aztec dependency not found. Please add aztec as a dependency in your Nargo.toml`

All smart contracts written in Aztec.nr need the `aztec` dependency. In your `Nargo.toml` under `[dependencies]`, add this:

```toml
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
```

You can learn more about dependencies and their paths [here](../contracts/resources/dependencies.md).

#### `compute_note_hash_and_nullifier function not found. Define it in your contract`

Any smart contract that works with storage must include a [`compute_note_hash_and_nullifier`](https://github.com/AztecProtocol/aztec-packages/blob/6c20b45993ee9cbd319ab8351e2722e0c912f427/yarn-project/aztec-nr/aztec/src/note/utils.nr#L69) function to allow the PXE to process encrypted events.

This is an example of this function in the token contract:

#include_code compute_note_hash_and_nullifier yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

This error may also show if the `compute_note_hash_and_nullifier` function is not correct or sits outside of the contract.

#### `backend has encountered an error`

This is likely due to a version mismatch or bad install of barretenberg. Try [reinstalling nargo](../updating.md) or uninstalling barretenberg:

```bash
nargo backend uninstall acvm-backend-barretenberg
```

It will then reinstall when you compile.

#### `Oracle callback {} not found` & `Oracle callback pedersenHash not found`

This can occasionally happen when there are breaking releases. Make sure that your dependencies in `Nargo.toml` are [updated to the latest release](../contracts/resources/dependencies.md).

#### `error: Failed constraint: 'Public state writes only supported in public functions`

Reading and writing to public state from private functions is currently not supported.
This is because public values may change before the private function execution is posted on-chain.

This may change in future versions.

#### `Simulation error: Assertion failed:...`

This is an assertion error that is thrown when a condition is not met.

To address the error. find the line in the contract that is throwing the error and investigate why the condition is not met.

#### `Unknown contract 0x0: add it to PXE by calling server.addContracts(...)`

This error occurs when you are trying to interact with a smart contract via an Private Execution Environment (PXE) that does not have the necessary information to execute a transaction.

To execute a transaction, the PXE needs to know the complete address of a contract, portal address (if portal is used) and contract artifacts.

To address the error, add the contract to the PXE by calling [`pxe.addContracts(...)`](../../apis/pxe/interfaces/PXE.md#addcontracts).

#### `Simulation error: No public key registered for address 0x0. Register it by calling pxe.registerRecipient(...) or pxe.registerAccount(...)`

This error occurs when your contract is trying to get a public key via the `get_public_key` oracle call, but the PXE does not have the Complete Address (Complete Address contains the public key).

Your contract typically needs a note recipient's public key when it wants to send a note to because the public key is used to encrypt notes.

:::info
Manually adding the recipient to the PXE should not be required in case the recipient contract has already been deployed and the PXE is fully synced.
This is because this information is submitted on-chain when the recipient contract is deployed.
:::

#### `Could not process note because of "Error: Unknown account.". Skipping note...`

This error occurs when your contract is trying to get a secret via the `get_secret` oracle call, but the PXE does not have the secret for the public key.

This error might occur when you register an account only as a recipient and not as an account.
To address the error, register the account by calling `server.registerAccount(...)`.

#### `Failed to solve brillig function, reason: explicit trap hit in brillig 'self._is_some`

You may encounter this error when trying to send a transaction that is using an invalid contract. The contract may compile without errors and you only encounter this when sending the transaction.

This error may arise when function parameters are not properly formatted, when trying to "double-spend" a note, or it may indicate that there is a bug deeper in the stack (e.g. a bug in the Aztec.nr library or deeper). If you hit this error, double check your contract implementation, but also consider [opening an issue](https://github.com/AztecProtocol/aztec-packages/issues/new).
