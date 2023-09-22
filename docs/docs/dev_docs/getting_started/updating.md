---
title: Updating
---

There are 4 components whose versions need to be kept compatible:
1. Aztec Sandbox,
2. Aztec CLI,
3. Noir compiler `nargo`,
4. Noir framework for Aztec contracts `aztec.nr`.

Aztec Sandbox, Aztec CLI and `aztec.nr` are using the same versioning scheme and their versions must match.
The Noir compiler `nargo` has its own versioning scheme and its version must match the compatible nargo version specified in Sandbox.

## Updating Aztec Sandbox
To update the sandbox to the latest version, simply run the curl command we used for installation again:
```shell
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

It will download and start the latest version of sandbox.

If you would like to use a fixed version of the sandbox, you can export the `SANDBOX_VERSION` environmental variable.
If you are unsure what version to use go to [aztec-packages repository](https://github.com/AztecProtocol/aztec-packages/releases) and choose the `aztec-packages` release based on the changelog.

Then set the `SANDBOX_VERSION` environmental variable to the version you want to use. E.g.:
```shell
export SANDBOX_VERSION=#include_aztec_short_version
```

Now when you run the curl command it will use the version you specified.
To verify that it's the case check the console output of the curl command.
You should see the following line:
```
Setting up Aztec Sandbox v#include_aztec_short_version (nargo #include_noir_version), please stand by...
```

Alternatively you can open a new terminal and use aztec-cli to get the version.

#include_code node-info yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

The sandbox version should be the same as the one we chose by setting the `SANDBOX_VERSION` environmental variable.

## Updating Aztec CLI
If the latest version was used when updating the sandbox then we can simply run the following command to update the CLI:
```shell
npm install -g @aztec/cli
```

If a specific version was set for the sandbox then we need to install the CLI with the same version:
```shell
npm install -g @aztec/cli@$SANDBOX_VERSION
```

E.g.:
```shell
npm install -g @aztec/cli@#include_aztec_short_version
```

## Updating Noir compiler
Now we need to update the Noir compiler `nargo` to the version compatible with the sandbox.
Use `aztec-cli` to get it:
#include_code node-info yarn-project/end-to-end/src/cli_docs_sandbox.test.ts bash

Then we install the `Compatible Nargo Version` with (replace `COMPATIBLE_NARGO_VERSION` with the version from the previous command):
```shell
noirup -v COMPATIBLE_NARGO_VERSION
```

## Updating Noir framework
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

Go to the project directory and try compiling it with `aztec-cli`` to verify that the update was successful:
```shell
cd /your/project/root
aztec-cli compile ./
```

If the dependencies fail to resolve ensure that the tag matches a tag in the [aztec-packages repository](https://github.com/AztecProtocol/aztec-packages/tags).
