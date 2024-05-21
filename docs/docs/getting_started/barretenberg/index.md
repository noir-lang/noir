---
title: Barretenberg Installation
description:
  `bb` is a command line tool for interacting with Aztec's proving backend Barretenberg. This page is a quick guide on how to install `bb`
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
mkdir -p $HOME/.barretenberg && \
curl -o ./barretenberg-aarch64-apple-darwin.tar.gz -L https://github.com/AztecProtocol/aztec-packages/releases/download/aztec-packages-v0.38.0/barretenberg-aarch64-apple-darwin.tar.gz && \
tar -xvf ./barretenberg-aarch64-apple-darwin.tar.gz -C $HOME/.barretenberg/ && \
echo 'export PATH=$PATH:$HOME/.barretenberg/' >> ~/.zshrc && \
source ~/.zshrc
```

##### macOS (Intel)

```bash
mkdir -p $HOME/.barretenberg && \
curl -o ./barretenberg-x86_64-apple-darwin.tar.gz -L https://github.com/AztecProtocol/aztec-packages/releases/download/aztec-packages-v0.38.0/barretenberg-x86_64-apple-darwin.tar.gz && \
tar -xvf ./barretenberg-x86_64-apple-darwin.tar.gz -C $HOME/.barretenberg/ && \
echo 'export PATH=$PATH:$HOME/.barretenberg/' >> ~/.zshrc && \
source ~/.zshrc
```

##### Linux (Bash)

```bash
mkdir -p $HOME/.barretenberg && \
curl -o ./barretenberg-x86_64-linux-gnu.tar.gz -L https://github.com/AztecProtocol/aztec-packages/releases/download/aztec-packages-v0.38.0/barretenberg-x86_64-linux-gnu.tar.gz && \
tar -xvf ./barretenberg-x86_64-linux-gnu.tar.gz -C $HOME/.barretenberg/ && \
echo -e 'export PATH=$PATH:$HOME/.barretenberg/' >> ~/.bashrc && \
source ~/.bashrc
```

Now we're ready to start working on [our first Noir program!](../hello_noir/index.md)
