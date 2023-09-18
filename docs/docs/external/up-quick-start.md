# Quick start

:::info
This guide is meant to be included in the sandbox.aztec.network site and not in the main documentation.
:::

To interact with the sandbox, install the [Aztec CLI](../dev_docs/cli/main.md):

`npm install -g @aztec/cli`

The sandbox is preloaded with two [accounts](../concepts/foundation/accounts/main.md), let's assign them as Alice and Bob:

#include_code declare-accounts yarn-project/end-to-end/src/guides/up_quick_start.sh bash noTitle,noLineNumbers,noSourceLink

Start by deploying a token [contract](../concepts/foundation/contracts.md), initializing it and minting tokens to Alice:

#include_code deploy yarn-project/end-to-end/src/guides/up_quick_start.sh bash noTitle,noLineNumbers,noSourceLink

We can check Alice's private token balance by querying the contract:

#include_code get-balance yarn-project/end-to-end/src/guides/up_quick_start.sh bash noTitle,noLineNumbers,noSourceLink

We can have Alice privately transfer tokens to Bob. Only Alice and Bob will know what's happened. Here, we use Alice's private key to [send a transaction](../concepts/foundation/transactions.md) to transfer tokens to Bob, and check the result:

#include_code transfer yarn-project/end-to-end/src/guides/up_quick_start.sh bash noTitle,noLineNumbers,noSourceLink

To learn more, check out an extended version of this quick start [on our docs](../dev_docs/getting_started/cli.md).
