---
title: Functions
description: This page covers functions, private and public functions composability, as well as their differences.
---

Functions serve as the building blocks of smart contracts. Functions can be either public, ie they can interact with other contracts and the blockchain, or private for internal contract use. Every smart contract also has a private `constructor` function which is called when the contract is deployed. There are also special oracle functions, which can get data from outside of the smart contract. In the context of Aztec, oracles are often used to get user-provided inputs.

On this page, you’ll learn more about:

- How function visibility works in Aztec
- A detailed understanding of public, private, and unconstrained functions, and how to write them
- How constructors work and remain private
- The process of calling functions from within the same smart contract and from different contracts, including calling private functions from private functions, public from public, and even private from public
- What oracles and how Aztec smart contracts might use them

## Visibility

In Aztec there are multiple different types of visibility that can be applied to functions. Namely we have `data visibility` and `function visibility`.

### Data Visibility

Data visibility is used to describe whether the data (or state) used in a function is generally accessible (public) or on a need to know basis (private). Functions with public data visibility are executed by the sequencer, and functions with private data visibility are executed by the user. For more information on why this is the case, see [communication](../../../learn/concepts/communication/public_private_calls/main.md).

In the following sections, we are going to see how these two "types" co-exists and interact.

### Function visibility

This is the kind of visibility you are more used to seeing in Solidity and more traditional programming languages. It is used to describe whether a function is callable from other contracts, or only from within the same contract.

By default, all functions are callable from other contracts, similarly to the Solidity `public` visibility. To make them only callable from the contract itself, you can mark them as `internal`. Contrary to solidity, we don't have the `external` nor `private` keywords. `external` since it is limited usage when we don't support inheritance, and `private` since we don't support inheritance and it would also be confusing with multiple types of `private`.

A good place to use `internal` is when you want a private function to be able to alter public state. As mentioned above, private functions cannot do this directly. They are able to call public functions and by making these internal we can ensure that this state manipulating function is only callable from our private function.

:::danger
Note that non-internal functions could be used directly as an entry-point, which currently means that the `msg_sender` would be `0`, so for now, using address `0` as a burn address is not recommended.
:::

## Mutability

Currently, any function is "mutable" in the sense that it might alter state. In the future, we will support static calls, similarly to EVM. A static call is essentially a call that does not alter state (it keeps state static). This is useful for when you want to call a function in a separate contract, but ensure that it cannot alter state, or call other functions that might alter state (such as re-entering).

Similarly, a special case of a mutating call is the `delegatecall` where the function executed might not be in the same contract as the state being altered. It is at this moment, not certain if `delegatecall`s should become a fully fledged feature.

:::danger No `staticcall` or `delegatecall` support
While `staticcall` and `delegatecall` both have flags in the call context, they are currently not supported and will not behave as one would expect if usage is attempted.
:::

## `constructor`

- A special `constructor` function MUST be declared within a contract's scope.
- A constructor doesn't have a name, because its purpose is clear: to initialize contract state.
- In Aztec terminology, a constructor is always a '`private` function' (i.e. it cannot be a `public` function).
- A constructor behaves almost identically to any other function. It's just important for Aztec to be able to identify this function as special: it may only be called once, and will not be deployed as part of the contract.

An example of a somewhat boring constructor is as follows:

#include_code empty-constructor /yarn-project/noir-contracts/contracts/test_contract/src/main.nr rust

Although you can have a constructor that does nothing, you might want to do something with it, such as setting the deployer as an owner.

#include_code constructor /yarn-project/noir-contracts/contracts/escrow_contract/src/main.nr rust

:::danger
It is not possible to call public functions from within a constructor. Beware that the compiler will not throw an error if you do, but the execution will fail!
:::

## `Public` Functions

A public function is executed by the sequencer and has access to a state model that is very similar to that of the EVM and Ethereum. Even though they work in an EVM-like model for public transactions, they are able to write data into private storage that can be consumed later by a private function.

To create a public function you can annotate it with the `#[aztec(public)]` attribute. This will make the [public context](./context.mdx#public-context) available within your current function's execution scope.

#include_code set_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

## `Private` Functions

As alluded to earlier, a private function operates on private information, and is executed by the user. To tell the compiler that this is the kind of function we are creating annotate it with the `#[aztec(private)]` attribute. This will make the [private context](./context.mdx#private-context-broken-down) available within your current function's execution scope.

#include_code redeem_shield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

## `unconstrained` functions

Unconstrained functions are an underlying part of Noir - a deeper explanation can be found [here](https://noir-lang.org/docs/noir/syntax/unconstrained). But in short, they are functions which are not directly constrained and therefore should be seen as untrusted! That they are untrusted means that, for security, the developer must make sure to constrain them when used!

Beyond using them inside your other functions, they are convenient for providing an interface that reads storage, applies logic and returns values to a UI or test. Below is a snippet from exposing the `balance_of_private` function from a token implementation, which allows a user to easily read their balance, similar to the `balanceOf` function in the ERC20 standard.

#include_code balance_of_private /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

:::info
Note, that unconstrained functions can have access to both public and private data when executed on the user's device. This is possible since it is not actually part of the circuits that are executed in contract execution.
:::

## Oracle functions

An oracle is something that allows us to get data from the outside world into our contracts. The most widely-known types of oracles in blockchain systems are probably Chainlink price feeds, which allow us to get the price of an asset in USD taking non-blockchain data into account.

While this is one type of oracle, the more general oracle, allows us to get "some" data into the contract. In the context of oracle functions or oracle calls in Aztec, it can essentially be seen as user-provided arguments, that can be embedded at any point in the circuit, and thus don't need to be an input parameter.

**Why is this useful? Why don't just pass them as input parameters?**
In the world of EVM, you would just read the values directly from storage and call it a day. However, when we are working with circuits for private execution, this becomes more tricky as you cannot just read the storage directly from your state tree, only commitments sit in there 😱. The pre-images (content) of your notes need to be provided to the function to prove that you actually allowed to spend them.

You could of course provide them to your function as inputs, but then functions that have different underlying notes would end up with different function signatures and thus selectors. This means that integrating with many different tokens (with different underlying notes) would become a pain for the developers, see some of the motivation behind [EIP-4626](https://eips.ethereum.org/EIPS/eip-4626) for similar case in EVM.

If we are instead fetching the notes using an oracle call, we can keep the function signature independent of the underlying notes and thus make it much easier to integrate with! A similar idea, but applied to the authentication mechanism is used for the Authentication Witnesses that allow us to have a single function signature for any wallet implementation making integrations a breeze, see [AuthWit](../../wallets/main#authorizing-actions) for more information on this.

Oracles introduce **non-determinism** into a circuit, and thus are `unconstrained`. It is important that any information that is injected into a circuit through an oracle is later constrained for correctness. Otherwise, the circuit will be **under-constrained** and potentially insecure!

`Aztec.nr` has a module dedicated to its oracles. If you are interested, you can view them by following the link below:
#include_code oracles-module /yarn-project/aztec-nr/aztec/src/oracle.nr rust

You can learn how to use oracles in your smart contracts [here](../syntax/oracles.md).

## Calling functions from other functions

### Private -> Private

In Aztec Private to Private function calls are handled by the [private kernel circuit](../../../learn/concepts/circuits/kernels/private_kernel.md), and take place on the user's device.
Behind the scenes, the `Private Execution Environment (PXE)` (the beating heart of Aztec that runs in your wallet) will execute all of the functions in the desired order "simulating" them in sequence. For example, a very common use-case of Private to Private interaction is calling another private function from an `account contract` (Account contracts are a general concept, more information about them can be found [here](../../wallets/writing_an_account_contract.md)).

Take, for example, the following call stack:

```
AccountContract::entrypoint
    |-> Foo::example_call
        | -> Bar::nested_call
    |-> Baz::example_call
```

In the example above the Account Contract has been instructed to call two external functions. In the first function all, to `Foo::example_call` a further nested call is performed to `Bar::nested_call`. Finally the account contract makes one last call to `Baz::example_call`.

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

Those of you who have written circuits before may see an issue here. The account contract, contract `Foo`, `Bar` and `Baz` are all distinct circuits, which do not know anything about each other. How is it possible to use a value from contract `Bar` in contract `Foo`? This value will not be constrained.

This is where the `kernel` circuit comes in. Once the execution of all of our functions has completed, we can just prove the execution of each of them independently. It is the job of the `kernel` circuit to constrain that the input parameters in a cross function call are correct, as well as the return values. The kernel will constrain that the value returned from `Foo::example_call` is the same value that is returned from `Bar::nested_call`, it will also be able to constrain the value returned by `Bar::nested_call` is the inputs to `Foo::example_call` + 1.

The orchestration of these calls has an added benefit. All of the nested calls are **recursively proven**. This means that the kernel circuit essentially gobbles up each of our function's execution proofs. Condensing the size of the final proof to just be one.

<!-- TODO(md): include a diagram displaying how the mental model of a kernel interaction works -->

With this intuition in place, lets see how we actually perform the call. To make things easier, we can make a small struct that wraps the calls to something as seen in the `token_interface`s burn function below. This struct is just providing us a clean way to call function, but we could also just call the function directly as it is done in this function.

:::info
Note that the function selector is computed using one of the oracles from earlier, and that the first `Field` is wrapped in parenthesis. Structs are outlined in tuple-form for selector computation, so they are wrapped in parenthesis--`AztecAddress` becomes `(Field)`.
:::

#include_code private_burn_interface /yarn-project/noir-contracts/contracts/token_bridge_contract/src/token_interface.nr rust

Using this interface, we can then call it as seen below. All the way down at the bottom we can see that we are calling the `burn` function from the `token_interface` struct.

The following snippet is from a token bridge that is burning the underlying token and creating a message for L1 to mint some assets to the `recipient` on Ethereum.

#include_code exit_to_l1_private /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

### Public -> Public

The public execution environment in Aztec takes place on the sequencer through a [Public VM](../../../learn/concepts/hybrid_state/public_vm.md). This execution model is conceptually much simpler than the private transaction model as code is executed and proven on the sequencer.

Using the same example code and call stack from the section [above](#private----private-function-calls), we will walk through how it gets executed in public.

The first key difference is that public functions are not compiled to circuits, rather they are compiled to `Aztec Bytecode` (might also be referred to as brillig).

This bytecode is run by the sequencer in the `Aztec VM`, which is in turn proven by the [`Aztec VM circuit`](../../../learn/concepts/hybrid_state/public_vm.md).
The mental model for public execution carries many of the same idea as are carried by Ethereum. Programs are compiled into a series of opcodes (known as bytecode). This bytecode is then executed. The extra step for the Aztec VM is that each opcode is then proven for correctness.

Calling a public function from another public function is quite similar to what we saw for private to private, with the keyword private swapped for public.

#include_code public_burn_interface /yarn-project/noir-contracts/contracts/token_bridge_contract/src/token_interface.nr rust
#include_code exit_to_l1_public /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

### Private -> Public

As discussed above, private function execution and calls take place on the user's device, while public function execution and calls take place on a sequencer, in two different places at two different times, it is natural to question how we can achieve composability between the two. The solution is asynchronicity. Further reading can be found in the foundational concepts [here](../../../learn/concepts/communication/public_private_calls/main.md)).

Private function execution takes place on the users device, where it keeps track of any public function calls that have been made. Whenever private execution completes, and a kernel proof is produced, the transaction sent to the network will include all of the public calls that were dispatched.
When the sequencer receives the messages, it will take over and execute the public parts of the transaction.

As a consequence a private function _CANNOT_ accept a return value from a public function. It can only dispatch it.

The code required to dispatch a public function call from a private function is actually quite similar to private to private calls. As an example, we will look at the token contract, where users can unshield assets from private to public domain, essentially a transfer from a private account to a public one (often used for depositing privately into DeFi etc).

#include_code unshield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust
#include_code increase_public_balance /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

As we can see above, the private to public transaction flow looks very similar to the others in snippets, with the practicality being a bit different behind the scenes.

<!-- TODO: drawing illustrating how the proof is handed off -->

### Public -> Private

Wait, I thought you said we could not do this? Well, you are right, we cannot do this directly. However, we can do this indirectly if we are a little cheeky.

While we cannot directly call a private function, we can indirectly call it by adding a commitment to the note hash tree. This commitment can then be consumed by a private function later, to "finish" the execution. So while it is not practically a call, we can ensure that it could only happen as an effect of a public function call, which is still pretty useful.

In the snippet below, we insert a custom note, the transparent note, into the commitments tree from public such that it can later be consumed in private.

#include_code shield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

If you recall the `redeem_shield` from back in the [private function section](#private-functions), you might remember it removing a `TransparentNote` from `pending_shields`. This is the note that we just inserted from public!

#include_code redeem_shield /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

When the note is removed, it emits a nullifier so that it cannot be used again. This nullifier is then added to the note hash tree, and can be used to prove that the note was removed from the pending shields. Interestingly, we can generate the nullifier such that no-one who saw the public execution will know that it have been consumed. When sending messages between L1 and L2 in [portals](../portals/main.md) we are going to see this pattern again.

:::danger
Something to be mindful of when inserting from public. Everyone can see the insertion and what happens in public, so if you are including a secret directly anyone would be able to see it. This is why the hash of the secret is used in the snippet above (`secret_hash`).
:::

---

## Deep dive

Below, we go more into depth of what is happening under the hood when you create a function in Aztec.nr and what the attributes are really doing.

### Function type attributes explained.

Aztec.nr uses an attribute system to annotate a function's type. Annotating a function with the `#[aztec(private)]` attribute tells the framework that this will be a private function that will be executed on a users device. Thus the compiler will create a circuit to define this function.

However; `#aztec(private)` is just syntactic sugar. At compile time, the framework inserts code that allows the function to interact with the [kernel](../../../learn/concepts/circuits/kernels/private_kernel.md).

To help illustrate how this interacts with the internals of Aztec and its kernel circuits, we can take an example private function, and explore what it looks like after Aztec.nr's macro expansion.

#### Before expansion

#include_code simple_macro_example /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### After expansion

#include_code simple_macro_example_expanded /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### The expansion broken down?

Viewing the expanded noir contract uncovers a lot about how noir contracts interact with the [kernel](../../../learn/concepts/circuits/kernels/private_kernel.md). To aid with developing intuition, we will break down each inserted line.

**Receiving context from the kernel.**
#include_code context-example-inputs /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

Private function calls are able to interact with each other through orchestration from within the [kernel circuit](../../../learn/concepts/circuits/kernels/private_kernel.md). The kernel circuit forwards information to each app circuit. This information then becomes part of the private context.
For example, within each circuit we can access some global variables. To access them we can call `context.chain_id()`. The value of this chain ID comes from the values passed into the circuit from the kernel.

The kernel can then check that all of the values passed to each circuit in a function call are the same.

**Returning the context to the kernel.**
#include_code context-example-return /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

Just as the kernel passes information into the app circuits, the application must return information about the executed app back to the kernel. This is done through a rigid structure we call the `PrivateCircuitPublicInputs`.

> _Why is it called the `PrivateCircuitPublicInputs`_  
> It is commonly asked why the return values of a function in a circuit are labeled as the `Public Inputs`. Common intuition from other programming paradigms suggests that the return values and public inputs should be distinct.  
> However; In the eyes of the circuit, anything that is publicly viewable (or checkable) is a public input. Hence in this case, the return values are also public inputs.

This structure contains a host of information about the executed program. It will contain any newly created nullifiers, any messages to be sent to l2 and most importantly it will contain the actual return values of the function!

**Hashing the function inputs.**
#include_code context-example-hasher /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

_What is the hasher and why is it needed?_

Inside the kernel circuits, the inputs to functions are reduced to a single value; the inputs hash. This prevents the need for multiple different kernel circuits; each supporting differing numbers of inputs. The hasher abstraction that allows us to create an array of all of the inputs that can be reduced to a single value.

**Creating the function's context.**
#include_code context-example-context /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

Each Aztec function has access to a [context](./context.mdx) object. This object although ergonomically a global variable, is local. It is initialized from the inputs provided by the kernel, and a hash of the function's inputs.

#include_code context-example-context-return /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

As previously mentioned we use the kernel to pass information between circuits. This means that the return values of functions must also be passed to the kernel (where they can be later passed on to another function).
We achieve this by pushing return values to the execution context, which we then pass to the kernel.

**Making the contract's storage available**
#include_code storage-example-context /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

When a [`Storage` struct](./storage/main.md) is declared within a contract, the `storage` keyword is made available. As shown in the macro expansion above, this calls the init function on the storage struct with the current function's context.

Any state variables declared in the `Storage` struct can now be accessed as normal struct members.

**Returning the function context to the kernel.**
#include_code context-example-finish /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

This function takes the application context, and converts it into the `PrivateCircuitPublicInputs` structure. This structure is then passed to the kernel circuit.
