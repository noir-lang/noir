# Aztec CLI Documentation

The Aztec CLI `aztec-cli` is a command-line interface (CLI) tool for interacting with Aztec. It provides various commands for deploying contracts, creating accounts, interacting with contracts, and retrieving blockchain data.

## Installation

To use `aztec-cli`, you need to have Node.js installed on your system. Follow these steps to install and set up the CLI tool:

1. Install Node.js: Visit the official Node.js website (https://nodejs.org) and download the installer for your operating system. Follow the installation instructions to install Node.js.

2. Install `aztec-cli` package: Open a terminal or command prompt and run the following command to install `aztec-cli` globally on your system:

   ```shell
   npm install -g @aztec/cli
   ```

   This will install the `aztec-cli` globally, making it accessible from any location in your terminal.

3. Verify the installation: After the installation is complete, run the following command to verify that `aztec-cli` is installed correctly:

   ```shell
   aztec-cli --version
   ```

   This command will display the version number of `aztec-cli` if the installation was successful.

## Usage

To use `aztec-cli`, open a terminal or command prompt and run the `aztec-cli` command followed by the desired command and its options.

Here's the basic syntax for running a command:

```shell
aztec-cli <command> [options]
```

Replace `<command>` with the actual command you want to execute and `[options]` with any optional flags or parameters required by the command.

### Environment Variables

Some options can be set globally as environment variables to avoid having to re-enter them every time you call `aztec-cli.`
These options are:

- `PRIVATE_KEY` -> `-k, --private-key` for all commands that require a private key.
- `PUBLIC_KEY` -> `-k, --public-key` for all commands that require a public key.
- `AZTEC_RPC_HOST` -> `-u, --rpc-url` for commands that require an Aztec RPC URL.
- `API_KEY` -> `a, --api-key` for `deploy-l1-contracts`.
- `ETHEREUM_RPC_HOST` -> `-u, --rpc-url` for `deploy-l1-contracts`.

So if for example you are running your Aztec RPC server remotely you can do:

```shell
export AZTEC_RPC_HOST=http://external.site/rpc:8080
aztec-cli deploy my_contract.json
```

And this will send the request to `http://external.site/rpc:8080`.

**NOTE**: Entering an option value will override the environment variable.

## Available Commands

`aztec-cli` provides the following commands for interacting with Aztec:

### deploy-l1-contracts

Deploys all necessary Ethereum contracts for Aztec.

Syntax:

```shell
aztec-cli deploy-l1-contracts [rpcUrl] [options]
```

- `rpcUrl` (optional): URL of the Ethereum host. Chain identifiers `localhost` and `testnet` can be used. Default: `http://localhost:8545`.

Options:

- `-a, --api-key <string>`: API key for the Ethereum host.
- `-p, --private-key <string>`: The private key to use for deployment.
- `-m, --mnemonic <string>`: The mnemonic to use in deployment. Default: `test test test test test test test test test test test junk`.

This command deploys all the necessary Ethereum contracts required for Aztec. It creates the rollup contract, registry contract, inbox contract, outbox contract, and contract deployment emitter. The command displays the addresses of the deployed contracts.

Example usage:

```shell
aztec-cli deploy-l1-contracts
```

### create-private-key

Generates a 32-byte private key.

Syntax:

```shell
aztec-cli create-private-key [options]
```

Options:

- `-m, --mnemonic`: A mnemonic string that can be used for the private key generation.

This command generates a random 32-byte private key or derives one from the provided mnemonic string. It displays the generated private key.

Example usage:

```shell
aztec-cli create-private-key
```

### create-account

Creates an Aztec account that can be used for transactions.

Syntax:

```shell
aztec-cli create-account [options]
```

Options:

- `-k, --private-key`: Private key to use for the account generation. Uses a random key by default.
- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command creates an Aztec account that can be used for transactions. It generates a new account with a private key or uses the provided private key. The command displays the account's address and public key.

Example usage:

```shell
aztec-cli create-account
```

### deploy

Deploys a compiled Noir contract to Aztec.

Syntax:

```shell
aztec-cli deploy <contractAbi> [options]
```

Options:

- `-c, --contract-abi <fileLocation>`: Path to the compiled Noir contract's ABI file in JSON format. You can also use one of Aztec's example contracts found in [@aztec/noir-contracts](https://www.npmjs.com/package/@aztec/noir-contracts), e.g. PrivateTokenContractAbi. You can get a full ist of the available contracts with `aztec-cli example-contracts`
- `-a, --args <constructorArgs...>` (optional): Contract constructor arguments Default: [].
- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.
- `-k, --public-key <string>`: Public key of the deployer. If not provided, it will check the RPC for existing ones.

This command deploys a compiled Noir contract to Aztec. It requires the path to the contract's ABI file in JSON format. Optionally, you can specify the public key of the deployer and provide constructor arguments for the contract. The command displays the address of the deployed contract.

Example usage:

```shell
aztec-cli deploy -c path/to/contract.abi.json -a ...args
```

With an Aztec example contract:

```shell
aztec-cli deploy -c PrivateTokenContractAbi -a 333 0x134567890abcdef
```

### check-deploy

Checks if a contract is deployed to the specified Aztec address.

Syntax:

```shell
aztec-cli check-deploy <contractAddress> [options]
```

Options:

- `-ca, --contract-address <address>`: An Aztec address to check if the contract has been deployed to.
- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command checks if a contract is deployed to the specified Aztec address. It verifies if the contract is present at the given address and displays the result.

Example usage:

```shell
aztec-cli check-deploy -ca 0x123456789abcdef123456789abcdef12345678
```

### get-tx-receipt

Gets the receipt for the specified transaction hash.

Syntax:

```shell
aztec-cli get-tx-receipt <txHash> [options]
```

- `txHash`: A transaction hash to get the receipt for.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command retrieves and displays the receipt for the specified transaction hash. It shows details such as the transaction status, block number, and block hash.

Example usage:

```shell
aztec-cli get-tx-receipt 0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678
```

### get-contract-data

Gets information about the Aztec contract deployed at the specified address.

Syntax:

```shell
aztec-cli get-contract-data <contractAddress> [options]
```

- `contractAddress`: Aztec address of the contract.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.
- `-b, --include-bytecode`: Include the contract's public function bytecode, if any.

This command retrieves and displays information about the Aztec contract deployed at the specified address. It shows the contract address, portal contract address, and optionally, the bytecode of the contract's public functions.

Example usage:

```shell
aztec-cli get-contract-data 0x123456789abcdef123456789abcdef12345678
```

### register-public-key

Register an account's public key to the RPC server.
To read about how keys are generated and used, head to our docs [here](https://github.com/AztecProtocol/aztec-packages/blob/master/docs/docs/aztec/developer/wallet-providers/keys.md#addresses-partial-addresses-and-public-keys).

Syntax:

```shell
aztec-cli register-public-key [options]
```

Options:

- `-a, --address <aztecAddress>`: The account's Aztec address.
- `-p, --public-key <publicKey>`: 'The account public key.'
- `-pa, --partial-address <partialAddress`: 'The partially computed address of the account contract.'
- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

Example usage:

```shell
aztec-cli register-public-key -p 0x20d9d93c4a9eb2b4bdb70ead07d28d1edb74bfd78443a8c36b098b024cd26f0e0647f5dbe3619453f42eb788c2beed0294c84676425047aadac23294605c4af9 -a 0x111fdc0f6bf831ca59f05863199762d643b782699d7ce6feaae40a923baf60af -pa 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed
```

### get-accounts

Gets all the Aztec accounts stored in an Aztec RPC.

Syntax:

```shell
aztec-cli get-accounts [options]
```

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command retrieves and displays all the Aztec accounts available in the system.

Example usage:

```shell
aztec-cli get-accounts
```

### get-account-public-key

Gets an account's public key, given its Aztec address.

Syntax:

```shell
aztec-cli get-account-public-key <address> [options]
```

- `address`: The Aztec address to get the public key for.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command retrieves and displays the public key of an account given its Aztec address.

Example usage:

```shell
aztec-cli get-account-public-key 0x123456789abcdef123456789abcdef12345678
```

### send

Calls a function on an Aztec contract.

Syntax:

```shell
aztec-cli call-fn <contractAbi> <contractAddress> <functionName> [functionArgs...] [options]
```

- `functionName`: Name of the function to call.

Options:

- `'-a, --args [functionArgs...]` (optional): Function arguments. Default: [].
- `-c, --contract-abi <fileLocation>`: The compiled contract's ABI in JSON format. You can also use one of Aztec's example contracts found in (@aztec/noir-contracts)[https://www.npmjs.com/package/@aztec/noir-contracts], e.g. PrivateTokenContractAbi.
- `-ca, --contract-address <address>`: Address of the contract.
- `-k, --private-key <string>`: The sender's private key.
- `-u, --rpcUrl <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command calls a function on an Aztec contract. It requires the contract's ABI, address, function name, and optionally, function arguments. The command executes the function call and displays the transaction details.

Example usage:

```shell
aztec-cli send transfer -ca 0x123456789abcdef123456789abcdef12345678 -a 100 -c path/to/abi.json
```

### view-fn

Simulates the execution of a view (read-only) function on a deployed contract, without modifying state.

Syntax:

```shell
aztec-cli call <contractAbi> <contractAddress> <functionName> [functionArgs...] [options]
```

- `functionName`: Name of the function to view.

Options:

- `'-a, --args [functionArgs...]` (optional): Function arguments. Default: [].
- `-c, --contract-abi <fileLocation>`: The compiled contract's ABI in JSON format. You can also use one of Aztec's example contracts found in (@aztec/noir-contracts)[https://www.npmjs.com/package/@aztec/noir-contracts], e.g. PrivateTokenContractAbi.
- `-ca, --contract-address <address>`: Address of the contract.
- `-f, --from <string>`: Public key of the transaction viewer. If empty, it will try to find an account in the RPC.
- `-u, --rpcUrl <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command simulates the execution of a view function on a deployed contract without modifying the state. It requires the contract's ABI, address, function name, and optionally, function arguments. The command displays the result of the view function.

Example usage:

```shell
aztec-cli call balanceOf -c path/to/contract.abi.json -ca 0x123456789abcdef123456789abcdef12345678 -a balanceOf 0xabcdef1234567890abcdef1234567890abcdef12
```

### parse-parameter-struct

Helper for parsing an encoded string into a contract's parameter struct.

Syntax:

```shell
aztec-cli parse-parameter-struct <encodedString> <contractAbi> <parameterName>
```

- `encodedString`: The encoded hex string.
- `contractAbi`: The compiled contract's ABI in JSON format.
- `parameterName`: The name of the struct parameter to decode into.

This command is a helper for parsing an encoded hex string into a contract's parameter struct. It requires the encoded string, the contract's ABI, and the name of the struct parameter. The command decodes the string and displays the struct data.

Example usage:

```shell
aztec-cli parse-parameter-struct 0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890 path/to/contract.abi.json paramName
```

### get-logs

Gets all the unencrypted logs from L2 blocks in the specified range.

Syntax:

```shell
aztec-cli get-logs <from> <limit> [options]
```

- `from`: Block number to start fetching logs from.
- `limit`: Maximum number of block logs to obtain.

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command retrieves and displays all the unencrypted logs from L2 blocks in the specified range. It shows the logs found in the blocks and unrolls them for readability.

Example usage:

```shell
aztec-cli get-logs 1000 10
```

### block-num

Gets the current Aztec L2 block number.

Syntax:

```shell
aztec-cli block-number
```

Options:

- `-u, --rpc-url <string>`: URL of the Aztec RPC. Default: `http://localhost:8080`.

This command retrieves and displays the current Aztec L2 block number.

### example-contracts

Lists the contracts available in [@aztec/noir-contracts](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts)

Syntax:

```shell
aztec-cli example-contracts
```

## Conclusion

That covers the available commands and their usage in the `aztec-cli`. You can now use these commands to interact with Aztec and perform various actions such as deploying contracts, creating accounts, executing functions, and retrieving blockchain data.
