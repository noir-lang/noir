---
title: Technical Introduction
---

This page is taken from the [Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges) README on Github.

### What is Aztec?

Aztec is a privacy focused L2, that enables cheap private interactions with Layer 1 smart contracts and liquidity, via a process called DeFi Aggregation. We use advanced zero-knowledge technology "zk-zk rollups" to add privacy and significant gas savings any Layer 1 protocol via Aztec Connect bridges.

#### What is a bridge?

A bridge is a Layer 1 Solidity contract deployed on mainnet that conforms a DeFi protocol to the interface the Aztec rollup expects. This allows the Aztec rollup contract to interact with the DeFi protocol via the bridge.

A bridge contract models any Layer 1 DeFi protocol as an asynchronous asset swap. You can specify up to two input assets and two output assets per bridge.

#### How does this work?

Users who have shielded assets on Aztec can construct a zero-knowledge proof instructing the Aztec rollup contract to make an external L1 contract call.

Rollup providers batch multiple L2 transaction intents on the Aztec Network together in a rollup. The rollup contract then makes aggregate transaction against L1 DeFi contracts and returns the funds pro-rata to the users on L2.

#### How much does this cost?

Aztec Connect transactions can be batched together into one rollup. Each user in the batch pays a base fee to cover the verification of the rollup proof and their share of the L1 DeFi transaction gas cost. A single Aztec Connect transaction requires 3 base fees or approximately ~8000 gas.

The user then pays their share of the L1 Defi transaction. The rollup will call a bridge contract with a fixed deterministic amount of gas so the total fee is simple ~8000 gas + BRIDGE_GAS / BATCH_SIZE.

#### What is private?

The source of funds for any Aztec Connect transaction is an Aztec shielded asset. When a user interacts with an Aztec Connect Bridge contract, their identity is kept hidden, but balances sent to the bridge are public.

#### Batching

Rollup providers are incentivised to batch any transaction with the same `bridgeId`. This reduces the cost of the L1 transaction for similar trades. A `bridgeId` consists of:

## Virtual Assets

Aztec uses the concept of Virtual Assets or Position tokens to represent a share of assets held by a Bridge contract. This is far more gas efficient than minting ERC20 tokens. These are used when the bridge holds an asset that Aztec doesn't support (e.g. Uniswap Position NFT's or other non-fungible assets).

If the output asset of any interaction is specified as "VIRTUAL", the user will receive encrypted notes on Aztec representing their share of the position, but no tokens or ETH need to be transferred. The position tokens have an `assetId` that is the `interactionNonce` of the DeFi Bridge call. This is globally unique. Virtual assets can be used to construct complex flows, such as entering or exiting LP positions (e.g. one bridge contract can have multiple flows which are triggered using different input assets).

## Aux Input Data

The `auxData` field can be used to provide data to the bridge contract, such as slippage, tokenIds etc.

### Bridge Contract Interface

```ts
// SPDX-License-Identifier: GPL-2.0-only
// Copyright 2020 Spilsbury Holdings Ltd
pragma solidity >=0.6.6 <0.8.0;
pragma experimental ABIEncoderV2;

import { Types } from "../Types.sol";

interface IDefiBridge {
  /**
   * Input cases:
   * Case1: 1 real input.
   * Case2: 1 virtual asset input.
   * Case3: 1 real 1 virtual input.
   *
   * Output cases:
   * Case1: 1 real
   * Case2: 2 real
   * Case3: 1 real 1 virtual
   * Case4: 1 virtual
   *
   * Example use cases with asset mappings
   * 1 1: Swapping.
   * 1 2: Swapping with incentives (2nd output reward token).
   * 1 3: Borrowing. Lock up collateral, get back loan asset and virtual position asset.
   * 1 4: Opening lending position OR Purchasing NFT. 
            Input real asset, get back virtual asset representing NFT or position.
   * 2 1: Selling NFT. Input the virtual asset, get back a real asset.
   * 2 2: Closing a lending position. Get back original asset and reward asset.
   * 2 3: Claiming fees from an open position.
   * 2 4: Voting on a 1 4 case.
   * 3 1: Repaying a borrow. Return loan plus interest. Get collateral back.
   * 3 2: Repaying a borrow. Return loan plus interest. 
            Get collateral plus reward token. (AAVE)
   * 3 3: Partial loan repayment.
   * 3 4: DAO voting stuff.
   */

  /*
  @dev This function is called from the RollupProcessor.sol contract via the DefiBridgeProxy.
    It receives the aggregate sum of all users funds for the input assets.
  @param AztecAsset inputAssetA a struct detailing the first input asset, 
    this will always be set
  @param AztecAsset inputAssetB an optional struct detailing the second input asset, 
    this is used for repaying borrows and should be virtual
  @param AztecAsset outputAssetA a struct detailing the first output asset, 
    this will always be set
  @param AztecAsset outputAssetB a struct detailing an optional second output asset
  @param uint256 inputValue, the total amount input, 
    if there are two input assets, equal amounts of both assets will have been input
  @param uint256 interactionNonce a globally unique identifier for this DeFi interaction.
    This is used as the assetId if one of the output assets is virtual
  @param uint64 auxData other data to be passed into the bridge contract (slippage / nftID etc)
  @return uint256 outputValueA the amount of outputAssetA returned from this interaction,
    should be 0 if async
  @return uint256 outputValueB the amount of outputAssetB returned from this interaction, 
    should be 0 if async or bridge only returns 1 asset.
  @return bool isAsync a flag to toggle if this bridge interaction will return assets 
    at a later date after some third party contract has interacted with it via finalise()
  */
  function convert(
    Types.AztecAsset calldata inputAssetA,
    Types.AztecAsset calldata inputAssetB,
    Types.AztecAsset calldata outputAssetA,
    Types.AztecAsset calldata outputAssetB,
    uint256 inputValue,
    uint256 interactionNonce,
    uint64 auxData
  )
    external
    payable
    returns (
      uint256 outputValueA,
      uint256 outputValueB,
      bool isAsync
    );

  /*
  @dev This function is called from the RollupProcessor.sol contract via the DefiBridgeProxy.
    It receives the aggregate sum of all users funds for the input assets.
  @param AztecAsset inputAssetA a struct detailing the first input asset, 
    this will always be set
  @param AztecAsset inputAssetB an optional struct detailing the second input asset, 
    this is used for repaying borrows and should be virtual
  @param AztecAsset outputAssetA a struct detailing the first output asset, 
    this will always be set
  @param AztecAsset outputAssetB a struct detailing an optional second output asset
  @param uint256 interactionNonce
  @param uint64 auxData other data to be passed into the bridge contract (slippage / nftID etc)
  @return uint256 outputValueA the return value of output asset A
  @return uint256 outputValueB optional return value of output asset B
  @dev this function should have a modifier on it to ensure 
    it can only be called by the Rollup Contract
  */
  function finalise(
    Types.AztecAsset calldata inputAssetA,
    Types.AztecAsset calldata inputAssetB,
    Types.AztecAsset calldata outputAssetA,
    Types.AztecAsset calldata outputAssetB,
    uint256 interactionNonce,
    uint64 auxData
  ) external payable returns (uint256 outputValueA, uint256 outputValueB, bool interactionComplete);
}

```

### Async flow explainer

If a Defi Bridge interaction is asynchronous, it must be finalised at a later date once the DeFi interaction has completed.

This is achieved by calling `RollupProcessor.processAsyncDefiInteraction(uint256 interactionNonce)`. This internally will call `finalise` and ensure the correct amount of tokens have been transferred.

#### convert()

This function is called from the Aztec Rollup Contract via the DeFi Bridge Proxy. Before this function on your bridge contract is called the rollup contract will have sent you ETH or tokens defined by the input params.

This function should interact with the DeFi protocol (e.g Uniswap) and transfer tokens or ETH back to the Aztec Rollup Contract. The Rollup contract will check if it received the correct amount.


At a later date, this interaction can be finalised by prodding the rollup contract to call `finalise` on the bridge.

#### finalise()

This function will be called from the Aztec Rollup contract. The Aztec Rollup contract will check that it received the correct amount of ETH and tokens specified by the return values and trigger the settlement step on Aztec.
