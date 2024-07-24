---
title: CLI Reference
---

:::warning

`aztec-builder` and `aztec-sandbox` have been deprecated in favor of `aztec` CLI

:::
- [Start](#starting-and-testing)
- [Accounts](#account-management)
- [Contract deployments and interaction](#contract-deployment-and-interaction)
- [Network and node info](#network-and-node-information)
- [Querying](#transaction-and-block-querying)
- [Logging](#logging-and-data-retrieval)
- [Debugging](#development-and-debugging-tools)
- [L1 contracts](#l1-contract-management)
- [Utils](#utility-commands)

## Starting and testing

### start

Initiates various Aztec modules. It can be used to start individual components or the entire Aztec Sandbox.

```
aztec start [options]
```

Options:
- `-sb, --sandbox`: Starts the Aztec Sandbox.
- `-p, --port <port>`: Specifies the port to run Aztec on (default: 8080).
- `-n, --node [options]`: Starts the Aztec Node with specified options.
- `-px, --pxe [options]`: Starts the PXE (Private eXecution Environment) with specified options.
- `-a, --archiver [options]`: Starts the Archiver with specified options.
- `-s, --sequencer [options]`: Starts the Sequencer with specified options.
- `-r, --prover [options]`: Starts the Prover Agent with specified options.
- `-o, --prover-node [options]`: Starts the Prover Node with specified options.
- `-p2p, --p2p-bootstrap [options]`: Starts the P2P Bootstrap node with specified options.
- `-t, --txe [options]`: Starts the TXE (Transaction Execution Environment) with specified options.

### test

Runs tests written in contracts.

```
aztec test [options]
```

Options:
- `--workdir <path>`: Sets the working directory inside the container (default: current directory).
- `-e, --env <key=value>`: Set environment variables (can be used multiple times).
- `--no-tty`: Run the container without a TTY.
- `--rm`: Automatically remove the container when it exits.
- `-i, --interactive`: Keep STDIN open even if not attached.
- `-t, --tty`: Allocate a pseudo-TTY.

## Account Management

### create-account
Creates an Aztec account for sending transactions.

```
aztec create-account [options]
```

Options:
- `--skip-initialization`: Skip initializing the account contract.
- `--public-deploy`: Publicly deploys the account and registers the class if needed.
- `--private-key <key>`: Private key for the account (uses random by default).
- `--register-only`: Just register the account on the PXE without deploying.
- `--no-wait`: Skip waiting for the contract deployment.

### get-accounts
Retrieves all Aztec accounts stored in the PXE.

```
aztec get-accounts [options]
```

Options:
- `--json`: Emit output as JSON.

### get-account
Retrieves an account given its Aztec address.

```
aztec get-account <address> [options]
```

### register-recipient
Registers a recipient in the PXE.

```
aztec register-recipient [options]
```

Required options:
- `-a, --address <aztecAddress>`: The account's Aztec address.
- `-p, --public-key <publicKey>`: The account public key.
- `-pa, --partial-address <partialAddress>`: The partially computed address of the account contract.

## Contract Deployment and Interaction

### deploy
Deploys a compiled Aztec.nr contract to Aztec.

```
aztec deploy <artifact> [options]
```

Options:
- `--init <string>`: The contract initializer function to call (default: "constructor").
- `--no-init`: Leave the contract uninitialized.
- `-a, --args <constructorArgs...>`: Contract constructor arguments.
- `-k, --public-key <string>`: Optional encryption public key for this address.
- `-s, --salt <hex string>`: Optional deployment salt for generating the deployment address.
- `--universal`: Do not mix the sender address into the deployment.
- `--json`: Emit output as JSON.
- `--no-wait`: Skip waiting for the contract deployment.
- `--no-class-registration`: Don't register this contract class.
- `--no-public-deployment`: Don't emit this contract's public bytecode.

### send
Calls a function on an Aztec contract.

```
aztec send <functionName> [options]
```

Options:
- `-a, --args [functionArgs...]`: Function arguments.
- `-c, --contract-artifact <fileLocation>`: Compiled Aztec.nr contract's ABI.
- `-ca, --contract-address <address>`: Aztec address of the contract.
- `--no-wait`: Print transaction hash without waiting for it to be mined.

### call
Simulates the execution of a view (read-only) function on a deployed contract.

```
aztec call <functionName> [options]
```

Options:
- `-a, --args [functionArgs...]`: Function arguments.
- `-c, --contract-artifact <fileLocation>`: Compiled Aztec.nr contract's ABI.
- `-ca, --contract-address <address>`: Aztec address of the contract.
- `-f, --from <string>`: Aztec address of the caller.

### add-contract
Adds an existing contract to the PXE.

```
aztec add-contract [options]
```

Required options:
- `-c, --contract-artifact <fileLocation>`: Compiled Aztec.nr contract's ABI.
- `-ca, --contract-address <address>`: Aztec address of the contract.
- `--init-hash <init hash>`: Initialization hash.

Optional:
- `--salt <salt>`: Optional deployment salt.
- `-p, --public-key <public key>`: Optional public key for this contract.
- `--portal-address <address>`: Optional address to a portal contract on L1.
- `--deployer-address <address>`: Optional address of the contract deployer.

## Network and Node Information

### get-node-info
Retrieves information about an Aztec node at a URL.

```
aztec get-node-info [options]
```

### get-pxe-info
Retrieves information about a PXE at a URL.

```
aztec get-pxe-info [options]
```

### block-number
Retrieves the current Aztec L2 block number.

```
aztec block-number [options]
```

## Transaction and Block Querying

### get-tx
Retrieves the receipt for a specified transaction hash.

```
aztec get-tx <txHash> [options]
```

### get-block
Retrieves information for a given block or the latest block.

```
aztec get-block [blockNumber] [options]
```

Options:
- `-f, --follow`: Keep polling for new blocks.

## Logging and Data Retrieval

### get-logs
Retrieves unencrypted logs based on filter parameters.

```
aztec get-logs [options]
```

Options:
- `-tx, --tx-hash <txHash>`: Transaction hash to get the receipt for.
- `-fb, --from-block <blockNum>`: Initial block number for getting logs.
- `-tb, --to-block <blockNum>`: Up to which block to fetch logs.
- `-al --after-log <logId>`: ID of a log after which to fetch the logs.
- `-ca, --contract-address <address>`: Contract address to filter logs by.
- `--follow`: Keep polling for new logs until interrupted.

### add-note
Adds a note to the database in the PXE.

```
aztec add-note <address> <contractAddress> <storageSlot> <noteTypeId> <txHash> [options]
```

Required option:
- `-n, --note [note...]`: The members of a Note serialized as hex strings.

## Development and Debugging Tools

### codegen
Validates and generates an Aztec Contract ABI from Noir ABI.

```
aztec codegen <noir-abi-path> [options]
```

Options:
- `-o, --outdir <path>`: Output folder for the generated code.
- `--force`: Force code generation even when the contract has not changed.

### update
Updates Nodejs and Noir dependencies.

```
aztec update [projectPath] [options]
```

Options:
- `--contract [paths...]`: Paths to contracts to update dependencies.
- `--aztec-version <semver>`: The version to update Aztec packages to (default: latest).

### inspect-contract
Shows a list of external callable functions for a contract.

```
aztec inspect-contract <contractArtifactFile>
```

### parse-parameter-struct
Helper for parsing an encoded string into a contract's parameter struct.

```
aztec parse-parameter-struct <encodedString> [options]
```

Required options:
- `-c, --contract-artifact <fileLocation>`: Compiled Aztec.nr contract's ABI.
- `-p, --parameter <parameterName>`: The name of the struct parameter to decode into.

## L1 Contract Management

### deploy-l1-contracts
Deploys all necessary Ethereum contracts for Aztec.

```
aztec deploy-l1-contracts [options]
```

Required options:
- `-u, --rpc-url <string>`: URL of the Ethereum host.
- `-pk, --private-key <string>`: The private key to use for deployment.

### deploy-l1-verifier
Deploys the rollup verifier contract.

```
aztec deploy-l1-verifier [options]
```

Required options:
- `--eth-rpc-url <string>`: URL of the Ethereum host.
- `-pk, --private-key <string>`: The private key to use for deployment.
- `--verifier <verifier>`: Either 'mock' or 'real'.

### bridge-l1-gas
Mints L1 gas tokens and pushes them to L2.

```
aztec bridge-l1-gas <amount> <recipient> [options]
```

Required option:
- `--l1-rpc-url <string>`: URL of the Ethereum host.

### get-l1-balance
Gets the balance of gas tokens in L1 for a given Ethereum address.

```
aztec get-l1-balance <who> [options]
```

Required option:
- `--l1-rpc-url <string>`: URL of the Ethereum host.

## Utility Commands

### generate-keys
Generates encryption and signing private keys.

```
aztec generate-keys [options]
```

Option:
- `-m, --mnemonic`: Optional mnemonic string for private key generation.

### generate-p2p-private-key
Generates a LibP2P peer private key.

```
aztec generate-p2p-private-key
```

### example-contracts
Lists the example contracts available to deploy from @aztec/noir-contracts.js.

```
aztec example-contracts
```

### compute-selector
Computes a selector for a given function signature.

```
aztec compute-selector <functionSignature>
```

### bootstrap
Bootstraps the blockchain.

```
aztec bootstrap [options]
```

### sequencers
Manages or queries registered sequencers on the L1 rollup contract.

```
aztec sequencers <command> [who] [options]
```

Commands: list, add, remove, who-next

Required option:
- `--l1-rpc-url <string>`: URL of the Ethereum host.

Note: Most commands accept a `--rpc-url` option to specify the Aztec node URL, and many accept fee-related options for gas limit and price configuration.