---
title: Alternative Installations
description: bb is a command line tool for interacting with Aztec's proving backend Barretenberg. This page is a quick guide on how to install `bb` using the simplest and most straightforward method, bbup.
keywords: [
  Barretenberg
  bb
  bbup
  Installation
  Specific Versions
  Binaries
  WSL for Windows
  Linux
  Uninstalling bb
]
sidebar_position: 1
---

## Encouraged Installation Method: bbup

bbup is the recommended tool for installing Barretenberg, simplifying the process of obtaining binaries. It offers versatile options for installing specific versions.

### Installing bbup

Start by installing `bbup` with this command:

```sh
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
```

### Fetching Binaries

With `bbup`, you can easily manage different `bb` versions:
  
- **Specific Version**: Install a specific version of `bb`.
  ```sh
  bbup --version <version>
  ```

### Compiling from Source

`bbup` is made to fetch pre-compiled binaries of Barretenberg, some users might prefer or require building `bb` from source to meet specific needs or for development purposes. However, building `bb` from source is a more involved process and is not supported directly through `bbup` due to the complex environment setup and potential compatibility issues with serialization code in other components like Nargo.

If you choose to manually compile `bb` from source, refer to the detailed build instructions available in the [Barretenberg repository on GitHub](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg).

## Installation on Windows

The default backend for Noir (Barretenberg) doesn't provide Windows binaries at this time. For that reason, installation on native Windows systems is not supported. However, it is available by using Windows Subsystem for Linux (WSL).

Step 1: Follow the instructions [here](https://learn.microsoft.com/en-us/windows/wsl/install) to install and run WSL.

Step 2: Follow the [bbup instructions](#encouraged-installation-method-bbup).

## Uninstalling `bb`

To uninstall `bb` installed via bbup:

```bash
rm -r ~/.bb 
```
