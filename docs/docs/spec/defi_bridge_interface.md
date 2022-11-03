# Defi Bridge Contract Interface

## Types

```
library AztecTypes {
    enum AztecAssetType {
        NOT_USED,
        ETH,
        ERC20,
        VIRTUAL
    }

    struct AztecAsset {
        uint256 id;
        address erc20Address;
        AztecAssetType assetType;
    }
}
```

The `AztecAsset` struct is an attempt at a more developer-friendly description of an Aztec asset that does not rely on bit flags.

The _type_ of the asset is described by an enum. For virtual or not used assets, the `erc20Address` will be 0.

For input virtual assets, the `id` field will contain the interaction nonce of the interaction that created the asset.

For output virtual assets, the `id` field will be the current interaction nonce.

## External Methods

### convert

Initiate a DeFi interaction and inform the rollup contract of the proceeds. If the DeFi interaction cannot proceed for any reason, it is expected that the convert method will throw.

```solidity
    function convert(
        AztecTypes.AztecAsset memory inputAssetA,
        AztecTypes.AztecAsset memory inputAssetB,
        AztecTypes.AztecAsset memory outputAssetA,
        AztecTypes.AztecAsset memory outputAssetB,
        uint256 totalInputValue,
        uint256 interactionNonce,
        uint64 auxData
    )
        external
        payable
        override
        returns (
            uint256 outputValueA,
            uint256 outputValueB,
            bool _isAsync
        )
```

###### Input Values:

| Name               | Type         | Description                                                                                      |
| ------------------ | ------------ | ------------------------------------------------------------------------------------------------ |
| `inputAssetA`      | _AztecAsset_ | first input asset                                                                                |
| `inputAssetB`      | _AztecAsset_ | second input asset. Either VIRTUAL or NOT_USED                                                   |
| `outputAssetA`     | _AztecAsset_ | first output asset. Cannot be virtual                                                            |
| `outputAssetB`     | _AztecAsset_ | second output asset. Can be real or virtual (or NOT_USED)                                        |
| `totalInputValue`  | _uint256_    | The amount of `inputAsset` this bridge contract is allowed to transfer from the rollup contract. |
| `interactionNonce` | _uint256_    | The current defi interaction nonce                                                               |
| `auxData`          | _uint64_     | Custom auxiliary metadata                                                                        |

###### Return Values:

| Name           | Type      | Description                                                                                                                                            |
| -------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `outputValueA` | _uint256_ | The amount of `outputAssetA` the rollup contract will be able to transfer from this bridge contract. Must be greater than 0 if `numOutputAssets` is 1. |
| `outputValueB` | _uint256_ | The amount of `outputAssetB` the rollup contract will be able to transfer from this bridge contract. Must be 0 if `numOutputAssets` is 1.              |

In the unfortunate event when both output values are zeros, this function should throw so that the rollup contract could refund `inputValue` back to the users.

[^1]: BridgeCallData is a 250-bit concatenation of the following data (starting at the most significant bit position):

| bit position | bit length | definition        | description                                                                         |
| ------------ | ---------- | ----------------- | ----------------------------------------------------------------------------------- |
| 0            | 64         | `auxData`         | custom auxiliary data for bridge-specific logic                                     |
| 64           | 32         | `bitConfig`       | flags that describe asset types                                                     |
| 96           | 32         | `openingNonce`    | (optional) reference to a previous defi interaction nonce (used for virtual assets) |
| 128          | 30         | `outputAssetB`    | asset id of 2nd output asset                                                        |
| 158          | 30         | `outputAssetA`    | asset id of 1st output asset                                                        |
| 188          | 30         | `inputAsset`      | asset id of 1st input asset                                                         |
| 218          | 32         | `bridgeAddressId` | id of bridge smart contract address                                                 |

Bit Config Definition

| bit | meaning |
| --- | --- |
| 0 | firstInputVirtual |
| 1 | secondInputVirtual |
| 2 | firstOutputVirtual |
| 3 | secondOutputVirtual |
| 4 | secondInputReal |
| 5 | secondOutputReal |
