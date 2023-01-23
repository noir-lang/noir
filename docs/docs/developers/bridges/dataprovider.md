---
title: DataProvider
---

## What does it do?

It is a contract we use as a source of truth when it comes to configuration of bridges and assets.

It is mainly used by a frontend to get a bridge or asset information by a tag, but the information can be used by anyone.

You can also get deployed bridge information from the Aztec sequencer (falafel) endpoint: [https://api.aztec.network/aztec-connect-prod/falafel/status](https://api.aztec.network/aztec-connect-prod/falafel/status).

## Usage

The contract is deployed [here](https://etherscan.io/address/0xB4319947947781FFe91dDf96A32aF2D4693FEf64) and these are the 4 relevant functions:

```js
function getAsset(uint256 _assetId) public view returns (AssetData memory);

function getAsset(string memory _tag) public view returns (AssetData memory);

function getAssets() public view returns (AssetData[] memory);

function getBridges() public view returns (BridgeData[] memory);
```

The easiest way to access it, if already using [the Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master) is to execute the `readProvider(address)` script in the DataProviderDeployment solidity file. You will need to specify the DataProvider address for the network you want to read.

For example

```bash
export NETWORK=MAINNET
export ETH_RPC_URL=<INSERT_URL>
```

```bash
forge script DataProviderDeployment --rpc-url $ETH_RPC_URL --sig "readProvider(address)" 0x8B2E54fa4398C8f7502f30aC94Cb1f354390c8ab
```

## Usage by owner

Updating values stored in the data provider is only possible by the owner of the contract. This is a restricted function on mainnet, but you can update your [local development environment](../local-devnet).

Before running the commands below export relevant environment variables:

```bash
export RPC="http://localhost:8545" && PRIV_KEY="PROVIDER_OWNER_PRIVATE_KEY"
```

### Adding a bridge

```bash
cast send 0xB4319947947781FFe91dDf96A32aF2D4693FEf64 "addBridge(uint256,string)" "2" "ExampleBridge" --rpc-url $RPC --private-key $PRIV_KEY
```

### Adding an asset

```bash
cast send 0xB4319947947781FFe91dDf96A32aF2D4693FEf64 "addAsset(uint256,string)" "2" "wsteth" --rpc-url $RPC --private-key $PRIV_KEY
```

### Adding multiple assets and bridges

```bash
cast send 0xB4319947947781FFe91dDf96A32aF2D4693FEf64 "addAssetsAndBridges(uint256[],string[],uint256[],string[])" '[2,1]' '["wsteth","dai"]' '[5,8]' '["ExampleBridge_deposit","ExampleBridge_withdraw"]' --rpc-url $RPC --private-key $PRIV_KEY
```
