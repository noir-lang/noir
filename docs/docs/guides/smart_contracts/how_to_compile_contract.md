---
title: How to Compile a Contract
sidebar_position: 3
---

Once you have written a contract in Aztec.nr, you will need to compile it into an [artifact](../../aztec/concepts/smart_contracts/contract_structure.md) in order to use it.

In this guide we will cover how to do so, both using the `aztec-nargo` command and programmatically.

We'll also cover how to generate a helper [TypeScript interface](#typescript-interfaces) and an [Aztec.nr interface](#noir-interfaces) for easily interacting with your contract from your typescript app and from other Aztec.nr contracts, respectively.

## Compile using aztec-nargo

To compile a contract using the Aztec's build of nargo.

Run the `aztec-nargo compile` command within your contract project folder (the one that contains the `Nargo.toml` file):

```bash
aztec-nargo compile
```

This will output a JSON [artifact](../../aztec/concepts/smart_contracts/contract_structure.md) for each contract in the project to a `target` folder containing the Noir ABI artifacts.

:::note
This command looks for `Nargo.toml` files by ascending up the parent directories, and will compile the top-most Nargo.toml file it finds.
Eg: if you are in `/hobbies/cool-game/contracts/easter-egg/`, and both `cool-game` and `easter-egg` contain a Nargo.toml file, then `aztec-nargo compile` will be performed on `cool-game/Nargo.toml` and compile the project(s) specified within it. Eg

```
[workspace]
members = [
    "contracts/easter-egg",
]
```

:::

### Typescript Interfaces

You can use the code generator to autogenerate type-safe typescript classes for each of your contracts. These classes define type-safe methods for deploying and interacting with your contract based on their artifact.

```bash
aztec codegen ./aztec-nargo/output/target/path -o src/artifacts
```

Below is typescript code generated from the [Token](https://github.com/AztecProtocol/aztec-packages/blob/master/noir-projects/noir-contracts/contracts/token_contract/src/main.nr) contract:

```ts showLineNumbers
export class TokenContract extends ContractBase {
  private constructor(instance: ContractInstanceWithAddress, wallet: Wallet) {
    super(instance, TokenContractArtifact, wallet);
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
  public static deploy(
    wallet: Wallet,
    admin: AztecAddressLike,
    name: string,
    symbol: string,
    decimals: bigint | number,
  ) {
    return new DeployMethod<TokenContract>(
      Fr.ZERO,
      wallet,
      TokenContractArtifact,
      TokenContract.at,
      Array.from(arguments).slice(1),
    );
  }

  /**
   * Creates a tx to deploy a new instance of this contract using the specified public keys hash to derive the address.
   */
  public static deployWithPublicKeysHash(
    publicKeysHash: Fr,
    wallet: Wallet,
    admin: AztecAddressLike,
    name: string,
    symbol: string,
    decimals: bigint | number,
  ) {
    return new DeployMethod<TokenContract>(
      publicKeysHash,
      wallet,
      TokenContractArtifact,
      TokenContract.at,
      Array.from(arguments).slice(2),
    );
  }

  /**
   * Creates a tx to deploy a new instance of this contract using the specified constructor method.
   */
  public static deployWithOpts<M extends keyof TokenContract['methods']>(
    opts: { publicKeysHash?: Fr; method?: M; wallet: Wallet },
    ...args: Parameters<TokenContract['methods'][M]>
  ) {
    return new DeployMethod<TokenContract>(
      opts.publicKeysHash ?? Fr.ZERO,
      opts.wallet,
      TokenContractArtifact,
      TokenContract.at,
      Array.from(arguments).slice(1),
      opts.method ?? 'constructor',
    );
  }

  /**
   * Returns this contract's artifact.
   */
  public static get artifact(): ContractArtifact {
    return TokenContractArtifact;
  }

  public static get storage(): ContractStorageLayout<
    | 'admin'
    | 'minters'
    | 'balances'
    | 'total_supply'
    | 'pending_shields'
    | 'public_balances'
    | 'symbol'
    | 'name'
    | 'decimals'
  > {
    return {
      admin: {
        slot: new Fr(1n),
        typ: 'PublicMutable<AztecAddress>',
      },
      minters: {
        slot: new Fr(2n),
        typ: 'Map<AztecAddress, PublicMutable<bool>>',
      },
      balances: {
        slot: new Fr(3n),
        typ: 'BalancesMap<TokenNote>',
      },
      total_supply: {
        slot: new Fr(4n),
        typ: 'PublicMutable<U128>',
      },
      pending_shields: {
        slot: new Fr(5n),
        typ: 'PrivateSet<TransparentNote>',
      },
      public_balances: {
        slot: new Fr(6n),
        typ: 'Map<AztecAddress, PublicMutable<U128>>',
      },
      symbol: {
        slot: new Fr(7n),
        typ: 'SharedImmutable<FieldCompressedString>',
      },
      name: {
        slot: new Fr(8n),
        typ: 'SharedImmutable<FieldCompressedString>',
      },
      decimals: {
        slot: new Fr(9n),
        typ: 'SharedImmutable<u8>',
      },
    } as ContractStorageLayout<
      | 'admin'
      | 'minters'
      | 'balances'
      | 'total_supply'
      | 'pending_shields'
      | 'public_balances'
      | 'symbol'
      | 'name'
      | 'decimals'
    >;
  }

  public static get notes(): ContractNotes<'TransparentNote' | 'TokenNote'> {
    const notes = this.artifact.outputs.globals.notes ? (this.artifact.outputs.globals.notes as any) : [];
    return {
      TransparentNote: {
        id: new Fr(84114971101151129711410111011678111116101n),
      },
      TokenNote: {
        id: new Fr(8411110710111078111116101n),
      },
    } as ContractNotes<'TransparentNote' | 'TokenNote'>;
  }

  /** Type-safe wrappers for the public methods exposed by the contract. */
  public override methods!: {
    /** transfer_public(from: struct, to: struct, amount: field, nonce: field) */
    transfer_public: ((
      from: AztecAddressLike,
      to: AztecAddressLike,
      amount: FieldLike,
      nonce: FieldLike,
    ) => ContractFunctionInteraction) &
      Pick<ContractMethod, 'selector'>;

    /** transfer(from: struct, to: struct, amount: field, nonce: field) */
    transfer: ((
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

Read more about interacting with contracts using `aztec.js` [here](../../tutorials/aztecjs-getting-started.md).

### Aztec.nr interfaces

An Aztec.nr contract can [call a function](writing_contracts/call_functions.md) in another contract via `context.call_private_function` or `context.call_public_function`. However, this requires manually assembling the function selector and manually serializing the arguments, which is not type-safe.

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

Read more about how to use the Aztec.nr interfaces [here](../../aztec/concepts/smart_contracts/functions/index.md).

:::info
At the moment, the compiler generates these interfaces from already compiled ABIs, and not from source code. This means that you should not import a generated interface from within the same project as its source contract, or you risk circular references.
:::

## Next steps

Once you have compiled your contracts, you can use the generated artifacts via the `Contract` class in the `aztec.js` package to deploy and interact with them, or rely on the type-safe typescript classes directly.
