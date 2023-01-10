---
title: DataProvider
---

## What does it do?

It is a contract we use as a source of truth when it comes to configuration of bridges and assets.
It is mainly used by our frontend to get a bridge or asset information by a tag, but the information can be used by anyone.

## Usage

The contract is deployed [here](https://etherscan.io/address/0xB4319947947781FFe91dDf96A32aF2D4693FEf64) and these are the 4 relevant functions:

```js
function getAsset(uint256 _assetId) public view returns (AssetData memory);

function getAsset(string memory _tag) public view returns (AssetData memory);

function getAssets() public view returns (AssetData[] memory);

function getBridges() public view returns (BridgeData[] memory);
```

The easiest way to access it, if already using [the Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master) is to execute the `read()` script in the DataProviderDeployment solidity file. It holds addresses for mainnet, testnet and devnet.

```bash
export network=<mainnet|testnet|devnet> && export simulateAdmin=false
export ETH_RPC_URL=<INSERT_URL>
forge script DataProviderDeployment --rpc-url $ETH_RPC_URL --sig "read()"
```

## Usage by owner

Updating values stored in the data provider is only possible by the owner of the contract.

Before running the commands below export relevant environment variables:

```bash
export RPC="https://mainnet.infura.io/v3/737bcb5393b146d7870be2f68a7cea9c" && PRIV_KEY="PROVIDER_OWNER_PRIVATE_KEY"
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
