---
title: How to compile a contract
---

Once you have written a [contract](../main.md) in Aztec.nr, you will need to compile it into an [artifact](./artifacts.md) in order to use it.

In this guide we will cover how to do so, both using the CLI and programmatically.

We'll also cover how to generate a helper [TypeScript interface](#typescript-interfaces) and an [Aztec.nr interface](#noir-interfaces) for easily interacting with your contract from your typescript app and from other Aztec.nr contracts, respectively.

## Compile using aztec-nargo

To compile a contract using the Aztec's build of nargo.

Run the `aztec-nargo compile` command within your [contract project folder](../writing_contracts/layout.md), which is the one that contains the `Nargo.toml` file:

```bash
aztec-nargo compile
```

This will output a JSON [artifact](./artifacts.md) for each contract in the project to a `target` folder containing the Noir ABI artifacts.

### Typescript Interfaces

You can use the code generator to autogenerate type-safe typescript classes for each of your contracts. These classes define type-safe methods for deploying and interacting with your contract based on their artifact.

```bash
aztec-cli codegen ./aztec-nargo/output/target/path -o src/artifacts
```

Below is typescript code generated from the [Token](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/noir-contracts/contracts/token_contract/src/main.nr) contract:

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

Read more about interacting with contracts using `aztec.js` [here](../../getting_started/aztecjs-getting-started.md).

### Aztec.nr interfaces

An Aztec.nr contract can [call a function](../writing_contracts/functions/call_functions.md) in another contract via `context.call_private_function` or `context.call_public_function`. However, this requires manually assembling the function selector and manually serializing the arguments, which is not type-safe.

To make this easier, the compiler automatically generates interface structs that expose a convenience method for each function listed in a given contract artifact. These structs are intended to be used from another contract project that calls into the current one. 

Below is an example of interface usage generated from the [Token](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/noir-contracts/contracts/token_contract/src/main.nr) contract, used from the [FPC](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/noir-contracts/contracts/fpc_contract/src/main.nr):

```rust
contract FPC {

    ...

    use dep::token::Token;

    ...


   #[aztec(private)]
    fn fee_entrypoint_private(amount: Field, asset: AztecAddress, secret_hash: Field, nonce: Field) {
        assert(asset == storage.other_asset.read_private());
        Token::at(asset).unshield(context.msg_sender(), context.this_address(), amount, nonce).call(&mut context);
        FPC::at(context.this_address()).pay_fee_with_shielded_rebate(amount, asset, secret_hash).enqueue(&mut context);
    }

    #[aztec(private)]
    fn fee_entrypoint_public(amount: Field, asset: AztecAddress, nonce: Field) {
        FPC::at(context.this_address()).prepare_fee(context.msg_sender(), amount, asset, nonce).enqueue(&mut context);
        FPC::at(context.this_address()).pay_fee(context.msg_sender(), amount, asset).enqueue(&mut context);
    }

    ...

}
```

Read more about how to use the Aztec.nr interfaces [here](../writing_contracts/functions/main.md).

:::info
At the moment, the compiler generates these interfaces from already compiled ABIs, and not from source code. This means that you should not import a generated interface from within the same project as its source contract, or you risk circular references.
:::

## Next steps

Once you have compiled your contracts, you can use the generated artifacts via the `Contract` class in the `aztec.js` package to deploy and interact with them, or rely on the type-safe typescript classes directly. Alternatively, use the CLI [to deploy](../../sandbox/references/cli-commands.md#deploying-a-token-contract) and [interact](../../sandbox/references/cli-commands.md#calling-an-unconstrained-view-function) with them.

import Disclaimer from "../../../misc/common/\_disclaimer.mdx";
<Disclaimer/>
