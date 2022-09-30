---
title: Aztec Connect Bridges
---

This guide is intended for readers interested in developing Aztec Connect Bridges or learning about them technically, instead of using them.

## What is Aztec Connect

Aztec Connect is a framework that enables users to access protocols on Ethereum Layer 1 (L1) privately, securely and inexpensively. It consists of three main components:

**L1 Protocols**
Your favorite protocols already deployed and running on L1, e.g. Lido, Element, Yearn, Uniswap and more.
    
**Aztec Connect Bridges**
Smart contracts on L1 that facilitate the Aztec rollup contract to interact with other L1 Protocols on behalf of L2 Aztec Network users.

**Aztec SDK**
A tool set for developing user-facing applications to interact with the Aztec Network, including APIs for interacting with L1 Protocols using Aztec Connect Bridges.

A typical flow of action starts from a user initiating an interaction request through a frontend powered by the Aztec SDK. The request is then passed to the Aztec backend and the corresponding Aztec Connect Bridge, which interacts with the specific L1 Protocol on behalf of the user.

## What are Aztec Connect Bridges

Aztec Connect Bridges are L1 smart contracts that conform L1 Protocols to the interface of the L1 Aztec rollup contract. They enable L2 Aztec users to interact with L1 Protocols cheaply and privately.

A bridge contract models any L1 Protocol as an asset swap. Up to two input assets and two output assets can be specified for each bridge:

![](https://i.imgur.com/2N4Noha.jpg)

> **Note:** Assets A and B shall be of identical amounts (but not necessarily of identical types) in the current design to avoid free-riders. Further research to lift the limitation is underway.

The asset swap delay depends on whether the bridge is designed to be synchronous or asynchronous. For more information on synchronicity, check the [Sync vs Async](#Sync-vs-Async) section.

A high-level look of the Lido-Curve bridge that swaps users' ETH into wstETH:

![](https://i.imgur.com/0wtoDKk.jpg)

For more information on how Aztec Connect Bridges work, check the workshop video linked at the beginning of this guide and the [Aztec Connect](../how-aztec-works/aztec-connect) section of the Aztec Docs.
    
## Workshop Video

Certain content of this guide is also covered in this workshop video:

[![](https://i.imgur.com/fA8f1Du.jpg)](https://www.youtube.com/watch?v=029Vm6PAnrM&t=1822s)

## Writing a Bridge

### Solidity Dev Env
    
An Aztec Connect Bridge is developed entirely in Solidity. You are therefore welcome to choose your preferred Solidity development environment.
    
We recommend [Foundry](https://book.getfoundry.sh/) given the interconnected nature of Aztec Connect Bridges with existing Mainnet protocols.

### Boilerplates
    
To get started with an example setup, follow the steps below:
    
1. Fork the [Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges).

2. Clone your fork:
    
```shell
git clone https://github.com/{YOUR_GITHUB_USERNAME}/aztec-connect-bridges.git
```

3. Create a new branch with your preferred bridge name:
    
```shell
git checkout -b {YOUR_GITHUB_USERNAME}/{BRIDGE_NAME}
```
    
4. Change directory into `aztec-connect-bridges`:

```shell
cd aztec-connect-bridges
```
    
5. Install dependencies and build the repository:
    
```shell
yarn setup
```

6. Copy and rename the example folders as your boilerplates to work on:

```
src/bridges/example
src/test/example
src/client/example
src/deployment/example
src/specs/TEMPLATE.md
```

You are now equipped to write your first Aztec Connect Bridge! Simply start by reading and implementing your bridge over the boilerplate files.

For more information on how the Aztec rollup contract would call your bridge, check [IDeFiBridge.sol](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/aztec/interfaces/IDefiBridge.sol).

### BridgeCallData

In production, a bridge is called by a user creating a client-side proof via the Aztec SDK. These transaction proofs are sent to the sequencer for aggregation. The sequencer then sends the aggregate rollup proof with the sum of all users' proofs with identical `BridgeCallData` ([class definition](../sdk/types/bridge-clients/BridgeData)) to your bridge contract in one go for gas savings and improved privacy.

A `BridgeCallData` uniquely defines the expected inputs/outputs of an L1 interaction. It is a `uint256` that represents a bit-string containing multiple fields. When unpacked its data is used to create a `BridgeData` struct in the rollup contract.

The structure of the bit-string is as follows (starting at the least significant bit):

| bit position | bit length | definition | description |
| - | - | - | - |
| 0 | 32 | `bridgeAddressId` | id of bridge smart contract address |
| 32 | 30 | `inputAssetA` | asset id of 1st input asset |
| 62 | 30 | `inputAssetB` | asset id of 1st input asset |
| 92 | 30 | `outputAssetA` | asset id of 1st output asset |
| 122 | 30 | `outputAssetB` | asset id of 2nd output asset |
| 184 | 64 | `auxData` | custom auxiliary data for bridge-specific logic |

> **Note:** The last 8 bits of the `BridgeCallData` bit-string are wasted as the rollup proving circuits cannot support values of full 256 bits (248 is the largest multiple of 8 that we can use).  
    
`bitConfig` definition:

| bit | meaning |
| - | - |
| 0 | secondInputInUse |
| 1 | secondOutputInUse |

> **Note:** Despite using only 2 bits in the current design, `bitConfig` is 32 bits large for future-proofing (new bit flags would be needed to add e.g.  NFT support).

For more information on bridge call data, check the [Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges) README.

### Tests

Testing is critical to ensure your bridge is working as intended. Refer to example tests under [`src/test/bridges/example/`](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master/src/test/bridges/example) and other tests under [`src/test/bridges/`](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master/src/test/bridges) for inspirations.

The main objective of unit tests is to demonstrate the bridge works by itself. The testing focus is recommended to be on edge cases, reverts, output value assertions and fuzzy tests.

The main objective of end-to-end (E2E) tests, meanwhile, is to demonstrate the bridge works in a production-like environment. The testing setup should involve mocking the rollup with [`BridgeTestBase.sol`](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/test/aztec/base/BridgeTestBase.sol) and the focus is recommended to be on event emissions and token transfers.

For Foundry users:

```shell
forge test --match-contract {BRIDGE_NAME} -vvv
```

### Deployment

The best way to deploy your bridge is through a deployment script with Foundry.

:::info
Read more about Solidity scripting with foundry [here](https://book.getfoundry.sh/tutorials/solidity-scripting).
:::

Refer to [`ExampleDeployment.s.sol`](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master/src/deployment/example) and other scripts under [`src/deployment`](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master/src/deployment) for inspirations.

The following command will run the `deployAndList()` function in `ExampleDeployment.s.sol`. You will need to export a couple of environment variables before running the command.

```shell
export network=testnet # wont work on mainnet, permissionless bridge listing not enabled yet
export simulateAdmin=false # to broadcast your deployment to the testnet
```

```shell
forge script --fork-url https://mainnet-fork.aztec.network:8545 --private-key $PRIV --legacy --ffi ExampleDeployment --sig "deployAndList()" 
```

where `$PRIV` is a private key for an Ethereum account on the testnet that has ETH to pay fees.

Some notes on the additional flags in the above command:

- `--ffi` allows us to access stuff outside solidity, so we use it to fetch rollup processor address
- `--sig` is the function signature that we want to call
- `--legacy` is because ganache (which the testnet is running on) and eip1559 don't play well
- `-vvvv` prints trace

Refer to [this section](https://github.com/AztecProtocol/aztec-connect-bridges#writing-a-bridge) of the bridges repo README for more detail.

You can use this command to get all of the deployed assets and bridges on the testnet.

```shell
forge script --fork-url https://mainnet-fork.aztec.network:8545 --ffi AggregateDeployment --sig "readStats()"
```

### Aux Data

The `auxData` field in the bridge call data is custom auxiliary data supporting bridge-specific logic.

To benefit from the gas savings and improved privacy of aggregated proofs with identical `bridgeCallData`, the definition of `auxData` of a bridge could be an important consideration during its design process.
    
### Rebasing Token

Tokens bridged onto the Aztec Network are represented as Aztec notes of fixed values. Bridging rebasing tokens like Lido stETH and Ampleforth naively without wrappers would result in users losing out on entitled rewards and suffering from insolvent withdrawals.

The use of a canonical wrapper like Lido wstETH / a self-built wrapper to anchor the amounts of bridged assets is therefore highly recommended.

### Sync vs Async

Depending on the application, Aztec Connect interactions that require 2-step processes can utilize the asynchronous option by flipping the `isAsync` flag during bridge design.

An example of an asynchronous bridge would be [Element's](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/bridges/element/ElementBridge.sol), where redemptions are activated no earlier than the maturity of deposits.

### Stateful vs Stateless

Another consideration when designing an Aztec Connect Bridge is to decide if it should hold state within the contract.

For interactions involving fungible positions (e.g. token swaps), a stateless bridge that does not hold funds outside calls is likely preferred for the generally smaller code base.

For interactions involving long-standing non-fungible positions (e.g. borrowing, DCA, limit orders), a stateful bridge that handles accounting internally and holds funds between calls is likely required.

### Gas Limit

As a measure to avoid the entire Aztec rollup failing from out-of-gas issues, bridges are required to specify their upper gas usage limit when registering on the Aztec rollup.

Bridge designers should take variations in gas usage that depend on alterable L1 state into account when deciding their gas limits.

### Bridge Reverts

When a bridge reverts, the Aztec rollup will emit an event indicating that the bridge has reverted, and then continue with Aztec Connect interactions of other remaining bridges.

This could lead to tricky debugging if E2E tests are carried out as the first tests post-design, as revert messages are only discoverable in emitted events.

### ERC-4626

An [ERC-4626 Aztec Connect Bridge](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/bridges/erc4626/ERC4626Bridge.sol) that supports tokenized vaults complying with the [EIP-4626 standard](https://eips.ethereum.org/EIPS/eip-4626) is available. If an Aztec Connect interaction can conform to an ERC-4626 position, it may be desirable to utilize the existing bridge instead of building a new one.

## Resources

### [📝 Aztec Connect Bridges Repo](https://github.com/AztecProtocol/aztec-connect-bridges/)

The repository containing code of Aztec Connect Bridges deployed and in development, as well as boilerplates and information for writing a new bridge.

### [👾 Discord](https://discord.gg/aztec)

Join the channels:

- [`#💻│aztec-connect`](https://discord.com/channels/563037431604183070/563038059826774017) to discuss Aztec Connect
- [`#🇨🇴│ethbogota`](https://discord.com/channels/563037431604183070/1021410163221086268) to discuss the ETHBogota Hackathon