---
title: Standalone Noir Installation
description: There are different ways to install Nargo, the one-stop shop and command-line tool for developing Noir programs. This guide explains how to specify which version to install when using noirup, and using WSL for windows.
keywords: [
    Installation
    Nargo
    Noirup
    Binaries
    Compiling from Source
    WSL for Windows
    macOS
    Linux
    Nix
    Direnv
    Uninstalling Nargo
  ]
sidebar_position: 2
---

Noirup is the endorsed method for installing Nargo, streamlining the process of fetching binaries or compiling from source. It supports a range of options to cater to your specific needs, from nightly builds and specific versions to compiling from various sources.

### Installing Noirup

First, ensure you have `noirup` installed:

```sh
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
```

### Fetching Binaries

With `noirup`, you can easily switch between different Nargo versions, including nightly builds:

- **Nightly Version**: Install the latest nightly build.

  ```sh
  noirup --version nightly
  ```

- **Specific Version**: Install a specific version of Nargo.

  ```sh
  noirup --version <version>
  ```

### Compiling from Source

`noirup` also enables compiling Nargo from various sources:

- **From a Specific Branch**: Install from the latest commit on a branch.

  ```sh
  noirup --branch <branch-name>
  ```

- **From a Fork**: Install from the main branch of a fork.

  ```sh
  noirup --repo <username/repo>
  ```

- **From a Specific Branch in a Fork**: Install from a specific branch in a fork.

  ```sh
  noirup --repo <username/repo> --branch <branch-name>
  ```

- **From a Specific Pull Request**: Install from a specific PR.

  ```sh
  noirup --pr <pr-number>
  ```

- **From a Specific Commit**: Install from a specific commit.

  ```sh
  noirup -C <commit-hash>
  ```

- **From Local Source**: Compile and install from a local directory.

  ```sh
  noirup --path ./path/to/local/source
  ```

## Installation on Windows

The default backend for Noir (Barretenberg) doesn't provide Windows binaries at this time. For that reason, Noir cannot be installed natively. However, it is available by using Windows Subsystem for Linux (WSL).

Step 1: Follow the instructions [here](https://learn.microsoft.com/en-us/windows/wsl/install) to install and run WSL.

step 2: Follow the [Noirup instructions](#installing-noirup).

## Setting up shell completions

Once `nargo` is installed, you can [set up shell completions for it](setting_up_shell_completions.md).

## Uninstalling Nargo

If you installed Nargo with `noirup`, you can uninstall Nargo by removing the files in `~/.nargo`, `~/nargo`, and `~/noir_cache`. This ensures that all installed binaries, configurations, and cache related to Nargo are fully removed from your system.

```bash
rm -r ~/.nargo
rm -r ~/nargo
rm -r ~/noir_cache
```
