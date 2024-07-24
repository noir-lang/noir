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

Proving backends each provides their own tools for working with Noir programs, such as generating proofs, verifying proofs and generating verifier smart contracts.

For the latest information of each proving backend's tooling provisions, installation instructions, Noir version compatibility, etc., you may refer to the proving backends' own documentations.

You can find the full list of proving backends compatible with Noir in [Awesome Noir](https://github.com/noir-lang/awesome-noir/?tab=readme-ov-file#proving-backends).

## Example: Installing `bbup` and `bb`

`bb` is the CLI tool provided by the [Barretenberg proving backend](https://github.com/AztecProtocol/barretenberg) developed by Aztec Labs.

As an example of installing a proving backend, you can install `bbup` (the `bb` installation script) by opening the terminal and run:

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
```

Refresh your terminal paths:

macOS:
```bash
source ~/.zshrc
```

Linux:
```bash
source ~/.bashrc
```

Then install the version of `bb` compatible with your Noir version:

```bash
bbup -v 0.45.1
```

You can check if the installation was successful by running:
```bash
bb --version
```

If it successfully prints the version of `bb` installed, we're ready to start working on [our first Noir program!](../hello_noir/index.md)
