---
title: Local Devnet
---

# Setting up a local Aztec development network

You can set up a local Aztec network for development purposes using docker compose scripts.

## Steps

### Install Docker

From [here](https://docs.docker.com/get-docker/).

### Start the network

Run the relevant docker compose file for your needs. Make sure you have at least 8 GB of RAM allocated to Docker--the more you can allocate, the better.

:::info
Mac M1s run this on an emulator so they will be slower.
:::

#### Basic Network

For a simple, fresh Ethereum network + Aztec sequencer without any bridge contracts, run

```bash
curl -s https://raw.githubusercontent.com/AztecProtocol/dev-rel/main/docker-compose.dev.yml | docker-compose -f - up --force-recreate
```

This will be useful for testing basic functionality of the Aztec network like deposits, withdrawals, account registrations, account migrations, account recovery and asset transfers.

#### Mainnet fork with bridge contracts

Or for a an Ethereum fork of mainnet along with all of the mainnet Aztec bridge contracts deployed + Aztec sequencer, run

```bash
curl -s https://raw.githubusercontent.com/AztecProtocol/dev-rel/main/docker-compose.fork.yml  | CHAIN_ID=3567 FORK_URL=https://mainnet.infura.io/v3/{infura_api_key} docker-compose -f - up --force-recreate
```

This network will be useful for testing functionality associated with bridge contracts and interacting with other contracts/protocols that are on Ethereum.

Note the `CHAIN_ID=3567` and `FORK_URL=https://mainnet.infura.io/v3/{infura_api_key}` when running the mainnet fork. Specifying chain id 3567 will run the contract deployment script so that all of the bridge contracts that are on mainnet will be deployed to this dev network.

:::note
This mainnet fork deployment takes considerably longer than the basic devnet.
:::

These scripts will get the docker compose file from a github gist and run it.

You can download the [docker-compose.dev.yml here](https://raw.githubusercontent.com/AztecProtocol/dev-rel/main/docker-compose.dev.yml) or [docker-compose.fork.yml here](https://raw.githubusercontent.com/AztecProtocol/dev-rel/main/docker-compose.fork.yml).

### Check the sequencer

Check that the aztec sequencer (falafel) is running by visiting this url in your browser at [http://localhost:8081/status](http://localhost:8081/status).

Once it is up and running, you can use it to run testing scripts against or point your web application to it for testing.

### Connect the SDK

For a web application, point Metamask to the locally running Ethereum network (details below).

Connect the Aztec SDK to `http://localhost:8081`.

You can also interact with your local aztec network directly via the [CLI](https://github.com/critesjosh/azteccli#development) or the [frontend boilerplate](https://github.com/AztecProtocol/aztec-frontend-boilerplate).

### Deploy custom bridges

You can deploy your own bridge contracts to the mainnet fork devnet.

Here is an [example script](https://gist.github.com/critesjosh/a53aa1afc5042a8dfbba4d379356314f#file-addressregistrydeployment-s-sol) that shows how you would deploy the [AddressRegistry.sol](https://github.com/critesjosh/aztec-connect-starter/blob/nft-bridge/src/bridges/registry/AddressRegistry.sol) contract in the [aztec-connect-bridges repo](https://github.com/AztecProtocol/aztec-connect-bridges).

Set these local environment variables before running the deployment script.

```bash
export NETWORK=None
export SIMULATE_ADMIN=false # to broadcast your deployment to the devnet
export LISTER_ADDRESS=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

Check the `rollupProcessorContract` address on your local Aztec sequencer at [http://localhost:8081/status](http://localhost:8081/status) and export it as an environment variable as well.

For example:

```bash
export ROLLUP_PROCESSOR_ADDRESS=0xDA437738D931677e83a480C9c397d2d0A473c209
```

Then run the deployment script.

```bash
forge script AddressRegistryDeployment --sig "deployAndList()" --broadcast --fork-url http://localhost:8545 --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

Run this from the aztec connect bridges repo containing the deployment script. The private key here is associated with the first [anvil](https://book.getfoundry.sh/anvil/) account. It has enough ETH and permission to deploy and list new bridges.

Make sure you run `yarn setup` in the aztec-connect-bridges repo to set up forge for this repo.

See more example deployment scripts in the [aztec-connect-bridges repo here](https://github.com/AztecProtocol/aztec-connect-bridges/tree/master/src/deployment).

#### Update runtime config

After your bridge contract is deployed to your local Ethereum network, you need to update the Aztec sequencer (falafel) with information about how to interact with the new bridge.

Do this by appending the appropriate info for your bridge to the "bridgeConfigs" array in [config.json](https://github.com/AztecProtocol/dev-rel/blob/main/falafel-runtime-config.json) and sending it as a PATCH request to http://localhost:8081/runtime-config. You will need to set a couple of headers in the request for this to work: `server-auth-token`: `!changeme#` and `Content-Type`: `application/json`.

For the AddressRegistry, it might looks like

```json
{
  "bridgeConfigs": [
    // ... other configs here
    {
      "numTxs": 1,
      "gas": 120000,
      "bridgeAddressId": 14,
      "permittedAssets": [0]
    }
  ],
  "feePayingAssetIds": [0, 1]
}
```

where `numTxs` is the number of transactions per batch for the bridge. `gas` is the max gas that a bridge call requires. The rollup contract needs to know how much gas to send with a transaction. `bridgeAddressId` will be the `id` of the bridge that you deployed. and `permittedAssets` are the [asset ids](../glossary#asset-ids) of the assets that can be sent to the bridge (you can check what assets are currently configured by checking the /status endpoint).

To get you up and running, here is a [postman collection](https://raw.githubusercontent.com/AztecProtocol/dev-rel/main/local-devnet-postman-collection.json) that is plug and play. You learn how to import postman collections here [here](https://learning.postman.com/docs/getting-started/importing-and-exporting-data/).

### Connect Metamask

Add anvil accounts and network information to Metamask.

These devnets start the local Ethereum network at `http://localhost:8545`, so you will have to update the info associated with this network in Metamask.

Chain id is `31337` or `3567` (when running mainnet fork devnet).

[Anvil](https://book.getfoundry.sh/anvil/) private keys:

```
(0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
(1) 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
(2) 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
(3) 0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
(4) 0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a
(5) 0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba
(6) 0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e
(7) 0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356
(8) 0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97
(9) 0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6
Mnemonic:          test test test test test test test test test test test junk
Derivation path:   m/44'/60'/0'/0/
```

If you have questions, please reach out on [Discord](https://discord.com/invite/UDtJr9u).
