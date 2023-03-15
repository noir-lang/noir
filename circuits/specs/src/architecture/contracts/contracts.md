# Contracts

## An overview

Much of the functionality of Aztec 3's contracts are inspired by Ethereum contracts; it's just we'll be adding privacy to the mix.

A contract is simply a collection of [functions](./function-types.md) and [state variables](./states-and-storage.md). A function can only make edits to its own contract's state variables. For a function to edit _another_ contract's state, that function must make a call to a function of the other contract. Inter-contract calls are essential for many applications (e.g. An exchange triggering `transferFrom` calls on two token contracts).

A '[transaction](./transactions.md)' is an execution flow, triggered when a user calls a function with a set of inputs. The initially-called function might make calls to other functions, which themselves might make calls to other functions.

In the 'conventional' world of Ethereum, a transaction is completed when the originally-called function eventually completes (having possibly called several other functions along the way). Indeed, conventional programs execute in the order dictated by their code: if a line makes a call to another function, the program will 'jump into' that function and start executing its code, and so-on in a big nesting of calls, until eventually the inner-most call completes and returns some variables. Then program starts unwinding back up the nesting to the originally-called function. That's perhaps not the best mental model for Aztec programs ('transactions')...

As you might have guessed, 'functions' in Aztec's L2 will actually be circuits. Execution of a function will be performed by generating a zk-snark to prove its correct execution with a particular set of inputs. If a function makes a call to another function, a proof of each function's execution must be made _separately_. We need to then 'connect' those two proofs in a way which says "this function made a call to this other function, it passed-in these arguments and received these values in return". To do this, we make use of zk-snark recursion and a special 'callstack'. Notice that if each function must be executed as a standalone lego brick (a proof), then all input arguments and return values of every function call in a transaction must be known ahead of time, so that each proof can be generated separately. That'll be a fun engineering challenge!

This is all expanded-on later in this book. When a function calls another function, we'll add details of the function _being called_ to a callstack[^1]: the contract address, the function data, arguments, return values and a call context. We describe a 'Kernel Snark' which: pops the first proof off the callstack, verifies it, then adds new callstack items for any functions which the first function called. Kernel Snark execution proceeds recursively: the next Kernel Snark will verify the previous Kernel Snark's correctness, then will pop the next proof off the callstack, and add even more callstack items for any functions which this latest function has called. The 'stitching together' of arguments passed to functions and values returned from functions is implicit in this process, since each callstack item contains this information.

This protocol describes 'private states' (a state whose value is known by a single user) and 'public states' (states whose values are visible to everyone). To edit these different states, we need two different kinds of circuits: private circuits and public circuits.

> Whenever we refer to 'contract' in this document, we'll be referring to an Aztec 3 L2 contract (which may comprise both public and private functions). If we're talking about an Ethereum contract, we'll explicitly say something like 'Ethereum contract' or 'L1 contracct' or 'Portal contract'.

[^1]: More 'conventional' callstacks will push a 'return address' to the callstack, so that when the program finishes executing a called function, it knows where to return to, to continue execution of the _calling_ function. In our protocol, we push details of the next function to be called, instead.
