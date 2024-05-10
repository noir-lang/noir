---
title: Quickstart
---

The easiest way to start developing on Aztec is simply to click on one of these buttons:

[![One-Click React Starter](/img/codespaces_badges/react_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Freact%2Fdevcontainer.json) [![One-Click HTML/TS Starter](/img/codespaces_badges/vanilla_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Fvanilla%2Fdevcontainer.json) [![One-Click Token Starter](/img/codespaces_badges/token_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Ftoken%2Fdevcontainer.json)

That's it!

This creates a codespace with a prebuilt image containing one of the "Aztec Boxes" and a development network (sandbox). 
- You can develop directly on the codespace, push it to a repo, make yourself at home.
- You can also just use the sandbox that comes with it. The URL will be logged, you just need to use it as your `PXE_URL`.

## Develop Locally

The above method uses Aztec boxes to install the sandbox and clone the repo. You can use it too to get started on your own machine and use your own IDE.

You can also [install the sandbox manually](../sandbox/references/sandbox-reference.md).

### Prerequisites

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))
- Docker (visit [this page of the Docker docs](https://docs.docker.com/get-docker/) on how to install it)

### Run the `npx` script

With the node installation, you now should have `npm` and be able to run `npx` scripts. You can do that by running:

```bash
npx create-aztec-app
```

And follow the instructions. If all goes well, you should now have a development environment running locally on your machine.

You can run `npx create-aztec-app sandbox -h` to start, stop, update and output logs from the sandbox. 

## What's next?

To deploy a smart contract to your sandbox and interact with it using Aztec.js, go to the [next page](aztecjs-getting-started.md).

To skip this and write your first smart contract, go to the [Aztec.nr getting started page](aztecnr-getting-started.md).


