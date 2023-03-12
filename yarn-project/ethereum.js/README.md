# ethereum.js

A streamlined L1 client library that meets Aztec's needs. Can provide full contract type safety and IDE
auto-completion by leveraging the `contract_gen_def` tool.

There is an example project [here](./example/src/index.ts). Play with it in your IDE to experience auto-completion.

## Installing

`yarn add @aztec/ethereum.js`

## Building Contract Definitions from ABIs

Define a `contracts.json` file, e.g:

```json
{
  "outputPath": "./src/contracts",
  "contracts": {
    "RollupProcessorContract": {
      "source": "etherscan",
      "net": "mainnet",
      "address": "0x8430be7b8fd28cc58ea70a25c9c7a624f26f5d09"
    },
    "ERC20Mintable": {
      "source": "foundry",
      "buildFile": "./src/abis/ERC20Mintable.json"
    }
  }
}
```

### Running The Generator

If you want to generate a definition from an Etherscan ABI, you'll need to set `ETHERSCAN_API_KEY`.
If you want bytecode included for deployments, you'll need to provide an `ETHEREUM_HOST`.

```
export ETHEREUM_HOST=https://mainnet.infura.io/v3/<infura_apikey>
export ETHERSCAN_API_KEY=<etherscan_apikey>
yarn contract_gen_def
```

This will output two classes:

- `RollupProcessorContract` in `./src/contracts/RollupProcessorContract.ts`
- `ERC20Mintable` in `./src/contracts/ERC20Mintable.ts`

They will have method definitions using sensible types, unique per parameter e.g:

- `number` (<= 32 bits)
- `bigint` (> 32 bits)
- `string`
- `EthAddress`
- `Bytes32`

Transaction receipts will have logs and errors decoded according to abi entries.

### Snippet of ERC20Mintable Definition File

```ts
interface ERC20MintableMethods {
  allowance(owner: EthAddress, spender: EthAddress): TxCall<bigint>;
  approve(spender: EthAddress, amount: bigint): TxSend<ERC20MintableTransactionReceipt, boolean>;
  balanceOf(account: EthAddress): TxCall<bigint>;
  decimals(): TxCall<number>;
  decreaseAllowance(spender: EthAddress, subtractedValue: bigint): TxSend<ERC20MintableTransactionReceipt, boolean>;
  increaseAllowance(spender: EthAddress, addedValue: bigint): TxSend<ERC20MintableTransactionReceipt, boolean>;
  mint(_to: EthAddress, _value: bigint): TxSend<ERC20MintableTransactionReceipt, boolean>;
  name(): TxCall<string>;
  setDecimals(_decimals: number): TxSend<ERC20MintableTransactionReceipt>;
  symbol(): TxCall<string>;
  totalSupply(): TxCall<bigint>;
  transfer(to: EthAddress, amount: bigint): TxSend<ERC20MintableTransactionReceipt, boolean>;
  transferFrom(from: EthAddress, to: EthAddress, amount: bigint): TxSend<ERC20MintableTransactionReceipt, boolean>;
}
```

## Using the Contracts

Below is an example of using ERC20Mintable to deploy, mint, transfer and handle logs.

```ts
// Deploy.
const contract = new ERC20Mintable(ethRpc, undefined, { from: acc1, gas: 1000000 });
await contract.deploy('AZT').send().getReceipt();
console.log(`Deployed ERC20 with symbol: ${await contract.methods.symbol().call()}`);

// Mint.
console.log(`Minting 1000 tokens to ${acc1}...`);
await contract.methods.mint(acc1, toBaseUnits('1000', 18)).send().getReceipt();

// Transfer.
console.log(`Transferring from ${acc1} to ${acc2}`);
const receipt = await contract.methods.transfer(acc2, toBaseUnits('0.1', 18)).send().getReceipt();
const [{ args }] = receipt.events.Transfer;
console.log(`Log shows transfer of ${args.value} from ${args.from} to ${args.to}`);
```
