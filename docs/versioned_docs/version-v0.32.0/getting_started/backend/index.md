---
title: Proving Backend Installation
description: Proving backends offer command line tools for proving and verifying Noir programs. This page describes how to install `bb` as an example.
keywords: [
  Proving
  Backend
  Barretenberg
  bb
  bbup
  Installation
  Terminal
  Command
  CLI
  Version
]
pagination_next: getting_started/hello_noir/index
---

Proving backends each provide their own tools for working with Noir programs, providing utilities such as proof generation, proof verification and smart contracts verifier generation.

For the latest information on tooling provided by each proving backend, installation instructions, Noir version compatibility... you may refer to the proving backends' own documentations.

You can find the full list of proving backends compatible with Noir in [Awesome Noir](https://github.com/noir-lang/awesome-noir/?tab=readme-ov-file#proving-backends).

## Example: Installing `bb`

`bb` is the CLI tool provided by the [Barretenberg proving backend](https://github.com/AztecProtocol/barretenberg) developed by Aztec Labs.

As an example of how a proving backend could be installed, you can install `bb` running the commands below in a terminal.

1. Install `bbup`, Barretenberg CLI's installation script:

    ```bash
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
    ```

2. Reload your terminal shell environment:

    macOS:
    ```bash
    source ~/.zshrc
    ```

    Linux:
    ```bash
    source ~/.bashrc
    ```

3. Install the version of `bb` compatible with your Noir version:

    ```bash
    bbup -v 0.46.1
    ```

4. Check if the installation was successful:

    ```bash
    bb --version
    ```

If it successfully prints the version of `bb` installed, we are ready to start working on [our first Noir program](../hello_noir/index.md).
