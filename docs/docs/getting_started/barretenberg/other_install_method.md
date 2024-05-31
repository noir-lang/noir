---
title: Alternative Installations
description: bb is a command line tool for interacting with Aztec's proving backend Barretenberg. This page is a quick guide on how to install `bb` using the simplest and most straightforward method, bbup.
keywords: [
  Barretenberg
  bb
  bbup
  Installation
  Nightlies
  Specific Versions
  Branches
  Binaries
  Compiling from Source
  WSL for Windows
  Linux
  Uninstalling bb
]
sidebar_position: 1
---

## Encouraged Installation Method: bbup

bbup is the recommended tool for installing Barretenberg, simplifying the process of obtaining binaries or compiling from source. It offers versatile options for installing specific versions, nightly builds, or custom source compilations.

### Installing bbup

Start by installing `bbup` with this command:

```sh
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
```

### Fetching Binaries

With `bbup`, you can easily manage different `bb` versions:

- **Nightly Version**: Install the latest nightly build.
  ```sh
  bbup --version nightly
  ```
  
- **Specific Version**: Install a specific version of `bb`.
  ```sh
  bbup --version <version>
  ```

### Compiling from Source

`bbup` also enables compiling `bb` from various sources:

- **From a Specific Branch**: Install from the latest commit on a branch.

  ```sh
  bbup --branch <branch-name>
  ```
  
- **From a Fork**: Install from the main branch of a fork.

  ```sh
  bbup --repo <username/repo>
  ```
  
- **From a Specific Branch in a Fork**: Install from a specific branch in a fork.

  ```sh
  bbup --repo <username/repo> --branch <branch-name>
  ```
  
- **From a Specific Pull Request**: Install from a specific PR.

  ```sh
  bbup --pr <pr-number>
  ```
  
- **From a Specific Commit**: Install from a specific commit.

  ```sh
  bbup -C <commit-hash>
  ```
  
- **From Local Source**: Compile and install from a local directory.

  ```sh
  bbup --path ./path/to/local/source
  ```

## Installation on Windows

The default backend for Noir (Barretenberg) doesn't provide Windows binaries at this time. For that reason, installation on native Windows systems is not supported. However, it is available by using Windows Subsystem for Linux (WSL).

Step 1: Follow the instructions [here](https://learn.microsoft.com/en-us/windows/wsl/install) to install and run WSL.

Step 2: Follow the [bbup instructions](#encouraged-installation-method-bbup).

## Uninstalling `bb`

To uninstall `bb` installed via bbup:

```bash
rm -r ~/.bb 
```

This ensures that all installed binaries, configurations, and cache related to `bb` are fully removed from your system.
