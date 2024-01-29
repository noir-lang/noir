import Image from '@theme/IdealImage';

# Writing a private voting smart contract in Aztec.nr

In this tutorial we will go through writing a very simple private voting smart contract in Aztec.nr. You will learn about private functions, public functions, composability between them, state management and creatively using nullifiers to prevent people from voting twice!

We will build this:

<Image img={require('/img/tutorials/voting_flow.png')} />

- The contract will be initialized with an admin, stored publicly
- A voter can vote privately, which will call a public function and update the votes publicly
- The admin can end the voting period, which is a public boolean

To keep things simple, we won't create ballots or allow for delegate voting.

## Prerequisites

- You have followed the [quickstart](../getting_started/quickstart.md) to install `aztec-nargo`, `aztec-cli` and `aztec-sandbox`.
- Running Aztec Sandbox

## Set up a project

First, create a new contract project with `aztec-nargo`.

```bash
aztec-nargo new --contract private_voting
```

Your file structure should look something like this:

```tree
.
| | |--private_voting
| | |  |--src
| | |  |  |--main.nr
| | |  |--Nargo.toml
```

The file `main.nr` will soon turn into our smart contract!

We will need the Aztec library to create this contract. Add the following content to `Nargo.toml`:

```toml
[package]
name = "private_voting"
type = "contract"
authors = [""]
compiler_version = ">=0.18.0"

[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
```

## Initiate the contract and define imports

Go to `main.nr` and delete the sample code. Replace it with this contract initialization:

```rust
contract Voting {

}
```

This defines a contract called `Voter`. Everything will sit inside this block.

Inside this, paste these imports:

#include_code imports yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

We are using various utils within the Aztec library:

- `context` - exposes things such as the contract address, msg_sender, etc
- `context.request_nullifier_secret_key` - get your secret key to help us create a randomized nullifier
- `FunctionSelector::from_signature` - compute a function selector from signature so we can call functions from other functions
- `state_vars::{ map::Map, public_state::PublicState, }` - we will use a Map to store the votes (key = voteId, value = number of votes), and PublicState to hold our public values that we mentioned earlier
- `types::type_serialization::{..}` - various serialization methods for defining how to use these types
- `types::address::{AztecAddress},` - our admin will be held as an address
- `constants::EMPTY_NULLIFIED_COMMITMENT,` - this will come in useful when creating our nullifier

## Set up storage

Under these imports, we need to set up our contract storage. This is done in two steps:

1. Storage struct
2. Storage impl block with init function

Define the storage struct like so:

#include_code storage_struct yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

In this contract, we will store three vars:

1. admin, as an Aztec address held in public state
2. tally, as a map with key as the persona and value as the number (in Field) held in public state
3. voteEnded, as a boolean held in public state

Under the struct, define the impl block like this:

#include_code storage_impl yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

The `impl` block must define one function `init` that explains how to access and manipulate our variables. We pass context, a storage slot, and serialization methods we imported earlier.

This `init` function will be called every time we access `storage` in our functions.

## Constructor

The next step is to initialize the contract with a constructor. The constructor will take an address as a parameter and set the admin.

All constructors must be private, and because the admin is in public storage, we cannot directly update it from the constructor. You can find more information about this [here](../../learn/concepts/communication/public_private_calls/main.md).

Therefore our constructor must call a public function by using `context.call_public_function()`. Paste this under the `impl` storage block:

#include_code constructor yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

`context.call_public_function()` takes three arguments:

1. The contract address whose method we want to call
2. The selector of the function to call (we can use `FunctionSelector::from_signature(...)` for this)
3. The arguments of the function (we pass the `admin`)

We now need to write the `_initialize()` function:

#include_code initialize yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

This function takes the admin argument and writes it to the storage. We are also using this function to set the `voteEnded` boolean as false in the same way.

This function is set as `internal` so that it can only be called from within the contract. This stops anyone from setting a new admin.

## Casting a vote privately

For the sake of simplicity, we will have three requirements:

1. Everyone with an Aztec account gets a vote
2. They can only vote once in this contract
3. Who they are is private, but their actual vote is not

To ensure someone only votes once, we will create a nullifier as part of the function call. If they try to vote again, the function will revert as it creates the same nullifier again, which can't be added to the nullifier tree (as that indicates a double spend).

Create a private function called `cast_vote`:

#include_code cast_vote yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

In this function, we do not create a nullifier with the address directly. This would leak privacy as it would be easy to reverse-engineer. We must add some randomness or some form of secret, like [nullifier secrets](../../learn/concepts/accounts/keys.md#nullifier-secrets).

To do this, we make an [oracle call](../contracts/syntax/functions.md#oracle-functions) to fetch the caller's secret key, hash it to create a nullifier, and push the nullifier to Aztec. The `secret.high` and `secret.low` values here refer to how we divide a large [Grumpkin scalar](https://github.com/AztecProtocol/aztec-packages/blob/7fb35874eae3f2cad5cb922282a619206573592c/noir/noir_stdlib/src/grumpkin_scalar.nr) value into its higher and lower parts. This allows for faster cryptographic computations so our hash can still be secure but is calculated faster.

After pushing the nullifier, we update the `tally` to reflect this vote. As we know from before, a private function cannot update public state directly, so we are calling a public function.

Create this new public function like this:

#include_code add_to_tally_public yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

The first thing we do here is assert that the vote has not ended.

`assert()` takes two arguments: the assertion, in this case that `storage.voteEnded` is not false, and the error thrown if the assertion fails.

The code after the assertion will only run if the assertion is true. In this snippet, we read the current vote tally at the voteId, add 1 to it, and write this new number to the voteId. The `Field` element allows us to use `+` to add to an integer.

## Getting the number of votes

We will create a function that anyone can call that will return the number of votes at a given vote Id. Paste this in your contract:

#include_code get_vote yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

We set it as `unconstrained` and do not annotate it because it is only reading from state. You can read more about unconstrained functions [here](../../learn/concepts/pxe/acir_simulator.md#unconstrained-functions).

## Allowing an admin to end a voting period

To ensure that only an admin can end a voting period, we can use another `assert()` statement.

Paste this function in your contract:

#include_code end_vote yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

Here, we are asserting that the `msg_sender()` is equal to the admin stored in public state. We have to create an `AztecAddress` type from the `msg_sender()` in order to do a direct comparison.

## compute_note_hash_and_nullifier

Every Aztec contract that has storage must have a `compute_note_hash_and_nullifier()` function. If you try to compile without this function, you will get an error. This is explained in more detail [here](../contracts/resources/common_patterns/main.md#working-with-compute_note_hash_and_nullifier).

At the end of the contract, paste this:

#include_code compute_note_hash_and_nullifier yarn-project/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

We can simply return `[0,0,0,0]` because we are not creating any notes in our contract.

## Compiling and deploying

The easiest way to compile the contract is with `aztec-nargo`. Run the following command in the directory with your Nargo.toml file:

```bash
aztec-nargo compile
```

This will create a new directory called `target` and a JSON artifact inside it. To optionally create a typescript interface, run:

```bash
aztec-cli codegen target -o src/artifacts --ts
```

Once it is compiled you can [deploy](../contracts/deploying.md) it to the sandbox. Ensure your [sandbox is running](../cli/sandbox-reference.md) and run this in the same dir as before:

```bash
aztec-cli deploy ./target/Voting.json --args $ADMIN_ADDRESS
```

The constructor takes an address as an argument to set the admin, so you can use an address that is deployed with the sandbox - check the sandbox terminal or run `aztec-cli get-accounts`.

You should see a success message with the contract address. Now we can start calling functions!

Cast a vote like this:

```bash
aztec-cli send cast_vote --contract-artifact ./target/Voting.json --contract-address $CONTRACT_ADDRESS --args 1 --private-key $PRIVATE_KEY
```

You can get the contract address from the sandbox terminal or the message printed when you deployed the contract. You can also get a private key from the sandbox terminal, or generate one with `aztec-cli generate-private-key`.

This should return a `mined` success message.

You can now try running this command again to ensure our nullifier works.

Get the number of votes like this:

```bash
aztec-cli call get_vote --contract-artifact ./target/Voting.json --contract-address $CONTRACT_ADDRESS --args 1
```

This should return `1n`.

You can follow this pattern to test `end_vote()` and access control of other functions. Find more information about calling functions from the CLI [here](../cli/cli-commands.md).

## Next steps

Now you have learned the foundations of Aztec smart contracts, you can start to play around with some more advanced features. Some ideas:

- Add some more features into this contract, like the admin can distribute votes, people can delegate their votes, or voteIds can have more data like names, descriptions, etc
- Create a frontend for this contract using [Aztec.js](../aztecjs/main.md).
- Go to the [next tutorial](writing_token_contract.md) and learn how to write a token contract
