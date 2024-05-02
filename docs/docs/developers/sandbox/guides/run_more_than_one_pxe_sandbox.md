---
title: How to run more than one PXE in the sandbox
---

When you run the sandbox, the Aztec node and PXE have their own http server. This makes it possible to run two PXEs on your local machine, which can be useful for testing that notes are accurately stored and remaining private in their respective PXEs.

We are working on a better solution for this so expect an update soon, but currently you can follow this guide.

## Run the sandbox in one terminal

Rather than use the usual command, run:
```bash
cd ~/.aztec && docker-compose up
```
This removes any other arguments, allowing you to ensure an isolated environment for the sandbox so it doesn't interfere with another PXE.

## Run PXE mode in another terminal

In another terminal, run:

```bash
aztec start --pxe nodeUrl=http://localhost:8080/
```

This command uses the default ports, so they might need to be changed depending on yuor configuration.

You should see something like this:

```bash
 kv-store:lmdb Opening LMDB database at temporary location
  kv-store:lmdb Opening LMDB database at temporary location
  pxe_service Added contract ContractClassRegisterer at 0x030c6b23cf81a1c1387674e7d180ef04abc19387eb0ec71eea67c2b602b517b7
  pxe_service Added contract ContractInstanceDeployer at 0x2d8e7aedc70b65d49e6aa0794d8d12721896c177e87126701f6e60d184358e74
  pxe_service Added contract MultiCallEntrypoint at 0x0325a7874e168991a060b7f54e7324a42f87f48ffa592a903a5ce170b9d99e20
  pxe_service Added contract GasToken at 0x0f0be9c2e88fe0a7baa0823fbf7cfba98a6ba71558d6b5a4ee497e3b38f0aa7c
  pxe_synchronizer Initial sync complete
  pxe_service Started PXE connected to chain 31337 version 1
Aztec Server listening on port 8080
```

You can learn more about custom commands in the [sandbox reference](../references/sandbox-reference.md).


