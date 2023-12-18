# Compiling contracts

Once you have written a [contract](../contracts/main.md) in Aztec.nr, you will need to compile it into an [artifact](./artifacts.md) in order to use it.

In this guide we will cover how to do so, both using the CLI and programmatically.

We'll also cover how to generate a helper [TypeScript interface](#typescript-interfaces) and an [Aztec.nr interface](#noir-interfaces) for easily interacting with your contract from your typescript app and from other Aztec.nr contracts, respectively.

## Compile using the CLI

To compile a contract using the Aztec CLI, first [install it](../cli/cli-commands#installation).

Then run the `compile` command with the path to your [contract project folder](./layout.md#directory-structure), which is the one that contains the `Nargo.toml` file:

```
aztec-cli compile ./path/to/my_aztec_contract_project
```

This will output a JSON [artifact](./artifacts.md) for each contract in the project to a `target` folder containing their artifact, which you can use for deploying or interacting with your contracts.

`aztec-cli` uses `noir_wasm` by default for compiling contracts. This helps reduce the developer overhead of installation and maintaining the noir compiler, `nargo`. However, if you prefer, you can use `nargo` to compile contracts with `aztec-cli` as so:

```bash
aztec-cli compile my-contract --compiler nargo # switches compiler to nargo
```

When you specify `nargo` as your compiler, you need to make sure that you are using the correct version. You can find the [latest version information here](../updating.md#updating-nargo).

### Typescript Interfaces

You can use the compiler to autogenerate type-safe typescript classes for each of your contracts. These classes define type-safe methods for deploying and interacting with your contract based on their artifact.

To generate them, include a `--typescript` option in the compile command with a path to the target folder for the typescript files:

```
aztec-cli compile --typescript ./path/to/typescript/src ./path/to/my_aztec_contract_project
```

You can also generate these interfaces from prebuilt artifacts using the `generate-typescript` command:

```
aztec-cli generate-typescript ./path/to/my_aztec_contract_project
```

Below is typescript code generated from the [Token](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr) contract:

```ts showLineNumbers
export class TokenContract extends ContractBase {
  private constructor(completeAddress: CompleteAddress, wallet: Wallet, portalContract = EthAddress.ZERO) {
    super(completeAddress, TokenContractArtifact, wallet, portalContract);
  }

  /**
   * Creates a contract instance.
   * @param address - The deployed contract's address.
   * @param wallet - The wallet to use when interacting with the contract.
   * @returns A promise that resolves to a new Contract instance.
   */
  public static async at(address: AztecAddress, wallet: Wallet) {
    return Contract.at(address, TokenContract.artifact, wallet) as Promise<TokenContract>;
  }

  /**
   * Creates a tx to deploy a new instance of this contract.
   */
  public static deploy(pxe: PXE, admin: AztecAddressLike) {
    return new DeployMethod<TokenContract>(Point.ZERO, pxe, TokenContractArtifact, Array.from(arguments).slice(1));
  }

  /**
   * Creates a tx to deploy a new instance of this contract using the specified public key to derive the address.
   */
  public static deployWithPublicKey(pxe: PXE, publicKey: PublicKey, admin: AztecAddressLike) {
    return new DeployMethod<TokenContract>(publicKey, pxe, TokenContractArtifact, Array.from(arguments).slice(2));
  }

  /**
   * Returns this contract's artifact.
   */
  public static get artifact(): ContractArtifact {
    return TokenContractArtifact;
  }

  /** Type-safe wrappers for the public methods exposed by the contract. */
  public methods!: {

    /** balance_of_private(owner: struct) */
    balance_of_private: ((owner: AztecAddressLike) => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;

    /** balance_of_public(owner: struct) */
    balance_of_public: ((owner: AztecAddressLike) => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;

    /** shield(from: struct, amount: field, secret_hash: field, nonce: field) */
    shield: ((
      from: AztecAddressLike,
      amount: FieldLike,
      secret_hash: FieldLike,
      nonce: FieldLike,
    ) => ContractFunctionInteraction) &
      Pick<ContractMethod, 'selector'>;

    /** total_supply() */
    total_supply: (() => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;

    /** transfer(from: struct, to: struct, amount: field, nonce: field) */
    transfer: ((
      from: AztecAddressLike,
      to: AztecAddressLike,
      amount: FieldLike,
      nonce: FieldLike,
    ) => ContractFunctionInteraction) &
      Pick<ContractMethod, 'selector'>;

    /** transfer_public(from: struct, to: struct, amount: field, nonce: field) */
    transfer_public: ((
      from: AztecAddressLike,
      to: AztecAddressLike,
      amount: FieldLike,
      nonce: FieldLike,
    ) => ContractFunctionInteraction) &
      Pick<ContractMethod, 'selector'>;

    ...
  };
}
```

Read more about interacting with contracts using `aztec.js` [here](../getting_started/aztecjs-getting-started.md).

### Aztec.nr interfaces

An Aztec.nr contract can [call a function](./syntax/functions.md) in another contract via `context.call_private_function` or `context.call_public_function`. However, this requires manually assembling the function selector and manually serializing the arguments, which is not type-safe.

To make this easier, the compiler can generate contract interface structs that expose a convenience method for each function listed in a given contract artifact. These structs are intended to be used from another contract project that calls into the current one. For each contract, two interface structs are generated: one to be used from private functions with a `PrivateContext`, and one to be used from open functions with a `PublicContext`.

To generate them, include a `--interface` option in the compile command with a path to the target folder for the generated Aztec.nr interface files:

```
aztec-cli compile --interface ./path/to/another_aztec_contract_project/src ./path/to/my_aztec_contract_project
```

You can also generate these interfaces from prebuilt artifacts using the `generate-noir-interface` command:

```
aztec-cli generate-noir-interface ./path/to/my_aztec_contract_project
```

Below is an example interface, also generated from the [Token](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr) contract:

```rust
impl TokenPrivateContextInterface {
  pub fn at(address: Field) -> Self {
      Self {
          address,
      }
  }

  pub fn burn(
    self,
    context: &mut PrivateContext,
    from: FromBurnStruct,
    amount: Field,
    nonce: Field
  ) -> [Field; RETURN_VALUES_LENGTH] {
    let mut serialized_args = [0; 3];
    serialized_args[0] = from.address;
    serialized_args[1] = amount;
    serialized_args[2] = nonce;

    context.call_private_function(self.address, 0xd4fcc96e, serialized_args)
  }


  pub fn burn_public(
    self,
    context: &mut PrivateContext,
    from: FromBurnPublicStruct,
    amount: Field,
    nonce: Field
  ) {
    let mut serialized_args = [0; 3];
    serialized_args[0] = from.address;
    serialized_args[1] = amount;
    serialized_args[2] = nonce;

    context.call_public_function(self.address, 0xb0e964d5, serialized_args)
  }
  ...

}

impl TokenPublicContextInterface {
  pub fn at(address: Field) -> Self {
      Self {
          address,
      }
  }

  pub fn burn_public(
    self,
    context: PublicContext,
    from: FromBurnPublicStruct,
    amount: Field,
    nonce: Field
  ) -> [Field; RETURN_VALUES_LENGTH] {
    let mut serialized_args = [0; 3];
    serialized_args[0] = from.address;
    serialized_args[1] = amount;
    serialized_args[2] = nonce;

    context.call_public_function(self.address, 0xb0e964d5, serialized_args)
  }


  pub fn mint_private(
    self,
    context: PublicContext,
    amount: Field,
    secret_hash: Field
  ) -> [Field; RETURN_VALUES_LENGTH] {
    let mut serialized_args = [0; 2];
    serialized_args[0] = amount;
    serialized_args[1] = secret_hash;

    context.call_public_function(self.address, 0x10763932, serialized_args)
  }


}
```

Read more about how to use the Aztec.nr interfaces [here](./syntax/functions.md#contract-interface).

:::info
At the moment, the compiler generates these interfaces from already compiled ABIs, and not from source code. This means that you should not import a generated interface from within the same project as its source contract, or you risk circular references.
:::

## Next steps

Once you have compiled your contracts, you can use the generated artifacts via the `Contract` class in the `aztec.js` package to deploy and interact with them, or rely on the type-safe typescript classes directly. Alternatively, use the CLI [to deploy](../../dev_docs/cli/main.md#deploying-a-token-contract) and [interact](../../dev_docs/cli/main.md#sending-a-transaction) with them.

import Disclaimer from "../../misc/common/\_disclaimer.mdx";
<Disclaimer/>
