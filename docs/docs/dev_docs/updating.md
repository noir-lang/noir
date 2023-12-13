---
title: Updating
---

:::info
The `@aztec/aztec-sandbox` and `@aztec/cli` packages published to npm **should not be used**, in favor of Docker. If you've installed the sandbox or the CLI via NPM, **uninstall** them and remove them from your project dependencies and [install via Docker](./cli/sandbox-reference.md#with-docker).

<Tabs>
  <TabItem value="yarn" label="yarn" default>
    
<code>
yarn global remove @aztec/aztec-sandbox @aztec/cli
</code>

  </TabItem>
  <TabItem value="npm" label="npm">

<code>
npm -g uninstall @aztec/aztec-sandbox @aztec/cli
</code>

   </TabItem>
</Tabs>

:::

## TL;DR

1. Updating the sandbox and CLI:

```shell
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

2. Updating aztec-nr and individual @aztec dependencies:

Inside your project run:

```shell
cd your/aztec/project
aztec-cli update . --contract src/contract1 --contract src/contract2
```

The sandbox must be running for the update command to work.

---

There are three components whose versions need to be kept compatible:

1. Aztec Sandbox
2. Aztec CLI
3. Noir framework for Aztec contracts `aztec.nr`

All three are using the same versioning scheme and their versions must match. Docker ensures that the sandbox and CLI are always compatible, but you need to update Aztec.nr manually or using `aztec-cli update`.

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
