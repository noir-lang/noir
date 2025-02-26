---
title: Noir Codegen for TypeScript
description: Learn how to use Noir codegen to generate TypeScript bindings 
keywords: [Nargo, Noir, compile, TypeScript]
sidebar_position: 3
---

When using TypeScript, it is extra work to interpret Noir program outputs in a type-safe way. Third party libraries may exist for popular Noir programs, but they are either hard to find or unmaintained.

Now you can generate TypeScript bindings for your Noir programs in two steps:

1. Exporting Noir functions using `nargo export`
2. Using the TypeScript module `noir_codegen` to generate TypeScript binding

**Note:** you can only export functions from a Noir *library* (not binary or contract program types).

## Installation

### Your TypeScript project

If you don't already have a TypeScript project you can add the module with `yarn` (or `npm`), then initialize it:

```bash
yarn add typescript -D
npx tsc --init
```

### Add TypeScript module - `noir_codegen`

The following command will add the module to your project's devDependencies:

```bash
yarn add @noir-lang/noir_codegen -D
```

### Nargo library

Make sure you have Nargo, v0.25.0 or greater, installed. If you don't, follow the [installation guide](../getting_started/noir_installation.md).

If you're in a new project, make a `circuits` folder and create a new Noir library:

```bash
mkdir circuits && cd circuits
nargo new --lib myNoirLib
```

## Usage

### Export ABI of specified functions

First go to the `.nr` files in your Noir library, and add the `#[export]` macro to each function that you want to use in TypeScript.

```rust
#[export]
fn your_function(...
```

From your Noir library (where `Nargo.toml` is), run the following command:

```bash
nargo export
```

You will now have an `export` directory with a .json file per exported function.

You can also specify the directory of Noir programs using `--program-dir`, for example:

```bash
nargo export --program-dir=./circuits/myNoirLib
```

### Generate TypeScript bindings from exported functions

To use the `noir-codegen` package we added to the TypeScript project:

```bash
yarn noir-codegen ./export/your_function.json
```

This creates an `exports` directory with an `index.ts` file containing all exported functions.

**Note:** adding `--out-dir` allows you to specify an output dir for your TypeScript bindings to go. Eg:

```bash
yarn noir-codegen ./export/*.json --out-dir ./path/to/output/dir
```

## Example .nr function to .ts output

Consider a Noir library with this function:

```rust
#[export]
fn not_equal(x: Field, y: Field) -> bool {
    x != y
}
```

After the export and codegen steps, you should have an `index.ts` like:

```typescript
export type Field = string;


export const is_equal_circuit: CompiledCircuit = 
{"abi":{"parameters":[{"name":"x","type":{"kind":"field"},"visibility":"private"},{"name":"y","type":{"kind":"field"},"visibility":"private"}],"return_type":{"abi_type":{"kind":"boolean"},"visibility":"private"}},"bytecode":"H4sIAAAAAAAA/7WUMQ7DIAxFQ0Krrr2JjSGYLVcpKrn/CaqqDQN12WK+hPBgmWd/wEyHbF1SS923uhOs3pfoChI+wKXMAXzIKyNj4PB0TFTYc0w5RUjoqeAeEu1wqK0F54RGkWvW44LPzExnlkbMEs4JNZmN8PxS42uHv82T8a3Jeyn2Ks+VLPcO558HmyLMCDOXAXXtpPt4R/Rt9T36ss6dS9HGPx/eG17nGegKBQAA"};

export async function is_equal(x: Field, y: Field, foreignCallHandler?: ForeignCallHandler): Promise<boolean> {
  const program = new Noir(is_equal_circuit);
  const args: InputMap = { x, y };
  const { returnValue } = await program.execute(args, foreignCallHandler);
  return returnValue as boolean;
}
```

Now the `is_equal()` function and relevant types are readily available for use in TypeScript.
