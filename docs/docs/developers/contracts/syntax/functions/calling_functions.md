---
title: Calling Functions from Other Functions
---

This page talks about how functions call other functions. For a more hands-on guide into calling functions from other functions, follow the [token tutorial](../../../tutorials/writing_token_contract.md).

### Private -> Private

In Aztec Private to Private function calls are handled by the [private kernel circuit](../../../../learn/concepts/circuits/kernels/private_kernel.md), and take place on the user's device.
Behind the scenes, the `Private Execution Environment (PXE)` (the beating heart of Aztec that runs in your wallet) will execute all of the functions in the desired order "simulating" them in sequence. For example, a very common use-case of private-to-private interaction is calling a private function on another contract from an `account contract` (Account contracts are a general concept, more information about them can be found [here](../../../wallets/writing_an_account_contract.md)).

Take, for example, the following call stack:

```
AccountContract::entrypoint
    |-> Foo::example_call
        | -> Bar::nested_call
    |-> Baz::example_call
```

In the example above the Account Contract has been instructed to call two external functions. In the first function all, to `Foo::example_call` a further nested call is performed to `Bar::nested_call`. Finally the Account Contract makes one last call to `Baz::example_call`.

Lets further illustrate what these examples could look like

<!-- TODO: should these move into the docs examples -->

```rust
// Foo contains a singular function that returns the result of Bar::nested_call
contract Foo {
    #[aztec(private)]
    fn example_call(input: Field) -> pub Field {
        Bar::at(<bar_address>).nested_call(input)
    }
}

// Bar contains a singular function that returns a `input + 1`
contract Bar {
    #[aztec(private)]
    fn nested_call(input: Field) -> pub Field {
        input + 1
    }
}

// Baz contains a singular function that simply returns `10`
contract Baz {
    #[aztec(private)]
    fn example_call() -> pub Field {
        10
    }
}
```

When simulating the following call stack, we can expect execution flow to continue procedurally. The simulator will begin at the account contract's entry point, find a call to `Foo::example_call`, then begin to execute the code there. When the simulator executes the code in contract `Foo`, it will find the further nested call to contract `Bar::nested_call`. It will execute the code in `Bar`, bringing the return value back to contract `Foo`.
The same process will be followed for contract `Baz`.

So far the provided example is identical to other executions. Ethereum execution occurs in a similar way, during execution the EVM will execute instructions until it reaches an external call, where it will hop into a new context and execute code there, returning back when it is complete, bringing with it return values from the foreign execution.

Aztec differs from Ethereum in that these function calls are really executing zk programs (or circuits). The account contract, contract `Foo`, `Bar` and `Baz` are all distinct circuits, which are not directly aware of one another in the way that Ethereum contracts are. How is it possible to use a value from contract `Bar` in contract `Foo`? `Foo` cannot guarantee claims about the execution of `Bar`.

This is where the `kernel` circuit comes in. Once the execution of all of the contract functions has completed, it can prove the execution of each of them independently. It is the job of the `kernel` circuit to constrain that the input parameters in a cross function call are correct, as well as the return values. The kernel will constrain that the value returned from `Foo::example_call` is the same value that is returned from `Bar::nested_call`, it will also be able to guarantee the value returned by `Bar::nested_call` is the inputs to `Foo::example_call` + 1.

The orchestration of these calls has an added benefit. All of the nested calls are **recursively proven**. This means that the kernel circuit essentially aggregates each of our function's execution proofs, resulting in one proof that proves all function execution.

<!-- TODO(md): include a diagram displaying how the mental model of a kernel interaction works -->

With this intuition in place, lets see how we actually perform the call. To make things easier, we can make a small struct that wraps the calls to something as seen in the token interface `burn` function below. This struct is providing us a clean way to call function, but we could also just call the function directly as it is done in this function.

:::info
Note that the function selector is computed using Oracles, and that the first `Field` is wrapped in parenthesis. Structs are outlined in tuple-form for selector computation, so they are wrapped in parenthesis--e.g. `AztecAddress` becomes `(Field)`.
:::

#include_code private_burn_interface /yarn-project/noir-contracts/contracts/token_bridge_contract/src/token_interface.nr rust

Using this interface, we can then call it as seen below. All the way down at the bottom we can see that we are calling the `burn` function from the `token_interface` struct.

The following snippet is from a token bridge that is burning the underlying token and creating a message for L1 to mint some assets to the `recipient` on Ethereum.

#include_code exit_to_l1_private /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

### Public -> Public

The public execution environment in Aztec takes place on the sequencer through a [Public VM](../../../../learn/concepts/hybrid_state/public_vm.md). This execution model is conceptually much simpler than the private transaction model as code is executed and proven on the sequencer.

Using the same example code and call stack from the section [above](#private---private-function-calls), we will walk through how it gets executed in public.

The first key difference is that public functions are not compiled to circuits, rather they are compiled to `Aztec Bytecode`.

This bytecode is run by the sequencer in the `Aztec VM`, which is in turn proven by the [`Aztec VM circuit`](../../../../learn/concepts/hybrid_state/public_vm.md).
The mental model for public execution carries many of the same idea as are carried by Ethereum. Programs are compiled into a series of opcodes (known as bytecode). This bytecode is then executed. The extra step for the Aztec VM is that each opcode is then proven for correctness.

Calling a public function from another public function is quite similar to what we saw for private to private, with the keyword private swapped for public.

#include_code public_burn_interface /yarn-project/noir-contracts/contracts/token_bridge_contract/src/token_interface.nr rust
#include_code exit_to_l1_public /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

### Private -> Public

As discussed above, private function execution and calls take place on the user's device, while public function execution and calls take place on a sequencer, in two different places at two different times. We can achieve composability between the two contexts via asynchronicity. Further reading can be found in the concepts [here](../../../../learn/concepts/communication/public_private_calls/main.md).

Private function execution takes place on the users device, where it keeps track of any public function calls that have been made. Whenever private execution completes, and a kernel proof is produced, the transaction sent to the network will include all of the public calls that were dispatched.
When the sequencer receives the messages, it will execute the public parts of the transaction.

As a consequence a private function _CANNOT_ accept a return value from a public function. It can only dispatch it.

The code required to dispatch a public function call from a private function is similar to private-to-private calls. As an example, we will look at the token contract, where users can unshield assets from private to public domain. This is essentially a transfer from a private account to a public one (often used for depositing privately into DeFi etc).

#include_code unshield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust
#include_code increase_public_balance /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

As we can see above, in the code the private to public transaction flow looks very similar to the others in snippets, with the transaction data flow being a bit different behind the scenes.

<!-- TODO: drawing illustrating how the proof is handed off -->

### Public -> Private


While we cannot directly call a private function from a public function, we can indirectly call it by adding a commitment to the note hash tree. This commitment can then be consumed by a private function later, to "finish" the execution. So while it is not practically a call, we can ensure that it could only happen as an effect of a public function call, which is still useful.

In the snippet below, we insert a custom note, the transparent note, into the commitments tree from public such that it can later be consumed in private.

#include_code shield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

If you recall the `redeem_shield` from back in the [private function section](./public_private_unconstrained.md#private-functions), you might remember it removing a `TransparentNote` from `pending_shields`. This is the note that we just inserted from public!

#include_code redeem_shield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

When the note is removed, it emits a nullifier so that it cannot be used again. This nullifier is then added to the nullifier tree, and can be used to prove that the note was removed from the pending shields. Interestingly, we can generate the nullifier such that no-one who saw the public execution will know that it have been consumed. When sending messages between L1 and L2 in [portals](./../../../../learn/concepts/communication/cross_chain_calls.md) we are going to see this pattern again.

:::danger
Something to be mindful of when inserting from public. Everyone can see the insertion and what happens in public, so if you are including a secret directly anyone would be able to see it. This is why the hash of the secret is used in the snippet above (`secret_hash`).
:::

