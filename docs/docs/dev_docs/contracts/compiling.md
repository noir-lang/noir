# Compiling contracts

Once you have written a [contract](../contracts/main.md) in Aztec.nr, you will need to compile it into an [artifact](./abi.md) in order to use it.

In this guide we will cover how to do so, both using the CLI and programmatically.

We'll also cover how to generate a helper [TypeScript interface](#typescript-interfaces) and an [Aztec.nr interface](#noir-interfaces) for easily interacting with your contract from your typescript app and from other Aztec.nr contracts, respectively.

## Prerequisites

You will need the Noir build tool `nargo`, which you can install via [`noirup`](https://github.com/noir-lang/noirup). Make sure you install the correct version of nargo:

<InstallNargoInstructions />

:::info
You can run `aztec-cli get-node-info` to query the version of nargo that corresponds to your current installation.
:::

## Compile using the CLI

To compile a contract using the Aztec CLI, first install it:

`npm install -g @aztec/cli`

Then run the `compile` command with the path to your [contract project folder](./layout.md#directory-structure), which is the one that contains the `Nargo.toml` file:

```
aztec-cli compile ./path/to/my_aztec_contract_project
```

This will output a JSON [artifact](./artifacts.md) for each contract in the project to a `target` folder containing their [ABI](./abi.md), which you can use for deploying or interacting with your contracts.

### Typescript Interfaces

You can use the compiler to autogenerate type-safe typescript classes for each of your contracts. These classes define type-safe methods for deploying and interacting with your contract based on their ABI.

To generate them, include a `--typescript` option in the compile command with a path to the target folder for the typescript files:

```
aztec-cli compile --typescript ./path/to/typescript/src ./path/to/my_aztec_contract_project
```

You can also generate these interfaces from prebuilt artifacts using the `generate-typescript` command:

```
aztec-cli generate-typescript ./path/to/my_aztec_contract_project
```

Example code generated from the [PrivateToken](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr) contract:

```ts showLineNumbers
export class PrivateTokenContract extends ContractBase {
  /** Creates a contract instance at the given address. */
  public static async at(address: AztecAddress, wallet: Wallet) { ... }

  /** Creates a tx to deploy a new instance of this contract. */
  public static deploy(rpc: AztecRPC, initial_supply: FieldLike, owner: FieldLike) { ... }

  /** Type-safe wrappers for the public methods exposed by the contract. */
  public methods!: {
    /** getBalance(owner: field) */
    getBalance: ((owner: FieldLike) => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;

    /** mint(amount: field, owner: field) */
    mint: ((amount: FieldLike, owner: FieldLike) => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;

    /** transfer(amount: field, sender: field, recipient: field) */
    transfer: ((amount: FieldLike, sender: FieldLike, recipient: FieldLike) => ContractFunctionInteraction) &
      Pick<ContractMethod, 'selector'>;
  };
}
```

Read more about interacting with contracts using `aztec.js` [here](../dapps/main.md).

### Aztec.nr interfaces

An Aztec.nr contract can [call a function](./functions.md) in another contract via `context.call_private_function` or `context.call_public_function`. However, this requires manually assembling the function selector and manually serialising the arguments, which is not type-safe.

To make this easier, the compiler can generate contract interface structs that expose a convenience method for each function listed in a given contract ABI. These structs are intended to be used from another contract project that calls into the current one. For each contract, two interface structs are generated: one to be used from private functions with a `PrivateContext`, and one to be used from open functions with a `PublicContext`.

To generate them, include a `--interface` option in the compile command with a path to the target folder for the generated Aztec.nr interface files:

```
aztec-cli compile --interface ./path/to/another_aztec_contract_project/src ./path/to/my_aztec_contract_project
```

You can also generate these interfaces from prebuilt artifacts using the `generate-noir-interface` command:

```
aztec-cli generate-noir-interface ./path/to/my_aztec_contract_project
```

Example code generated from the [PrivateToken](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr) contract:

```rust
impl PrivateTokenPrivateContextInterface {
  fn at(address: Field) -> Self {
      Self { address }
  }
  
  fn mint(
    self, context: &mut PrivateContext, amount: Field, owner: Field
  ) -> [Field; RETURN_VALUES_LENGTH] {
    let mut serialised_args = [0; 2];
    serialised_args[0] = amount;
    serialised_args[1] = owner;

    // 0x1dc9c3c0 is the function selector for `mint(field,field)`
    context.call_private_function(self.address, 0x1dc9c3c0, serialised_args)
  }
  

  fn transfer(
    self, context: &mut PrivateContext, amount: Field, sender: Field, recipient: Field
  ) -> [Field; RETURN_VALUES_LENGTH] {
    let mut serialised_args = [0; 3];
    serialised_args[0] = amount;
    serialised_args[1] = sender;
    serialised_args[2] = recipient;

    // 0xdcd4c318 is the function selector for `transfer(field,field,field)`
    context.call_private_function(self.address, 0xdcd4c318, serialised_args)
  }
}
```

Read more about how to use the Aztec.nr interfaces [here](./functions.md#contract-interface).

:::info
At the moment, the compiler generates these interfaces from already compiled ABIs, and not from source code. This means that you should not import a generated interface from within the same project as its source contract, or you risk circular references.
:::

## Compile using nodejs

You can also programmatically access the compiler via the `@aztec/noir-compiler` package. To do this, install the package into your nodejs project:

`
npm install @aztec/noir-compiler
`

The compiler exposes the following functions:
- `compileUsingNargo`: Compiles an Aztec.nr project in the target folder using the `nargo` binary available on the shell `PATH` and returns the generated ABIs.
- `generateTypescriptContractInterface`: Generates a typescript class for the given contract ABI.
- `generateNoirContractInterface`: Generates a Aztec.nr interface struct for the given contract ABI.

## Next steps

Once you have compiled your contracts, you can use the generated artifacts via the `Contract` class in the `aztec.js` package to deploy and interact with them, or rely on the type-safe typescript classes directly. Alternatively, use the CLI [to deploy](../../dev_docs/getting_started/cli.md#deploying-a-token-contract) and [interact](../../dev_docs/getting_started/cli.md#sending-a-transaction) with them.


import Disclaimer from "../../misc/common/\_disclaimer.mdx";
<Disclaimer/>