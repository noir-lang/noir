# Aztec CLI Documentation

The Aztec CLI is a command-line interface for interacting with Aztec. It provides various commands to perform different tasks related to Aztec contracts and accounts. This documentation provides an overview of the available commands and their usage.

## Installation

To use the Aztec CLI, you need to have Node.js installed on your system. You can install it from the official Node.js website: [https://nodejs.org](https://nodejs.org)

After installing Node.js, you can install the Aztec CLI globally using the following command:

```shell
npm install -g @aztec/azti
```

## Usage

Once the Aztec CLI is installed, you can run it using the `azti` command followed by the desired command and its arguments. Here's the basic syntax:

```shell
azti [command] [arguments] [options]
```

To get help about the available commands and their usage, you can use the `--help` option:

```shell
azti --help
```

## Commands

### deploy-l1-contracts

Deploy Aztec contracts on Layer 1.

Syntax:

```shell
azti deploy-l1-contracts [rpcUrl] [options]
```

- `rpcUrl` (optional): URL of the Ethereum host. Chain identifiers `localhost` and `testnet` can be used. Default: `http://localhost:8545`.

Options:

- `-a, --api-key <string>`: API key for the Ethereum host.
- `-p, --private-key <string>`: The private key to use for deployment.
- `-m, --mnemonic <string>`: The mnemonic to use in deployment. Default: "test test test test test test test test test test test junk".

### deploy

Deploy an Aztec contract.

Syntax:

```shell
azti deploy <contractAbi> [options]
```

- `contractAbi`: Path to the compiled Noir contract's ABI file in JSON format.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.
- `-k, --public-key <string>`: Public key to use for deployment.
- `-a, --constructor-args [args...]`: Constructor arguments for the contract.

### check-deploy

Check if a contract has been deployed to an Aztec address.

Syntax:

```shell
azti check-deploy <contractAddress> [options]
```

- `contractAddress`: Aztec address to check if the contract has been deployed to.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

### get-tx-receipt

Get the receipt for a transaction hash.

Syntax:

```shell
azti get-tx-receipt <txHash> [options]
```

- `txHash`: Transaction hash to get the receipt for.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

### get-contract-data

Get data about an Aztec contract.

Syntax:

```shell
azti get-contract-data <contractAddress> [options]
```

- `contractAddress`: Aztec address of the contract.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.
- `-b, --include-bytecode`: Include the contract's public function bytecode, if any.

### create-account

Create a new Aztec account.

Syntax:

```shell
azti create-account [options]
```

Options:

- `-k, --private-key`: Private Key to use for the 1st account generation.
- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.
- `-n, --num-addresses <number>`: Number of accounts to create. Default: 1.

### get-accounts

Get a list of Aztec accounts.

Syntax:

```shell
azti get-accounts [options]
```

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

### get-account-public-key

Get the public key for an Aztec account.

Syntax:

```shell
azti get-account-public-key <address> [options]
```

- `address`: Aztec address to get the public key for.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

### call-fn

Call a function on an Aztec contract.

Syntax:

```shell
azti call-fn <contractAbi> <contractAddress> <functionName> [from] [functionArgs...] [options]
```

- `contractAbi`: Path to the compiled contract's ABI file in JSON format.
- `contractAddress`: Address of the contract.
- `functionName`: Name of the function to call.
- `from` (optional): Caller of the transaction.
- `functionArgs` (optional): Function arguments.

Options:

- `-u, --rpcUrl <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

### view-tx

Simulate the execution of a view (read-only) function on a deployed contract without actually modifying state.

Syntax:

```shell
azti view-tx <contractAbi> <contractAddress> <functionName> [from] [functionArgs...] [options]
```

- `contractAbi`: Path to the compiled contract's ABI file in JSON format.
- `contractAddress`: Address of the contract.
- `functionName`: Name of the function to call.
- `from` (optional): Caller of the transaction.
- `functionArgs` (optional): Function arguments.

Options:

- `-u, --rpcUrl <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

### get-logs

Get logs from Aztec blocks.

Syntax:

```shell
azti get-logs <from> <take> [options]
```

- `

from`: Block number to start fetching logs from.

- `take`: Number of block logs to fetch.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.
