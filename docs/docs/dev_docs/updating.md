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
aztec-up latest
```

2. Updating aztec-nr and individual @aztec dependencies:

Inside your project run:

```shell
cd your/aztec/project
aztec-cli update . --contract src/contract1 --contract src/contract2
```

The sandbox must be running for the update command to work. Make sure it is [installed and running](../dev_docs/cli/sandbox-reference.md).

3. Refer [Migration Notes](../misc/migration_notes.md) on any breaking changes that might affect your dapp

---

There are three components whose versions need to be kept compatible:

1. Aztec Sandbox
2. Aztec CLI
3. `Aztec.nr`, the Noir framework for writing Aztec contracts

All three are using the same versioning scheme and their versions must match. Docker ensures that the sandbox and CLI are always compatible, but you need to update Aztec.nr manually or using `aztec-cli update`.

## Updating Aztec.nr packages

### Automatic update

`aztec-cli` will update your Aztec.nr packages to the appropriate version with the `aztec-cli update` command. Run this command from the root of your project and pass the paths to the folders containing the Nargo.toml files for your projects like so:

```shell
aztec-cli update . --contract src/contract1 --contract src/contract2
```

### Manual update

To update the aztec.nr packages manually, update the tags of the `aztec.nr` dependencies in the `Nargo.toml` file.

```diff
[dependencies]
-aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="aztec-packages-v0.7.5", directory="yarn-project/aztec-nr/aztec" }
+aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
-value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="aztec-packages-v0.7.5", directory="yarn-project/aztec-nr/value-note" }
+value_note = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="yarn-project/aztec-nr/value-note" }
```

Go to the contract directory and try compiling it with `aztec-nargo compile` to verify that the update was successful:

```shell
cd /your/contract/directory
aztec-nargo compile
```

If the dependencies fail to resolve ensure that the tag matches a tag in the [aztec-packages repository](https://github.com/AztecProtocol/aztec-packages/tags).

## Updating `aztec-nargo`

`aztec-nargo` is updated by running:

```bash
aztec-up latest
```
