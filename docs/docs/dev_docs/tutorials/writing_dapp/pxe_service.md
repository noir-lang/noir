# Connecting to the RPC Server

As an app developer, the [Private Execution Environment (PXE)](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/pxe) interface provides you with access to the user's accounts and their private state, as well as a connection to the network for accessing public global state.

During the Sandbox phase, this role is fulfilled by the [Aztec Sandbox](../../getting_started/sandbox.md), which runs a local RPC Server and an Aztec Node, both connected to a local Ethereum development node like Anvil. The Sandbox also includes a set of pre-initialized accounts that you can use from your app.

In this section, we'll connect to the Sandbox from our project.

## Create RPC client

We'll use the `createPXEClient` function from `aztec.js` to connect to the Sandbox, which by default runs on `localhost:8080`. To test the connection works, we'll request and print the node's chain id.

Let's create our first file `src/index.mjs` with the following contents:

#include_code all yarn-project/end-to-end/src/sample-dapp/connect.mjs javascript

Run this example as `node src/index.mjs` and you should see the following output:

```
Connected to chain 31337
```

:::info
Should the above fail due to a connection error, make sure the Sandbox is running locally and on port 8080.
:::

## Load user accounts

With our connection to the RPC server, let's try loading the accounts that are pre-initialized in the Sandbox:

#include_code showAccounts yarn-project/end-to-end/src/sample-dapp/index.mjs javascript

Run again the above, and we should see:

```
User accounts:
0x0c8a6673d7676cc80aaebe7fa7504cf51daa90ba906861bfad70a58a98bf5a7d
0x226f8087792beff8d5009eb94e65d2a4a505b70baf4a9f28d33c8d620b0ba972
0x0e1f60e8566e2c6d32378bdcadb7c63696e853281be798c107266b8c3a88ea9b
```

## Next steps

With a working connection to the RPC Server, let's now setup our application by [compiling and deploying our contracts](./contract_deployment.md).
