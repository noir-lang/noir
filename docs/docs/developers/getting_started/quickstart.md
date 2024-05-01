---
title: Quickstart
---

In this guide, you will

1. Set up the Aztec sandbox (local development environment) locally
2. Install the Aztec development kit
3. Use Aztec.js to deploy an example contract that comes with the sandbox
4. Use Aztec.js to interact with the contract you just deployed

... in less than 10 minutes.

## Prerequisites

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))

## Install Docker

Aztec tooling requires the Docker daemon to be running, and this is easily achieved via Docker Desktop. See [this page of the Docker docs](https://docs.docker.com/get-docker/) for instructions on how to install Docker Desktop for your operating system.
Note: if installing via Docker Desktop, you do NOT need to keep the application open at all times (just Docker daemon).

Installing and running the Docker daemon can also be achieved by installing Docker Engine, see [these instructions](https://docs.docker.com/engine/install/).

However installed, ensure Docker daemon is running. See [start Docker daemon](https://docs.docker.com/config/daemon/start/).

### Note on Linux

If you are running Linux, you will need to set the context (because Docker Desktop runs in a VM by default). See [this page](https://docs.docker.com/desktop/faqs/linuxfaqs/#what-is-the-difference-between-docker-desktop-for-linux-and-docker-engine) for more information. You can do this by running:

```bash
docker context use default
```

## Install the Sandbox

You can run the Sandbox using Docker.

To install the latest Sandbox version, run:

```bash
bash -i <(curl -s install.aztec.network)
```

> If Docker has been installed on your linux server but you encounter the error "Docker is not running. Please start Docker and try again". If you're encountering this issue, it's likely because Docker is running with root user privileges. In such cases, consider [managing Docker as a non-root user](https://docs.docker.com/engine/install/linux-postinstall/#manage-docker-as-a-non-root-user) to resolve the problem.


This will install the following:

- **aztec** - launches various infrastructure subsystems (sequencer, prover, pxe, etc).
- **aztec-nargo** - aztec's build of nargo, the noir compiler toolchain.
- **aztec-sandbox** - a wrapper around docker-compose that launches services needed for sandbox testing.
- **aztec-up** - a tool to upgrade the aztec toolchain to the latest, or specific versions.
- **aztec-builder** - A useful tool for projects to generate ABIs and update their dependencies.


Once these have been installed, to start the sandbox, run:

```bash
aztec-sandbox
```

This will attempt to run the Sandbox on ` localhost:8080`, so you will have to make sure nothing else is running on that port or change the port defined in `./.aztec/docker-compose.yml`. Running the installation again will overwrite any changes made to the `docker-compose.yml`.

**Congratulations, you have just installed and run the Aztec Sandbox!**

```bash                
     /\        | |
    /  \    ___| |_ ___  ___
   / /\ \  |_  / __/ _ \/ __|
  / ____ \  / /| ||  __/ (__
 /_/___ \_\/___|\__\___|\___|

```

In the terminal, you will see some logs:
1. Sandbox version
2. Contract addresses of rollup contracts
3. PXE (private execution environment) setup logs
4. Initial accounts that are shipped with the sandbox and can be used in tests

## What's next?

To deploy a smart contract to your sandbox and interact with it using Aztec.js, go to the [next page](aztecjs-getting-started.md).

To skip this and write your first smart contract, go to the [Aztec.nr getting started page](aztecnr-getting-started.md).

