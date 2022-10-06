---
title: Command Line Interface (CLI)
---

This guide is intended to introduce the Aztec SDK through the lens of Aztec CLI.

## What is Aztec CLI

Aztec CLI is a command line application for interacting with the Aztec Network, powered by the Aztec SDK. It is useful for both accessing the Aztec Network, as well as experimenting with the Aztec SDK.

## What is Aztec SDK

Aztec SDK is a set of tools for developing user-facing means to interact with the Aztec Network.

It is designed to abstract away the complexities of zero-knowledge proofs from developers, providing them with simple APIs to develop applications that enjoy the privacy and scaling benefits the Aztec Network offers.

## Workshop Video

A video demo of the Aztec CLI is available at:

[![](https://github.com/critesjosh/azteccli/raw/main/img/preview.png)](https://www.youtube.com/watch?v=Og04qRak-SM)

Certain content of this guide is also covered in this workshop video:

[![](https://i.imgur.com/WZbVHWC.jpg)](https://www.youtube.com/watch?v=029Vm6PAnrM&t=741s)

## Install

### Prerequisites

- Install [Node.js](https://nodejs.org/en/download/)
- Install [Yarn](https://classic.yarnpkg.com/lang/en/docs/install/)
- Install [Truffle](https://trufflesuite.com/docs/truffle/getting-started/installation/)

### Install Aztec CLI

1. Install Aztec CLI using Yarn:

```shell
yarn global add azteccli
```

1.1 If you are prompted to select an `@aztec/bridge-clients` version, select the latest one.

2. Check the path where yarn binaries are installed:

```shell
yarn global bin
```

You should see something like:
```shell
$ yarn global bin
/{HOME_DIRECTORY}/.yarn/bin
```

3. If not already, add the path to the `PATH` environmental variable to enable access to the yarn binaries (including azteccli) by your terminal.

You can do this by adding:

```
export PATH="/{HOME_DIRECTORY}/.yarn/bin:$PATH"
```

to your `.profile` / `.bashrc` file in your home directory.

> **Note:** Changes on the profile file are not applied until the system restarts. To apply the changes immediately, run:

```shell
source $HOME/.profile
```

4. Check if azteccli is successfully installed:
```shell
azteccli help
```

5. Set Metamask as the wallet to be used by azteccli:

```shell
azteccli conf wallet metamask
```

6. Start the Truffle Dashboard:

```shell
truffle dashboard
```

7. You may now connect your Metamask wallet through the dashboard and start using azteccli by running e.g. `azteccli history`.

For more details on available commands, you can:
- Run `azteccli help`
- Check the [`commands`](https://github.com/critesjosh/azteccli/tree/main/src/commands) directory
- Check the [Aztec CLI repository](https://github.com/critesjosh/azteccli) README

### Development

You can also run the CLI with the latest updates directly from the Github repo.

1. Clone the [repo](https://github.com/critesjosh/azteccli).
2. Install dependencies. `$ yarn`
3. Edit/add new commands in `./src/`.
4. Test your edits by running `./bin/dev [command] [args] [flags]`. (ie `$ ./bin/dev deposit .01`)

Feel free to add features and open a PR. 🙂

## Code Highlights

The workshop video linked at the beginning of this guide is a great walkthrough of the content in this section.

## SDK Version

The version of SDK used in Aztec CLI is specified in its [`package.json`](https://github.com/critesjosh/azteccli/blob/main/package.json#L11):

```json
"dependencies": {
    "@aztec/sdk": "2.1.0-testnet.47", // check for a newer version
    ...
  },
```

The SDK is rapidly developed upon. The list of version numbers can be found in the Versions tab of [@aztec/sdk](https://www.npmjs.com/package/@aztec/sdk) on the npm registry.

## Network Configuration

The networks the Aztec CLI is configured to support are specified in [`network_config.ts`](https://github.com/critesjosh/azteccli/blob/main/src/network_config.ts):

```javascript
let networkConfig: Config = {
  1: {
    rollupProvider: "https://api.aztec.network/aztec-connect-prod/falafel",
    explorerUrl: "https://aztec-connect-prod-explorer.aztec.network/",
  },
  677868: {
    rollupProvider: "https://api.aztec.network/aztec-connect-testnet/falafel/",
    explorerUrl: "https://aztec-connect-testnet-explorer.aztec.network/",
  },
};
```

The network of Chain ID `677868` is the Aztec Testnet. It is a test network forked from Ethereum Mainnet, hence production contracts on Mainnet (e.g. deployed Aztec Connect Bridges) can be accessed on the Testnet under the same addresses.

To access the Testnet with your Metamask, add the network to your wallet using the following parameters:

- **RPC URL:** https://aztec-connect-testnet-eth-host.aztec.network:8545
- **Chain ID:** 677868
- **Currency Symbol:** ETH

## Account Alias

Account registering on the Aztec Network comes with an option to specify a preferred account alias. An example of utilizing so through the Aztec SDK can be seen in Aztec CLI's [`register.ts`](https://github.com/critesjosh/azteccli/blob/main/src/commands/register.ts):

```javascript
public async run(): Promise<void> {
    const { alias, ttpPubKey, time, asset } = this.flags;
    ...
    const controller = await this.sdk.createRegisterController(
      accountKeys.publicKey,
      alias,
      accountKeys.privateKey,
      ...
    );
```

An account alias is an arbitrary string that users could "name" their accounts with. Users transferring assets on the Aztec Network can then specify aliases instead of long public keys as their recipients, simplifying the UX.

## Controllers

The Aztec SDK abstracts away the backend complexities by exposing interactions with the Aztec Network through controllers.

An example of utilizing so can be seen in Aztec CLI's [`register.ts`](https://github.com/critesjosh/azteccli/blob/main/src/commands/register.ts), where a `RegisterController` is first initiated with the arguments gathered:

```javascript
const controller = await this.sdk.createRegisterController(
      accountKeys.publicKey,
      alias,
      accountKeys.privateKey,
      signer.getPublicKey(),
      recoveryPublicKey, // defaults to nothing
      depositValue,
      txFee,
      depositor // defaults to the logged in Ethereum accounts
      // optional feePayer requires an Aztec Signer to pay the fee
    );
```

Functions of the controller are then called to perform a deposit, generate a client-side proof, sign it and send it to the Aztec backend:

```javascript
    if ((await controller.getPendingFunds()) < tokenQuantity) {
      await controller.depositFundsToContract();
      await controller.awaitDepositFundsToContract();
    }

    await controller.createProof();
    await controller.sign();
    let txId = await controller.send();
```

Different controllers for different actions are available in the Aztec SDK. For example, `DepositController` is used for depositing assets from Ethereum and `TransferController` is used for asset transfers on the Aztec Network.
                                        
For more information on available controllers, check the [SDK](https://docs.aztec.network/category/sdk) section of the Aztec Docs and the [`controllers`](https://github.com/AztecProtocol/aztec-connect/tree/master/sdk/src/controllers) directory of the SDK repository.

## Aztec Connect

One of the most interesting use cases of the Aztec SDK is to enable users on the Aztec Network to interact with protocols on Ethereum Layer 1 privately and inexpensively through Aztec Connect Bridges with the [`DefiController`](https://github.com/AztecProtocol/aztec-connect/blob/master/sdk/src/controllers/defi_controller.ts).

For more information, check the [Ethereum Interaction](https://docs.aztec.network/sdk/usage/ethereum-interaction) section of the Aztec Docs and our separate guide on *Getting Started with Aztec Connect Bridges*.

## Resources

### [📓 Aztec Docs](https://docs.aztec.network/)
Documentation of everything Aztec.

### [📝 Aztec CLI](https://github.com/critesjosh/azteccli)
A CLI tool for interacting with the Aztec Network. Powered by Aztec SDK.

### [📝 Aztec SDK Repo](https://github.com/AztecProtocol/aztec-connect/tree/master/sdk)
The repository of the Aztec SDK in production.

### [📝 @aztec/sdk](https://www.npmjs.com/package/@aztec/sdk)
The Aztec SDK npm package on the npm registry.

### [📝 Frontend Boilerplate](https://github.com/Globallager/aztec-frontend-boilerplate)

A sample Web App powered by the Aztec SDK.

### [👾 Discord](https://discord.gg/aztec)

Join the channels:
- [`#💻│aztec-connect`](https://discord.com/channels/563037431604183070/563038059826774017) to discuss the Aztec SDK
- [`#🇨🇴│ethbogota`](https://discord.com/channels/563037431604183070/1021410163221086268) to discuss the ETHBogota Hackathon