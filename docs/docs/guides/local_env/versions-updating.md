---
title: Updating the Sandbox
sidebar_position: 0
---

## Versions

Aztec tools (sandbox, nargo), dependencies (aztec-nr), and sample contracts are constantly being improved.
When developing and referring to example .nr files/snippets, it is helpful to verify the versions of different components (below), and if required keep them in lock-step by [updating](#updating).

### Checking tool versions

:::note
The `aztec-nargo` versions follow `nargo` versions, which is different to the Aztec tool versions.
:::

The latest version of the Aztec tooling is currently `#include_aztec_version` , updating roughly every week.

### Dependency versions

Dependency versions in a contract's `Nargo.toml` file correspond to the `aztec-packages` repository tag `aztec-packages` (filter tags by `aztec`...)

If you get an error like: `Cannot read file ~/nargo/github.com/AztecProtocol/aztec-packages/...`
Check the `git=` github url, tag, and directory.

:::note
The folder structure changed at **0.24.0** from `yarn-project/aztec-nr` to `noir-projects/aztec-nr`. More details [here](https://docs.aztec.network/misc/migration_notes#aztecnr-aztec-nr-contracts-location-change-in-nargotoml)
:::

### Example contract versions

Example contracts serve as a helpful reference between versions of the aztec-nr framework since they are strictly maintained with each release.

Code referenced in the documentation is sourced from contracts within [this directory](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/noir-projects/noir-contracts/contracts).

As in the previous section, the location of the noir contracts moved at version `0.24.0`, from `yarn-project/noir-contracts` before, to `noir-projects/noir-contracts`.

:::tip
Notice the difference between the sample Counter contract from `0.23.0` to `0.24.0` shows the `note_type_id` was added.

```shell
diff ~/nargo/github.com/AztecProtocol/aztec-packages-v0.23.0/yarn-project/noir-contracts/contracts/counter_contract/src/main.nr ~/nargo/github.com/AztecProtocol/aztec-packages-v0.24.0/noir-projects/noir-contracts/contracts/counter_contract/src/main.nr
```

```
57a58
>         note_type_id: Field,
```

:::

### Language server version (aztec-nargo)

The [Noir LSP](https://docs.aztec.network/developers/contracts/main.md#install-noir-lsp-recommended) uses your local version of `aztec-nargo`, and thus also `aztec-nargo compile`.
The path of the former (once installed) can be seen by hovering over "Nargo" in the bottom status bar of VS Code, and the latter via the `which aztec-nargo` command.

:::caution
For Aztec contract files, this should be `aztec-nargo` and for noir-only files this should be `nargo`. Mismatching tools and file types will generate misleading syntax and compiler errors.
:::

This can present confusion when opening older contracts (and dependencies) written in older version of noir, such as:

- Logs filled with errors from the dependencies
- Or the LSP fails (re-runs automatically then stops)
  The second point requires a restart of the extension, which you can trigger with the command palette (Ctrl + Shift + P) and typing "Reload Window".

## Updating

### Steps to keep up to date

1. Update the Aztec sandbox to the latest version (includes `aztec-nargo`, pxe, etc):

```shell
aztec-up
```

To set `VERSION` for a particular git tag, eg for [aztec-package-v**0.35.0**](https://github.com/AztecProtocol/aztec-packages/tree/aztec-packages-v0.35.0)

```shell
VERSION=0.35.0 aztec-up
```

2. Update aztec-nr and individual @aztec dependencies:

Inside your project run:

```shell
cd your/aztec/project
aztec update . --contract src/contract1 --contract src/contract2
```

The sandbox must be running for the update command to work. Make sure it is [installed and running](../../reference/sandbox_reference/sandbox-reference.md).

Follow [updating Aztec.nr packages](#updating-aztecnr-packages) and [updating JavaScript packages](#updating-aztecjs-packages) guides.

3. Refer to [Migration Notes](../../migration_notes.md) on any breaking changes that might affect your dapp

---

There are four components whose versions need to be kept compatible:

1. Aztec Sandbox
2. aztec-nargo
3. `Aztec.nr`, the Noir framework for writing Aztec contracts

First three are packaged together in docker and are kept compatible by running `aztec-up`.
But you need to update your Aztec.nr version manually or using `aztec update`.

## Updating Aztec.nr packages

### Automatic update

You can update your Aztec.nr packages to the appropriate version with the `aztec update` command. Run this command from the root of your project and pass the paths to the folders containing the Nargo.toml files for your projects like so:

```shell
aztec update . --contract src/contract1 --contract src/contract2
```

### Manual update

To update the aztec.nr packages manually, update the tags of the `aztec.nr` dependencies in the `Nargo.toml` file.

```diff
[dependencies]
-aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="aztec-packages-v0.7.5", directory="noir-projects/aztec-nr/aztec" }
+aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="noir-projects/aztec-nr/aztec" }
-value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="aztec-packages-v0.7.5", directory="noir-projects/aztec-nr/value-note" }
+value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="noir-projects/aztec-nr/value-note" }
```

Go to the contract directory and try compiling it with `aztec-nargo compile` to verify that the update was successful:

```shell
cd /your/contract/directory
aztec-nargo compile
```

If the dependencies fail to resolve ensure that the tag matches a tag in the [aztec-packages repository](https://github.com/AztecProtocol/aztec-packages/tags).

## Updating Aztec.js packages

To update Aztec.js packages, go to your `package.json` and replace the versions in the dependencies.

```diff
[dependencies]
-"@aztec/accounts": "0.7.5",
+"@aztec/accounts": "#include_aztec_short_version",
-"@aztec/noir-contracts.js": "0.35.1",
+"@aztec/accounts": "#include_aztec_short_version",
```

## Updating `aztec-nargo`

As mentioned in the tl;dr, `aztec-nargo` is updated as part of updating the whole sandbox via:

```bash
aztec-up
```

The version of aztec-nargo that comes with a particular version of the Aztec sandbox can be seen in the monorepo. Eg tag: aztec-packages-v0.35.0 contains aztec-nargo [v0.27.0](https://github.com/AztecProtocol/aztec-packages/blob/aztec-packages-v0.35.0/noir/noir-repo/Cargo.toml#L44).

Set VERSION to specify the desired Aztec sandbox version, eg monorepo tag suffix [0.35.0](https://github.com/AztecProtocol/aztec-packages/tree/aztec-packages-v0.35.0) (to have `aztec-nargo` v0.27.0).

```bash
VERSION=<tag-suffix> aztec-up
```

Note: Being under highly active development it is NOT recommended to specify, `master`, due to the increased effort to align tooling, dependencies, and example code syntax.
