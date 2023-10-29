---
title: Migration notes
description: Read about migration notes from previous versions, which could solve problems while updating
keywords: [Noir, notes, migration, updating, upgrading]
---

Noir is in full-speed development. Things break fast, wild, and often. This page attempts to leave some notes on errors you might encounter when upgrading and how to resolve them until proper patches are built.

## ≥0.14

The index of the [for loops](./language_concepts/02_control_flow.md#loops) is now of type `u64` instead of `Field`. An example refactor would be:

```rust
for i in 0..10 {
    let i = i as Field;
}
```

## ≥v0.11.0 and Nargo backend

From this version onwards, Nargo starts managing backends through the `nargo backend` command. Upgrading to the versions per usual steps might lead to:

### `backend encountered an error`

This is likely due to the existing locally installed version of proving backend (e.g. barretenberg) is incompatible with the version of Nargo in use.

To fix the issue:

1. Uninstall the existing backend

```bash
nargo backend uninstall acvm-backend-barretenberg
```

You may replace _acvm-backend-barretenberg_ with the name of your backend listed in `nargo backend ls` or in ~/.nargo/backends.

2. Reinstall a compatible version of the proving backend.

If you are using the default barretenberg backend, simply run:

```
nargo prove
```

with you Noir program.

This will trigger the download and installation of the latest version of barretenberg compatible with your Nargo in use.

### `backend encountered an error: illegal instruction`

On certain Intel-based systems, an `illegal instruction` error may arise due to incompatibility of barretenberg with certain CPU instructions.

To fix the issue:

1. Uninstall the existing backend

```bash
nargo backend uninstall acvm-backend-barretenberg
```

You may replace _acvm-backend-barretenberg_ with the name of your backend listed in `nargo backend ls` or in ~/.nargo/backends.

2. Reinstall a compatible version of the proving backend.

If you are using the default barretenberg backend, simply run:

```
nargo backend install acvm-backend-barretenberg https://github.com/noir-lang/barretenberg-js-binary/raw/master/run-bb.tar.gz
```

This downloads and installs a specific bb.js based version of barretenberg binary from GitHub.

The gzipped filed is running this bash script: <https://github.com/noir-lang/barretenberg-js-binary/blob/master/run-bb-js.sh>, where we need to gzip it as the Nargo currently expect the backend to be zipped up.

Then run:

```
DESIRED_BINARY_VERSION=0.8.1 nargo info
```

This overrides the bb native binary with a bb.js node application instead, which should be compatible with most if not all hardware. This does come with the drawback of being generally slower than native binary.

0.8.1 indicates bb.js version 0.8.1, so if you change that it will update to a different version or the default version in the script if none was supplied.
