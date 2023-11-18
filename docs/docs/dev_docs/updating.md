---
title: Updating
---

## TL;DR

1. **Updating the sandbox:**

- If you installed sandbox via docker, run:

```shell
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

- If you have installed via an npm package then step 3 handles the update.

2. **Updating Aztec-CLI:**

- The above command also downloads the aztec-cli if a node package version of the CLI isn't found locally.
- If you have globally installed the CLI previously, then run:

```shell
npm install -g @aztec/cli
```

(replace with `yarn` or your node package version manager tool).

- If you have aztec-cli listed as a local dependency in your project's `package.json`, then step 3 handles the update.

:::info

You can install the CLI globally, but it is recommended that you install the CLI as a local dependency in your project. This will make it easier to keep the CLI version in sync with the sandbox version.

:::

1. **Updating aztec-nr and individual @aztec dependencies:**

Inside your project run:

```shell
cd your/aztec/project
npx @aztec/cli@latest update . --contract src/contract1 --contract src/contract2
```

The sandbox must be running for the update command to work unless there the project defines `@aztec/aztec-sandbox` as a dependency, in which case the command will compare against the version listed in `package.json`.

---

There are three components whose versions need to be kept compatible:

1. Aztec Sandbox,
2. Aztec CLI,
3. Noir framework for Aztec contracts `aztec.nr`.

All three are using the same versioning scheme and their versions must match.

## Updating Aztec Sandbox

To update the sandbox to the latest version, simply run the curl command we used for installation again:

```shell
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

This will also update the CLI if a node package version of the CLI isn't found locally.

## Updating Aztec CLI

### npm

:::info

You can install the CLI globally, but it is recommended that you install the CLI as a local dependency in your project. This will make it easier to keep the CLI version in sync with the sandbox version.

:::

If the latest version was used when updating the sandbox then we can simply run the following command to update the CLI:

```shell
npm install --save-dev @aztec/cli
```

If a specific version was set for the sandbox then we need to install the CLI with the same version:

```shell
npm install --save-dev @aztec/cli@$SANDBOX_VERSION
```

E.g.:

```shell
npm install --save-dev @aztec/cli@#include_aztec_short_version
```

### Docker

If you don't have the CLI installed globally via package manager or locally in your npm project, then you can update it by running the sandbox installation command again:

```shell
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

## Updating Aztec.nr packages

### Automatic update

`aztec-cli` will update your Aztec.nr packages to the appropriate version with the `aztec-cli update` command. Run this command from the root of your project and pass the paths to the folders containing the Nargo.toml files for your projects like so:

```shell
aztec-cli update . --contract src/contract1 --contract src/contract2
```

### Manual update

Finally we need to update the Noir framework for Aztec contracts.
We need to install a version compatible with our `nargo` and Sandbox.

To update the framework we will update a tag of the `aztec.nr` dependency in the `Nargo.toml` file to the `SANDBOX_VERSION` from above.
Find all the dependencies pointing to the directory within `aztec.nr` framework and update the corresponding tag.
E.g.:

```diff
[dependencies]
-aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="aztec-packages-v0.7.5", directory="yarn-project/aztec-nr/aztec" }
+aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
-value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="aztec-packages-v0.7.5", directory="yarn-project/aztec-nr/value-note" }
+value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="yarn-project/aztec-nr/value-note" }
```

Go to the project directory and try compiling it with `aztec-cli` to verify that the update was successful:

```shell
cd /your/project/root
aztec-cli compile ./
```

If the dependencies fail to resolve ensure that the tag matches a tag in the [aztec-packages repository](https://github.com/AztecProtocol/aztec-packages/tags).

## Updating `nargo`

Nargo is not strictly required, but you may want to use it for the LSP or testing. More info [here](./getting_started/aztecnr-getting-started.md#install-nargo-recommended).

<InstallNargoInstructions />
