---
title: Getting Started
---

## Building on Aztec

There are two common ways that developers can start building on Aztec:

1. Build a user-facing application that connects to the Aztec network via the TypeScript SDK.
2. Build an Aztec Connect bridge that connects the Aztec network to Ethereum smart contracts.

We are also working on Noir, a domain specific language that can be used to develop ZK-provable programs. Scroll down for more details.

:::tip
Running out of ideas to build? Check our [ETHBogota Hacking Ideas](https://docs.google.com/document/d/1uQ6jOU4LfnkzoaETFbk1EouVD41Dh5j6jjtcCh6tpEU) and [Aztec Grants](https://aztec.network/grants) for inspirations!
:::

:::info
The Aztec core engineering team has a regular release cadence that will occasionally introduce breaking changes. Updates are typically applied to Testnet on Thursdays, which are tested and monitored over the weekend and would be pushed to Mainnet afterwards.

We are working on making this process more transparent. Feel free to [get in touch](#get-in-touch) if you have any queries.
:::

## Testnet Information

### Configuration

Aztec maintains a Testnet environment that mimics Ethereum, enabling more comprehensive testing of projects built on Aztec.

To make use of the Testnet:

1. Connect Metamask (or other Ethereum wallet) to the Testnet:

| Properties      | Network Details                                           |
| --------------- | --------------------------------------------------------- |
| Network name    | Aztec Testnet                                             |
| RPC URL         | https://aztec-connect-testnet-eth-host.aztec.network:8545 |
| Chain ID        | 677868                                                    |
| Currency symbol | ETH                                                       |

2. Get Testnet funds:
   - Use the [Testnet faucet](https://aztec-connect-testnet-faucet.aztec.network/)
   - For larger amounts of Testnet ETH, [get in touch](#get-in-touch)

After connecting to the Testnet, you can try out Aztec projects on the Testnet such as [zk.money (Testnet)](https://aztec-connect-testnet.zk.money/), [Aztec CLI](https://github.com/critesjosh/azteccli) and the [Minimal Frontend](https://aztec-frontend-boilerplate.netlify.app/) that involve account registrations, deposits, transfers, withdrawals and more on the Aztec network.

For the status and details of the Aztec Testnet infrastructure, check the [Testnet Falafel Status API](https://api.aztec.network/aztec-connect-testnet/falafel/status).

### Deployed Aztec Connect Bridges

If you are developing with the SDK / on multi-step bridges that involve interaction with other existing bridges, you may come across the need to retrieve details of bridges already deployed on the Testnet (e.g. `bridgeAddressId`). There are currently two ways to do so:

#### Aztec Frontend Boilerplate

After logging in on the [Minimal Frontend](https://aztec-frontend-boilerplate.netlify.app/), you will see a "Log Bridges" button which logs bridge details in the browser console.

#### Foundry Script

Alternatively, you can read from the bridge data provider contract directly by running a script from the [Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges). Check [this section](./bridges#testnet-deployment-info) for more details.

## Building with the SDK

### Setup

To start using the SDK, install it in your project by running:

```shell
npm install @aztec/sdk
```

And import it into your project:

```ts
import { createAztecSdk } from "@aztec/sdk";
```

For a proper walkthrough of setting up the SDK, check the [SDK Setup page](../sdk/usage/setup.mdx).

### Example Code

To learn how the SDK works in action, the [CLI page](./cli) provides a detailed breakdown of the [Aztec CLI](https://github.com/critesjosh/azteccli) tool powered by the SDK.

If you are building a web interface, you might also find the [Frontend Boilerplate](https://github.com/Globallager/aztec-frontend-boilerplate) useful as a starting point.

:::tip
By default, Aztec transactions could take up to a few hours to settle on the Testnet like on Mainnet. If you want transactions to settle quickly, be sure to set `TxSettlementTime` as `INSTANT`.

`INSTANT` transactions pay higher fees in Testnet ETH in exchange for settlement within minutes rather than hours.
:::

### Aztec SDK Resources

- [🧑‍💻 Aztec SDK npm](https://www.npmjs.com/package/@aztec/sdk)
- [🧑‍💻 Aztec SDK GitHub Repo](https://github.com/AztecProtocol/aztec-connect/tree/master/sdk)
- [🎥 ETHBogota Workshop - Aztec CLI / SDK](https://www.youtube.com/watch?v=I5M8LhOECpM&t=744s)
- [✍️ Getting Started with Aztec CLI / SDK](./cli.md)
- [📝 Aztec CLI](https://github.com/critesjosh/azteccli)
- [📝 Aztec Frontend Boilerplate](https://github.com/Globallager/aztec-frontend-boilerplate)
- [📱 zk.money (Testnet)](https://aztec-connect-testnet.zk.money/)
- [🔍 Testnet Block Explorer](https://aztec-connect-testnet-explorer.aztec.network/)
- [🔍 Testnet Falafel Status API](https://api.aztec.network/aztec-connect-testnet/falafel/status)

## Building an Aztec Connect Bridge Contract

The [Aztec Connect Bridges page](./bridges) covers how to develop an Aztec Connect Bridge in great detail.

The [Aztec Connect Bridges repository](https://github.com/AztecProtocol/aztec-connect-bridges) has the most up to date information and materials for creating a bridge contract.

### Bridges Resources

- [🧑‍💻 Aztec Connect Bridges GitHub Repo](https://github.com/AztecProtocol/aztec-connect-bridges)
- [🎥 ETHBogota Workshop - Aztec Connect Bridges](https://www.youtube.com/watch?v=I5M8LhOECpM&t=1826s)
- [✍️ Getting Started with Aztec Connect Bridges](./bridges.md)
- [✍️ Element Bridge Explained](https://hackmd.io/@aztec-network/SJ7-6Rbfq)

## Building in Noir

Noir is a domain specific language for building programs provable with Zero-Knowledge Proofs (ZKP), tapping into the privacy and scaling benefits ZKP technology has to offer.

The [Noir page](./noir) covers how to install and develop in Noir in great detail.

### Noir Resources

- [📓 The Noir Book](https://noir-lang.github.io/book/index.html)
- [🧑‍💻 Noir GitHub Repo](https://github.com/noir-lang/noir)
- [🎥 ETHBogota Workshop - Noir](https://www.youtube.com/watch?v=I5M8LhOECpM&t=2879s)
- [✍️ Getting Started with Noir](./noir.md)
- [📝 Standard Noir Example](https://github.com/vezenovm/basic_mul_noir_example)
- [📝 Mastermind in Noir](https://github.com/vezenovm/mastermind-noir)
- [📝 Semaphore in Noir (Concept Build)](https://github.com/vezenovm/simple_shield)

## Get in Touch

### Discord

Join [Aztec Discord](https://discord.gg/aztec) for discussions across channels:

- [`#💻│aztec-connect`](https://discord.com/channels/563037431604183070/563038059826774017) for SDK & Bridges
- [`#🖤│noir`](https://discord.com/channels/563037431604183070/824700393677783080) for Noir
- [`#🇨🇴│ethbogota`](https://discord.com/channels/563037431604183070/1021410163221086268) for the ETHBogota Hackathon

### Aztec Core Team Contacts

All questions, comments, suggestions, ideas, etc. welcome.

| Name  | Role                               | Discord          | Telegram     | Twitter                                             | Email                   |
| ----- | ---------------------------------- | ---------------- | ------------ | --------------------------------------------------- | ----------------------- |
| Josh  | Developer Relations                | joshc#0001       | @crites      | [@critesjosh\_](https://twitter.com/critesjosh_)    | josh@aztecprotocol.com  |
| Savio | Developer Relations                | Globallager#4834 | @Globallager | [@globallager](https://twitter.com/globallager)     | savio@aztecprotocol.com |
| Lasse | Engineer - Bridge & Smart Contract | LHerskind#8376   |              | [@HerskindLasse](https://twitter.com/herskindlasse) | lasse@aztecprotocol.com |
| Maxim | Engineer - Noir                    | vezzie#7609      |              | [@maximvezenov](https://twitter.com/maximvezenov)   | maxim@aztecprotocol.com |
