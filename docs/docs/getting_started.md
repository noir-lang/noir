---
title: Getting Started
description: Create and run your first Noir project.
---

## Using AI

Paste this into your AI agent to get started:

```
Follow https://noir-lang.org/docs/getting_started#manual and walk me through creating and running my first Noir project step-by-step end-to-end (from installation to proof verification); explain what each step does
```

The prompt walks you through setting up and interacting with a basic Noir project.

### Installing MCP server

Additionally, paste this into your AI agent to install Noir's MCP server:

```
If you support MCP, install noir-mcp-server following the instructions in https://github.com/critesjosh/noir-mcp-server/blob/master/README.md#install; test and make sure it is properly installed and configured to be accessible whenever you would need it (e.g. answering Noir questions, writing Noir code)
```

The prompt installs [noir-mcp-server](https://github.com/critesjosh/noir-mcp-server), which provides your AI agent with efficient access to Noir's repository, documentation, libraries, etc.

## Manual

### Installing Nargo

The Nargo CLI tool provides you the ability to create, compile, execute and test Noir programs from the terminal.

Install Nargo by running this in your terminal:

```sh
curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
noirup
```

This installs `noirup`, the installation script, and runs it to install the latest version of Nargo.

### Creating a project

Create a new Noir project named _hello\_world_:

```sh
nargo new hello_world
```

This command creates a new _hello\_world_ project directory, in which contains _src/main.nr_ that hosts a simple Noir program asserting _x_ does not equal _y_.

### Executing the project

Change directory into your _hello\_world_ project:

```sh
cd hello_world
```

Generate a _Prover.toml_ input file for specifying input values:

```sh
nargo check
```

This command generates a _Prover.toml_ file that hosts input values to be used when executing your Noir program.

Specify valid values in the _Prover.toml_ file, for example:

```toml
x = 1
y = 2
```

Then compile and execute your Noir program:

```sh
nargo execute
```

This command:
1. Compiles the Noir program into the _target/hello\_world.json_ circuit, and
2. Executes the Noir program with the specified inputs, and generates the _target/hello\_world.gz_ witness

Both to be used when proving your Noir program.

### Proving the execution

Noir is designed to be proving backend agnostic, which means you can choose to use different proving backends to prove and verify your Noir programs, hence the corresponding workflows could differ.

Using Barretenberg as an example, first install `bb` its CLI tool:

```sh
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/next/barretenberg/bbup/install | bash
bbup
```

Prove the execution of your Noir program:

```sh
bb prove -b ./target/hello_world.json -w ./target/hello_world.gz --write_vk -o target
```

This command:
1. Proves the valid execution of your Noir program and generates the zero-knowledge proof _target/proof_, and
2. Generates the verification key of your Noir program _target/vk_

Both to be used when verifying your proof.

### Verifying the proof

Verify your proof:

```sh
bb verify -p ./target/proof -k ./target/vk
```

This command verifies validity of the zero-knowledge proof, and returns a success message if valid.

In typical workflows, the prover who generates the proof and the verifier who verifies the proof are usually two different parties, where the verifier could verify the validity of the prover's proof without knowing the prover's private inputs, hence zero-knowledge.