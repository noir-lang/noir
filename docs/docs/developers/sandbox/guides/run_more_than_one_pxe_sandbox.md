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
aztec start --pxe nodeUrl=http://aztec-aztec-1:8079
```

This command uses the default ports, so they might need to be changed depending on yuor configuration.

You can learn more about custom commands in the [sandbox reference](../references/sandbox-reference.md).


