---
title: Barretenberg Installation
description: bb is a command line tool for interacting with Aztec's proving backend Barretenberg. This page is a quick guide on how to install `bb`
keywords: [
  Barretenberg
  bb
  Installation
  Terminal Commands
  Version Check
  Nightlies
  Specific Versions
  Branches
]
pagination_next: getting_started/hello_noir/index
---

`bb` is the CLI tool for generating and verifying proofs for Noir programs using the Barretenberg proving library. It also allows generating solidity verifier contracts for which you can verify contracts which were constructed using `bb`.

## Installing `bb`

Open a terminal on your machine, and write:

##### macOS (Apple Silicon)

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
source ~/.zshrc
bbup -v 0.41.0
```

##### macOS (Intel)

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
source ~/.zshrc
bbup -v 0.41.0
```

##### Linux (Bash)

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
source ~/.bashrc
bbup -v 0.41.0
```

Now we're ready to start working on [our first Noir program!](../hello_noir/index.md)
