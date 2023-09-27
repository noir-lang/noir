# Common Errors

There are two kinds of errors: errors in an Aztec.nr contract, and errors spat out by an Aztec Sandbox node!
This section will provide an overview of the errors you might encounter, and how to fix them.

### Assertion Error

This error is thrown when a condition is not met.

This is what the error typically looks like:
```
Simulation error: Assertion failed: Balance too low 'sum == amount'
```

To address the error. find the line in the contract that is throwing the error and investigate why the condition is not met.

### Unknown Contract Error
This error occurs when you are trying to interact with a smart contract via an Private Execution Environment (PXE) that does not have the necessary information to execute a transaction.

This is what the error typically looks like:
```
Unknown contract 0x1d206be10b873b78b875259e1a8c39e2212e2f181d2fd0b0407446487deba522: add it to PXE by calling server.addContracts(...)
```

To execute a transaction, the PXE needs to know the complete address of a contract, portal address (if portal is used) and contract artifacts.

To address the error, add the contract to the PXE by calling `server.addContracts(...)`.

### Unknown Complete Address Error
This error occurs when your contract is trying to get a public key via the `get_public_key` oracle call, but the PXE does not have the Complete Address (Complete Address contains the public key).

This is what the error typically looks like:
```
Simulation error: Unknown complete address for address 0x0d179a5f9bd4505f7dfb8ca37d64e0bd0cd31b5cb018e252fd647bdf88959b95. Add the information to PXE by calling pxe.registerRecipient(...) or pxe.registerAccount(...)
```

Your contract typically needs a public key when it wants to send a note to a recipient because the public key is used to encrypt notes.

:::info
Manually adding the recipient to the PXE should not be required in case the recipient contract has already been deployed and the PXE is fully synced.
This is because this information is submitted on-chain when the recipient contract is deployed.
:::

### Unknown account
This error occurs when your contract is trying to get a secret via the `get_secret` oracle call, but the PXE does not have the secret for the public key.

This is what the error typically looks like:
```
Could not process note because of "Error: Unknown account.". Skipping note...
```

This error might occurr when you register an account only as a recipient and not as an account.
To address the error, register the account by calling `server.registerAccount(...)`.