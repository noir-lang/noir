# Writing a token contract in Aztec.nr

In this tutorial we will go through writing an L2 native token contract
for the Aztec Network, using the Aztec.nr contract libraries. It is recommended that you go through the [the introduction to contracts](../contracts/main.md) and [setup instructions](../contracts/setup.md) section before this tutorial to gain some familiarity with writing Aztec smart contracts.

This tutorial is intended to help you get familiar with the Aztec.nr library, Aztec contract syntax and some of the underlying structure of the Aztec network.

In this tutorial you will learn how to:

- Write public functions that update public state
- Write private functions that update private state
- Implement access control on public and private functions
- Handle math operations safely
- Handle different private note types
- Pass data between private and public state

We are going to start with a blank project and fill in the token contract source code defined on Github [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr), and explain what is being added as we go.

## Requirements

You will need to have `aztec-cli` installed in order to compile Aztec.nr contracts. See the [quickstart guide](../getting_started/quickstart.md#cli) for installation instructions.

When you are running the Sandbox, `aztec-cli`, and compiling contracts with Noir, make sure you are using matching versions--we will be shipping breaking changes so mis-matched versions may not work.

You should also install the [Noir Language Support extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir) for VS Code.

Check the [Dev Tools section](https://github.com/noir-lang/awesome-noir#dev-tools) of the awesome-noir repo for language support for additional editors (Vim, emacs, tree-sitter, etc).

## Project setup

Create a new directory called `token_contract_tutorial`

```bash
mkdir token_contract_tutorial
```

inside that directory, create a `contracts` folder for the Aztec contracts.

```bash
cd token_contract_tutorial && mkdir contracts && cd contracts
```

Create the following file structure

```tree
.
└── contracts
    ├── Nargo.toml
    └── src
        └── main.nr
```

Add the following content to Nargo.toml file:

```toml
[package]
name = "token_contract"
authors = [""]
compiler_version = ">=0.18.0"
type = "contract"

[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
safe_math = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/safe-math"}
authwit={ git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/authwit"}
```

## Contract Interface

```rust
contract Token {
    #[aztec(private)]
    fn constructor() {}

    #[aztec(public)]
    fn set_admin(new_admin: AztecAddress) {}

    #[aztec(public)]
    fn set_minter(minter: AztecAddress, approve: bool) {}

    #[aztec(public)]
    fn mint_public(to: AztecAddress, amount: Field) -> Field {}

    #[aztec(public)]
    fn mint_private(amount: Field, secret_hash: Field) -> Field {}

    #[aztec(public)]
    fn shield(from: AztecAddress, amount: Field, secret_hash: Field, nonce: Field) -> Field {}

    #[aztec(public)]
    fn transfer_public(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) -> Field {}

    #[aztec(public)]
    fn burn_public(from: AztecAddress, amount: Field, nonce: Field) -> Field {}

    // Private functions

    #[aztec(private)]
    fn redeem_shield(to: AztecAddress, amount: Field, secret: Field) -> Field {}

    #[aztec(private)]
    fn unshield(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) -> Field {}

    #[aztec(private)]
    fn transfer(from: AztecAddress, to: AztecAddress, amount: Field, nonce: Field) -> Field {}

    #[aztec(private)]
    fn burn(from: AztecAddress, amount: Field, nonce: Field) -> Field {}

    // Internal functions below

    // Will be internal in the future
    #[aztec(public)]
    fn _initialize(new_admin: AztecAddress) {}

    #[aztec(public)]
    internal fn _increase_public_balance(to: AztecAddress, amount: Field) {}

    #[aztec(public)]
    internal fn _reduce_total_supply(amount: Field) {}

    // Unconstrained functions (read only)

    unconstrained fn admin() -> Field {}

    unconstrained fn is_minter(minter: AztecAddress) -> bool {}

    unconstrained fn total_supply() -> Field {}

    unconstrained fn balance_of_private(owner: AztecAddress) -> Field {}

    unconstrained fn balance_of_public(owner: AztecAddress) -> Field {}

    unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, serialized_note: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {}
}
```

This specifies the interface of the `Token` contract. Go ahead and copy and paste this interface into your `main.nr` file.

Before we through the interface and implement each function, let's review the functions to get a sense of what the contract does.

### Constructor interface

There is a `constructor` function that will be executed once, when the contract is deployed, similar to the constructor function in Solidity. This is marked private, so the function logic will not be transparent. To execute public function logic in the constructor, this function will call `_initialize` (marked internal, more detail below).

### Public functions

These are functions that have transparent logic, will execute in a publicly verifiable context and can update public storage.

- `set_admin` enables the admin to be updated
- `set_minter` enables accounts to be added / removed from the approved minter list
- `mint_public` enables tokens to be minted to the public balance of an account
- `mint_private` enables tokens to be minted to the private balance of an account (with some caveats we will dig into)
- `shield` enables tokens to be moved from a public balance to a private balance, not necessarily the same account (step 1 of a 2 step process)
- `transfer_public` enables users to transfer tokens from one account's public balance to another account's public balance
- `burn_public` enables users to burn tokens

### Private functions

These are functions that have private logic and will be executed on user devices to maintain privacy. The only data that is submitted to the network is a proof of correct execution, new data [commitments](https://en.wikipedia.org/wiki/Commitment_scheme) and [nullifiers](../../concepts/advanced/data_structures/trees#nullifier-tree), so users will not reveal which contract they are interacting with or which function they are executing. The only information that will be revealed publicly is that someone executed a private transaction on Aztec.

- `redeem_shield` enables accounts to claim tokens that have been made private via `mint_private` or `shield` by providing the secret
- `unshield` enables an account to send tokens from their private balance to any other account's public balance
- `transfer` enables an account to send tokens from their private balance to another account's private balance
- `burn` enables tokens to be burned privately

### Internal functions

Internal functions are functions that can only be called by the contract itself. These can be used when the contract needs to call one of it's public functions from one of it's private functions.

- `_initialize` is a way to call a public function from the `constructor` (which is a private function)
- `_increase_public_balance` increases the public balance of an account when `unshield` is called
- `_reduce_total_supply` reduces the total supply of tokens when a token is privately burned

To clarify, let's review some details of the Aztec transaction lifecycle, particularly how a transaction "moves through" these contexts.

#### Execution contexts

Transactions are initiated in the private context, then move to the L2 public context, then to the Ethereum L1 context.

Step 1. Private Execution

Users provide inputs and execute locally on a their device for privacy reasons. Outputs of the private execution are commitment and nullifier updates, a proof of correct execution and any return data to pass to the public execution context.

Step 2. Public Execution

This happens remotely by the sequencer, which takes inputs from the private execution and runs the public code in the network virtual machine, similar to any other public blockchain.

Step 3. Ethereum execution

Aztec transactions can pass data to Ethereum contracts through the rollup via the outbox. The data can consumed by Ethereum contracts at a later time, but this is not part of the transaction flow for an Aztec transaction. The technical details of this are beyond the scope of this tutorial, but we will cover them in an upcoming piece.

### Unconstrained functions

Unconstrained functions can be thought of as view functions from Solidity--they only return information from the contract storage or compute and return data without modifying contract storage.

The `compute_note_hash_and_nullifier` function allows contract devs to specify how to compute notes and nullifiers. This must be included in every contract because it depends on the storage slots, which are defined when we set up storage.

## Contract dependencies

Before we can implement the functions, we need set up the contract storage, and before we do that we need to import the appropriate dependencies.

:::info Copy required files

We will be going over the code in `main.nr` [here](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/yarn-project/noir-contracts/src/contracts/token_contract/src). If you are following along and want to compile `main.nr` yourself, you need to add the other files in the directory as they contain imports that are used in `main.nr`.

:::

Just below the contract definition, add the following imports:

#include_code imports /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

We are importing the Option type, note utilities, context (for managing private and public execution contexts), `state_vars` for helping manage state, `compute_selector` for helping with calling public functions from private functions, and `types` for data manipulation. We also import the `authwit` [library](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/auth.nr) to handle token authorizations from [Account Contracts](../../concepts/foundation/accounts/main). Check out the Account Contract with AuthWitness [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/noir-contracts/src/contracts/schnorr_single_key_account_contract/src/main.nr).

[SafeU120](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/safe-math/src/safe_u120.nr) is a library to do safe math operations on unsigned integers that protects against overflows and underflows.

For more detail on execution contexts, see [Contract Communication](../../concepts/foundation/communication/main).

We are also importing types from a `types.nr` file. The main thing to note from this types file is the `TransparentNote` definition. This defines how the contract moves value from the public domain into the private domain. It is similar to the `value_note` that we imported, but with some modifications namely, instead of a defined `owner`, it allows anyone that can produce the pre-image to the stored `secret_hash` to spend the note.

### Note on private state

Private state in Aztec is all [UTXOs](https://en.wikipedia.org/wiki/Unspent_transaction_output) under the hood. Handling UTXOs is largely abstracted away from developers, but there are some unique things for developers to be aware of when creating and managing private state in an Aztec contract. See [State Variables](../contracts/syntax/storage/main.md) to learn more about public and private state in Aztec.

## Contract Storage

Now that we have dependencies imported into our contract we can define the storage for the contract.

Below the dependencies, paste the following Storage struct:

#include_code storage_struct /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

Reading through the storage variables:

- `admin` a single Field value stored in public state. `FIELD_SERIALIZED_LEN` indicates the length of the variable, which is 1 in this case because it's a single Field element. A `Field` is basically an unsigned integer with a maximum value determined by the underlying cryptographic curve.
- `minters` is a mapping of Fields in public state. This will store whether an account is an approved minter on the contract.
- `balances` is a mapping of private balances. Private balances are stored in a `Set` of `ValueNote`s. The balance is the sum of all of an account's `ValueNote`s.
- `total_supply` is a Field value stored in public state and represents the total number of tokens minted.
- `pending_shields` is a `Set` of `TransparentNote`s stored in private state. What is stored publicly is a set of commitments to `TransparentNote`s.
- `public_balances` is a mapping field elements in public state and represents the publicly viewable balances of accounts.

You can read more about it [here](../contracts/syntax/storage/main.md).

### Initializing Storage

Once we have Storage defined, we need to specify how to initialize it. The `init` method creates and initializes an instance of `Storage`. We define an initialization method for each of the storage variables defined above. Storage initialization is generic and can largely be reused for similar types, across different contracts, but it is important to note that each storage variable specifies it's storage slot, starting at 1.

Also, the public storage variables define the type that they store by passing the methods by which they are serialized. Because all `PublicState` in this contract is storing Field elements, each storage variable takes `FieldSerializationMethods`.

#include_code storage_init /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

## Functions

Copy and paste the body of each function into the appropriate place in your project if you are following along.

### Constructor

In the source code, the constructor logic is commented out due to some limitations of the current state of the development.

#include_code constructor /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

The constructor is a private function. There isn't any private state to set up in this function, but there is public state to set up. The `context` is a global variable that is available to private and public functions, but the available methods differ based on the context. You can see the implementation details [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/aztec/src/context.nr). The `context.call_public_function` allows a private function to call a public function on any contract. In this case, the constructor is passing the `msg_sender` as the argument to the `_initialize` function, which is also defined in this contract.

### Public function implementations

Public functions are declared with the `#[aztec(public)]` macro above the function name like so:

#include_code set_admin /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

As described in the [execution contexts section above](#execution-contexts), public function logic and transaction information is transparent to the world. Public functions update public state, but can be used to prepare data to be used in a private context, as we will go over below (e.g. see the [shield](#shield) function).

Storage is referenced as `storage.variable`.

#### `set_admin`

After storage is initialized, the contract checks that the `msg_sender` is the `admin`. If not, the transaction will fail. If it is, the `new_admin` is saved as the `admin`.

#include_code set_admin /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `set_minter`

This function allows the `admin` to add or a remove a `minter` from the public `minters` mapping. It checks that `msg_sender` is the `admin` and finally adds the `minter` to the `minters` mapping.

#include_code set_minter /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `mint_public`

This function allows an account approved in the public `minters` mapping to create new public tokens owned by the provided `to` address.

First, storage is initialized. Then the function checks that the `msg_sender` is approved to mint in the `minters` mapping. If it is, a new `SafeU120` value is created of the `amount` provided. The function reads the recipients public balance and then adds the amount to mint, saving the output as `new_balance`, then reads to total supply and adds the amount to mint, saving the output as `supply`. `new_balance` and `supply` are then written to storage.

The function returns 1 to indicate successful execution.

#include_code mint_public /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `mint_private`

This public function allows an account approved in the public `minters` mapping to create new private tokens that can be claimed by anyone that has the pre-image to the `secret_hash`.

First, public storage is initialized. Then it checks that the `msg_sender` is an approved minter. Then a new `TransparentNote` is created with the specified `amount` and `secret_hash`. You can read the details of the `TransparentNote` in the `types.nr` file [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/noir-contracts/src/contracts/token_contract/src/types.nr#L61). The `amount` is added to the existing public `total_supply` and the storage value is updated. Then the new `TransparentNote` is added to the `pending_shields` using the `insert_from_public` function, which is accessible on the `Set` type. Then it's ready to be claimed by anyone with the `secret_hash` pre-image using the `redeem_shield` function. It returns `1` to indicate successful execution.

#include_code mint_private /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `shield`

This public function enables an account to stage tokens from it's `public_balance` to be claimed as a private `TransparentNote` by any account that has the pre-image to the `secret_hash`.

First, storage is initialized. Then it checks whether the calling contract (`context.msg_sender`) matches the account that the funds will be debited from.

##### Authorizing token spends

If the `msg_sender` is **NOT** the same as the account to debit from, the function checks that the account has authorized the `msg_sender` contract to debit tokens on its behalf. This check is done by computing the function selector that needs to be authorized (in this case, the `shield` function), computing the hash of the message that the account contract has approved. This is a hash of the contract that is approved to spend (`context.msg_sender`), the token contract that can be spent from (`context.this_address()`), the `selector`, the account to spend from (`from.address`), the `amount`, the `secret_hash` and a `nonce` to prevent multiple spends. This hash is passed to `assert_current_call_valid_authwit_public` to ensure that the Account Contract has approved tokens to be spent on it's behalf.

If the `msg_sender` is the same as the account to debit tokens from, the authorization check is bypassed and the function proceeds to update the account's `public_balance` and adds a new `TransparentNote` to the `pending_shields`.

It returns `1` to indicate successful execution.

#include_code shield /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `transfer_public`

This public function enables public transfers between Aztec accounts. The sender's public balance will be debited the specified `amount` and the recipient's public balances will be credited with that amount.

After storage is initialized, the [authorization flow specified above](#authorizing-token-spends) is checked. Then the sender and recipient's balances are updated and saved to storage using the `SafeU120` library.

#include_code transfer_public /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `burn_public`

This public function enables public burning (destroying) of tokens from the sender's public balance.

After storage is initialized, the [authorization flow specified above](#authorizing-token-spends) is checked. Then the sender's public balance and the `total_supply` are updated and saved to storage using the `SafeU120` library.

#include_code burn_public /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

### Private function implementations

Private functions are declared with the `#[aztec(private)]` macro above the function name like so:

```rust
    #[aztec(private)]
    fn redeem_shield(
```

As described in the [execution contexts section above](#execution-contexts), private function logic and transaction information is hidden from the world and is executed on user devices. Private functions update private state, but can pass data to the public execution context (e.g. see the [`unshield`](#unshield) function).

Storage is referenced as `storage.variable`.

#### `redeem_shield`

This private function enables an account to move tokens from a `TransparentNote` in the `pending_shields` mapping to any Aztec account as a `ValueNote` in private `balances`.

Going through the function logic, first the `secret_hash` is generated from the given secret. This ensures that only the entity possessing the secret can use it to redeem the note. Following this, a `TransparentNote` is retrieved from the set, using the provided amount and secret. The note is subsequently removed from the set, allowing it to be redeemed only once. The recipient's private balance is then increased using the `increment` helper function from the `value_note` [library](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/aztec-nr/value-note/src/utils.nr).

The function returns `1` to indicate successful execution.

#include_code redeem_shield /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `unshield`

This private function enables un-shielding of private `ValueNote`s stored in `balances` to any Aztec account's `public_balance`.

After initializing storage, the function checks that the `msg_sender` is authorized to spend tokens. See [the Authorizing token spends section](#authorizing-token-spends) above for more detail--the only difference being that `assert_current_call_valid_authwit` is modified to work specifically in the private context. After the authorization check, the sender's private balance is decreased using the `decrement` helper function for the `value_note` library. Then it stages a public function call on this contract ([`_increase_public_balance`](#_increase_public_balance)) to be executed in the [public execution phase](#execution-contexts) of transaction execution. `_increase_public_balance` is marked as an `internal` function, so can only be called by this token contract.

The function returns `1` to indicate successful execution.

#include_code unshield /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `transfer`

This private function enables private token transfers between Aztec accounts.

After initializing storage, the function checks that the `msg_sender` is authorized to spend tokens. See [the Authorizing token spends section](#authorizing-token-spends) above for more detail--the only difference being that `assert_current_call_valid_authwit` is modified to work specifically in the private context. After authorization, the function gets the current balances for the sender and recipient and decrements and increments them, respectively, using the `value_note` helper functions.

#include_code transfer /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `burn`

This private function enables accounts to privately burn (destroy) tokens.

After initializing storage, the function checks that the `msg_sender` is authorized to spend tokens. Then it gets the sender's current balance and decrements it. Finally it stages a public function call to [`_reduce_total_supply`](#_reduce_total_supply).

#include_code burn /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

### Internal function implementations

Internal functions are functions that can only be called by this contract. The following 3 functions are public functions that are called from the [private execution context](#execution-contexts). Marking these as `internal` ensures that only the desired private functions in this contract are able to call them. Private functions defer execution to public functions because private functions cannot update public state directly.

#### `_initialize`

This function is called via the [constructor](#constructor).

This function sets the creator of the contract (passed as `msg_sender` from the constructor) as the admin and makes them a minter.

#include_code initialize /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `_increase_public_balance`

This function is called from [`unshield`](#unshield). The account's private balance is decremented in `shield` and the public balance is increased in this function.

#include_code increase_public_balance /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `_reduce_total_supply`

This function is called from [`burn`](#burn). The account's private balance is decremented in `burn` and the public `total_supply` is reduced in this function.

#include_code reduce_total_supply /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

### Unconstrained function implementations

Unconstrained functions are similar to `view` functions in Solidity in that they only return information from the contract storage or compute and return data without modifying contract storage.

#### `admin`

A getter function for reading the public `admin` value.

#include_code admin /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `is_minter`

A getter function for checking the value of associated with a `minter` in the public `minters` mapping.

#include_code is_minter /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `total_supply`

A getter function for checking the token `total_supply`.

#include_code total_supply /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `balance_of_private`

A getter function for checking the private balance of the provided Aztec account. Note that the [Private Execution Environment (PXE)](https://github.com/AztecProtocol/aztec-packages/tree/#include_aztec_version/yarn-project/pxe) must have access to the `owner`s decryption keys in order to decrypt their notes.

#include_code balance_of_private /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `balance_of_public`

A getter function for checking the public balance of the provided Aztec account.

#include_code balance_of_public /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

#### `compute_note_hash_and_nullifier`

A getter function to compute the note hash and nullifier for notes in the contract's storage.

This must be included in every contract because it depends on the storage slots, which are defined when we set up storage.

#include_code compute_note_hash_and_nullifier /yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr rust

:::danger
If your contract works with storage (has Storage struct defined), you **MUST** include a `compute_note_hash_and_nullifier` function.
If you don't yet have any private state variables defined put there a placeholder function:

#include_code compute_note_hash_and_nullifier_placeholder /yarn-project/noir-contracts/src/contracts/token_bridge_contract/src/main.nr rust
:::

## Compiling

Now that the contract is complete, you should be able to compile it with the `aztec-cli`. See the [CLI page](../cli/main.md) for instructions on setting it up.

```bash
aztec-cli compile /path/to/token_contracts_folder
```

## Next Steps

### Testing

Review the end to end tests for reference:

https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/yarn-project/end-to-end/src/e2e_token_contract.test.ts

### Token Bridge Contract

The [token bridge tutorial](https://github.com/AztecProtocol/dev-rel/tree/main/tutorials/token-bridge) is a great follow up to this one.

It builds on the Token contract described here and goes into more detail about Aztec contract composability and Ethereum (L1) and Aztec (L2) cross-chain messaging.
