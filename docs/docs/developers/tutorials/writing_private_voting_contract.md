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

- You have followed the [quickstart](../getting_started/quickstart.md) to install `aztec-nargo` and `aztec-sandbox`.
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

We will need the Aztec library to create this contract. In your `Nargo.toml` you should see `[dependencies]` - paste this bellow it.

```toml
[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages", tag="#include_aztec_version", directory="noir-projects/aztec-nr/aztec" }
```

## Initiate the contract and define imports

Go to `main.nr` and delete the sample code. Replace it with this contract initialization:

```rust
contract Voting {

}
```

This defines a contract called `Voter`. Everything will sit inside this block.

Inside this, paste these imports:

#include_code imports noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

We are using various utils within the Aztec library:

- `Context` and `PrivateContext` - exposes things such as the contract address, msg_sender, etc
- `AztecAddress` - A type for storing an address on Aztec
- `FunctionSelector` - Used for computing a selector to call a function
- `Map` - A data storage type for storing candidates with the number of votes they have
- `PublicMutable` - A type of storage, which holds a mutable public value. We'll store votes as PublicMutables

## Set up storage

Under these imports, we need to set up our contract storage.
Define the storage struct like so:

#include_code storage_struct noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

In this contract, we will store three vars:

1. admin, as an Aztec address held in public state
2. tally, as a map with key as the persona and value as the number (in Field) held in public state
3. vote_ended, as a boolean held in public state

## Constructor

The next step is to initialize the contract with a constructor. The constructor will take an address as a parameter and set the admin.

#include_code constructor noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

This function takes the admin argument and writes it to the storage. We are also using this function to set the `vote_ended` boolean as false in the same way.

## Casting a vote privately

For the sake of simplicity, we will have three requirements:

1. Everyone with an Aztec account gets a vote
2. They can only vote once in this contract
3. Who they are is private, but their actual vote is not

To ensure someone only votes once, we will create a nullifier as part of the function call. If they try to vote again, the function will revert as it creates the same nullifier again, which can't be added to the nullifier tree (as that indicates a double spend).

Create a private function called `cast_vote`:

#include_code cast_vote noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

In this function, we do not create a nullifier with the address directly. This would leak privacy as it would be easy to reverse-engineer. We must add some randomness or some form of secret, like [nullifier secrets](../../learn/concepts/accounts/keys.md#nullifier-secrets).

To do this, we make an [oracle call](../contracts/writing_contracts/oracles/main.md) to fetch the caller's secret key, hash it to create a nullifier, and push the nullifier to Aztec. The `secret.high` and `secret.low` values here refer to how we divide a large [Grumpkin scalar](https://github.com/AztecProtocol/aztec-packages/blob/7fb35874eae3f2cad5cb922282a619206573592c/noir/noir_stdlib/src/grumpkin_scalar.nr) value into its higher and lower parts. This allows for faster cryptographic computations so our hash can still be secure but is calculated faster.

After pushing the nullifier, we update the `tally` to reflect this vote. As we know from before, a private function cannot update public state directly, so we are calling a public function.

Create this new public function like this:

#include_code add_to_tally_public noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

The first thing we do here is assert that the vote has not ended.

`assert()` takes two arguments: the assertion, in this case that `storage.vote_ended` is not false, and the error thrown if the assertion fails.

The code after the assertion will only run if the assertion is true. In this snippet, we read the current vote tally at the voteId, add 1 to it, and write this new number to the voteId. The `Field` element allows us to use `+` to add to an integer.

## Getting the number of votes

We will create a function that anyone can call that will return the number of votes at a given vote Id. Paste this in your contract:

#include_code get_vote noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

We set it as `unconstrained` and do not annotate it because it is only reading from state. You can read more about unconstrained functions [here](../../learn/concepts/pxe/acir_simulator.md#unconstrained-functions).

## Allowing an admin to end a voting period

To ensure that only an admin can end a voting period, we can use another `assert()` statement.

Paste this function in your contract:

#include_code end_vote noir-projects/noir-contracts/contracts/easy_private_voting_contract/src/main.nr rust

Here, we are asserting that the `msg_sender()` is equal to the admin stored in public state. We have to create an `AztecAddress` type from the `msg_sender()` in order to do a direct comparison.

## Compiling and deploying

The easiest way to compile the contract is with `aztec-nargo`. Run the following command in the directory with your Nargo.toml file:

```bash
aztec-nargo compile
```

This will create a new directory called `target` and a JSON artifact inside it.

Once it is compiled you can [deploy](../contracts/deploying_contracts/how_to_deploy_contract.md) it to the sandbox. Ensure your [sandbox is running](../sandbox/references/sandbox-reference.md). 

```bash
aztec-builder target -o src/artifacts
```

Once it is compiled you can [deploy](../contracts/deploying_contracts/how_to_deploy_contract.md) it to the sandbox. This is out of scope for this tutorial but you can learn how to do this in the [Aztec.js getting-started guide](../getting_started/aztecjs-getting-started.md).

## Next steps

Now you have learned the foundations of Aztec smart contracts, you can start to play around with some more advanced features. Some ideas:

- Add some more features into this contract, like the admin can distribute votes, people can delegate their votes, or voteIds can have more data like names, descriptions, etc
- Create a frontend for this contract using [Aztec.js](../aztecjs/main.md).
- Go to the [next tutorial](writing_token_contract.md) and learn how to write a token contract
