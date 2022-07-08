---
title: Ethereum Interactions
---

One of the most exciting features of Aztec Connect is the ability to interact directly with Ethereum from Aztec. Interactions with Ethereum are facilitated by bridge contracts. You can read more about bridge contracts in the [Aztec Connect bridges repo](https://github.com/AztecProtocol/aztec-connect-bridges) on Github.

The [DefiController](../types/sdk/DefiController) makes it easy to interact directly with deployed Ethereum bridge contracts. Bridge contracts make the connection between the Aztec rollup processor contract and various Ethereum contracts.

## Setup

```ts
AztecSdk.createDefiController(
    userId: GrumpkinAddress, 
    userSigner: Signer, 
    bridgeId: BridgeId, 
    value: AssetValue, 
    fee: AssetValue)
    : Promise<DefiController>
```

### Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| userId | [GrumpkinAddress](../types/barretenberg/GrumpkinAddress) | Owner of the value note to be sent. |
| userSigner | [Signer](../types/sdk/Signer) | A signer associated with the `userId`. |
| bridgeId | [BridgeId](../types/barretenberg/BridgeId) | A unique id corresponding to the bridge that the `value` is sent to. |
| value | [AssetValue](../types/barretenberg/AssetValue) | Asset type and amount to send. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | Asset type and amount to pay for the Aztec transaction fee. |

#### Returns

| Return Type | Description |
| --------- | ----------- |
| [DefiController](../types/sdk/DefiController) | A user instance with apis bound to the user's account id. |

## BridgeId Setup

A bridge `addressId` is required to setup a [BridgeId](../types/barretenberg/BridgeId). The `addressId` is just a number associated with the bridge. It increments by 1 as new bridges are deployed to Ethereum and added to the rollup processor contract.

You can get the bridge `addressId`s from the published [Deployed Bridge Info table](https://github.com/AztecProtocol/aztec-connect-bridges#deployed-bridge-info).

You can also query the bridge ids on the rollup processor contracts directly. [Here is the link](https://etherscan.io/address/0xff1f2b4adb9df6fc8eafecdcbf96a2b351680455#readProxyContract
) to read the contract Etherscan. You can get the bridge contract addresses from the `getSupportedBridge` or `getSupportedBridges` functions. The bridge `addressId` corresponds to it's index in the supported bridges array returned by `getSupportedBridges`.

Once you have the bridge `addressId`, you can initialize a new bridge instance. The `BridgeId` contstructor looks like this:

```ts
const bridge = new BridgeId(
        addressId: number, 
        inputAssetIdA: number, 
        outputAssetIdA: number, 
        inputAssetIdB?: number | undefined, 
        outputAssetIdB?: number | undefined, 
        auxData?: number);
```

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| addressId | number | The id of the bridge in the rollup processor contract. |
| inputAssetIdA | number | The [asset id](../../glossary#asset-ids) of the first input asset. |
| outputAssetIdA | number | The [asset id](../../glossary#asset-ids) of the first output asset. |
| inputAssetIdB | number | Optional. The [asset id](../../glossary#asset-ids) of the second input asset. |
| outputAssetIdB | number | Optional. The [asset id](../../glossary#asset-ids) of the second output asset. |
| auxData | number | Custom auxiliary data for bridge-specific logic. |

The `BridgeId` is passed to the `DefiController` to specify how to construct the interaction.

For example, you can create the `BridgeId` for the ETH to wstETH Lido bridge like this:

```ts
const ethToWstEthBridge = new BridgeId(2, 0, 2); // IN: ETH (0), OUT: wstETH (2)
```

The Lido bridge contract is `addressId` 2, takes 1 input asset (ETH, `assetId` = 0) and returns 1 output asset (wstETH, `assetId` = 2). This bridge doesn't take any `auxData` since it is just a simple exchange of ETH to wstETH.

### Advanced usage

The Element bridge is a bit more complicated than the Lido bridge, so it's worth going through. The Element bridge allows users to deposit funds into Element directly from Aztec to earn yield.

The Element bridge is asynchronous, meaning there is a user action to deposit funds and then another action later to withdraw. The bridge also uses `auxData` to track which [tranche](https://docs.element.fi/element/element-smart-contracts/core-protocol-contracts/tranche) a user is interacting with.

#### Typescript Bridge Connectors

The bridges repo contains a `client` directory that contains useful helper functions for interacting with deployed bridge contracts. You can review the full Element bridge file [here](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/client/element/element-bridge-data.ts). This file includes functions to get the expected `auxData` values, get expected yields and historical interaction information.

For this Element connector, we need to pass in an Ethereum provider, the rollup contract address, the bridge contract address and a mainnet flag (useful for testing). You can view the `ElementBridgeData` class interface [here](../types/bridge-clients/ElementBridgeData).

For example:

```ts
const elementAdaptor = createElementAdaptor(
    ethereumProvider,
    "0xFF1F2B4ADb9dF6FC8eAFecDcbF96A2B351680455", // rollup address
    "0xaeD181779A8AAbD8Ce996949853FEA442C2CDB47", // bridge address
    false // mainnet flag
);
```

Once it's set up, we can ask it for the expected `auxData` values with:

```ts
await elementAdaptor.getAuxData?(
    inputAssetA: AztecAsset, 
    inputAssetB: AztecAsset, 
    outputAssetA: AztecAsset, 
    outputAssetB: AztecAsset): Promise<bigint[]>;
```

This function returns an index of numbers corresponding to various tranche expiry times that correspond to the inputAsset. This will determine which tranche the user interacts with. So you could set up a `BridgeId` for a user to interact with the latest tranche by passing the last index of the array of returned expiry times.

Looking at the [Deployed Bridge Info table](https://github.com/AztecProtocol/aztec-connect-bridges#deployed-bridge-info), the Element bridge is id 1, it takes DAI (asset id 1) and returns DAI and you can pass an element from the `auxData` array from the connector and pass this to the `DefiController`.

```ts
const elementBridge = new BridgeId(1, 1, 1, undefined, undefined, Number(elementAuxData[0])); // IN: DAI (1), OUT: DAI (1)
```

#### Async `finalise`

Since the Element bridge is asynchronous, the `finalise` function must be called on the bridge contract after the Element tranche has matured to get the deposit + interest back in the Aztec account.

In the case of the Element bridge, this is handled automatically by users entering new Element positions. The bridge contract checks if there are any positions available to finalise and if there are, it will process them. You can view the relevant code in the bridge [here](https://github.com/AztecProtocol/aztec-connect-bridges/blob/25cb63d8092350527ab143be97142119bec638fe/src/bridges/element/ElementBridge.sol#L511).

If you want to finalise a defi interaction without having to rely on interactions from others or do it independently of other bridge contract interactions, you can. There is a `processAsyncDefiInteraction` function on the [rollup processor contract](https://github.com/AztecProtocol/aztec-connect/blob/b2103376608e46ffe50cf56f9ca5ce031f34c671/blockchain/contracts/RollupProcessor.sol#L748) that takes an `interactionNonce`. You can call this from any Ethereum account. You can fetch defi transaction `interactionNonces` from an Aztec account using the `getUserTxs` method in the [SDK](../types/sdk/AztecSdk).

## Fees

Similarly to other controllers, the SDK comes with a helper method to calculate fees. It requires the `BridgeId` since different bridges cost different amounts of gas. `settlementTime` is type [DefiSettlementTime](../types/barretenberg/DefiSettlementTime).

`DefiSettlementTime` is an enum with members `DEADLINE`, `NEXT_ROLLUP` and `INSTANT`. `DEADLINE` is the slowest and cheapest. It provides a longer opportunity for your transaction to be batched with other similar interactions. `NEXT_ROLLUP` is the next most expensive and will process in the next regardless of the number of similar interactions. `INSTANT` is the most expensive and will pay the cost to settle the rollup and process the interaction immediately.

```ts
const fee = (await sdk.getDefiFees(bridgeId))[settlementTime];
```

The settlement time is inferred from the fee a user pays, it is not explicitly sent to the controller.

## Bridge Address Ids

You can find the latest bridge contract `addressId`s and Ethereum contract addresses in the [Aztec Connect Bridges repo](https://github.com/AztecProtocol/aztec-connect-bridges).