---
sidebar_position: 5
---

# Inter-Layer Calls

## Public<>Private messaging

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.
:::

Private functions work by providing evidence of correct execution generated locally through kernel proofs. Public functions, on the other hand, are able to utilize the latest state to manage updates and perform alterations. As such, public state and private state are in different trees. In a private function you cannot reference or modify public state and vice versa. 

Yet, it should be possible for:
1. private functions to call private or public functions
2. public functions to call private or public functions

For private execution, the user executed methods locally and presents evidence of correct execution as part of their transaction in the form of a kernel proof (generated locally on user device ahead of time). This way, the builder doesn't need to have knowledge of everything happening in the transaction, only the results. However, public functions are executed at the "tip" of the chain (i.e. make use of the latest updates), they can only be done by a builder who is aware of all the changes. Therefore a public function can't be executed locally by the user in the same way a private function is, as it would lead to race conditions, if the user doesn't keep track of the latest updates of the chain. If we were to build this public proof on the latest state, we would encounter problems. How can two different users build proofs at the same time, given that they will be executed one after the other by the sequencer? The simple answer is that they cannot, as race conditions would arise where one of the proofs would be invalidated by the other due to a change in the state root (which would nullify Merkle paths).

As a result, private functions are always executed first, as they are executed on a state $S_i$, where $i \le n$, with $S_n$ representing the current state where the public functions always operate on the current state $S_n$. 

This enables private functions to enqueue calls to public functions. But vice-versa is not true. Since private functions execute first, it cannot "wait" on the results of any of their calls to public functions. Stated differently, any calls made across domains are unilateral in nature. 

The figure below shows the order of function calls on the left-hand side, while the right-hand side shows how the functions will be executed. Notably, the second private function call is independent of the output of the public function and merely occurs after its execution.

![Public - Private Ordering](./images/calls/pvt_pub_ordering.png)

## Private -> Public Messaging
If a private function in an Aztec smart contract wants to call a public function, it gets pushed into a separate public call stack that is enqueued. The private kernel circuit which must prove the execution of the private function(s), then hashes each of the item in the call stack and returns that. The private kernel proof, the public inputs of the private kernel (which contain the hash of the each of the public call stack item) and other transaction data (like enqueued public function calls, new commitments, nullifiers etc) get passed along to the sequencer. Sequencer then picks up the public call stack item and executes each of the functions. The Public VM which executes the methods then verifies that the hash provided by the private kernel matches the current call stack item.

This way, you can destroy your private state and create them in public within the same transaction or indirectly assert constraints on the execution of the private functions with latest data.

### Handling Privacy Leakage and `msg.sender`
In the above design, the sequencer only sees the public part of the call stack along with any new commitments, nullifiers etc that were created in the private transaction i.e. should learns nothing more of the private transaction (such as its origin, execution logic etc).

But what if the enqueued public function makes use of `msg_sender` which is meant to use 

Specifically, when the call stack is passed to the kernel circuit, the kernel should assert the `msg_sender` is 0 and hash appropriately. `msg_sender` could be the contract address too instead of `0`, but it leaks which contract is calling the public method and therefore leaks which contract the user was interacting with in private land.

### Reverts

If the private part of the transaction reverts, then public calls are never enqueued. But if the public part of the transaction reverts, it should still revert the entire transaction i.e. the sequencer should drop the execution results of the private part of the transaction and not include those in the state transitioner smart contract. However, since the sequencer had to execute your transaction, appropriate fee will be charged. Reverting in public causing the whole transaction to be dropped enables existing paradigms of ethereum where your valid transaction can revert because of altered state e.g., trade incurring too much slippage.

## Public -> Private Messaging
Since public functions execute after private functions, it isn't possible for public to call a private function in the same transaction. Nevertheless, it is quite useful for public functions to have a message passing system to private. A public function could add messages to an append only merkle tree to save messages from a public function call, that can later be executed by a private function. Note, only a transaction coming after the one including the message from a public function can consume it. In practice this means that unless you are the sequencer it will not be within the same rollup.

To elaborate, a public function may not have read access to encrypted private state in the note hash tree, but it can write to it. You could create a note in the public domain, compute it's note hash which gets passed to the inputs of the public VM which adds the hash to the note hash tree. The user who wants to redeem the note can add the note preimage to their PXE and then redeem/nullify the note in the private domain at a later time. 

In the picture below, it is worth noting that all data reads performed by private functions are historical in nature, and that private functions are not capable of modifying public storage. Conversely, public functions have the capacity to manipulate private storage (e.g., inserting new commitments, potentially as part of transferring funds from the public domain to the private domain).

![Public - Private Messaging](./images/calls/pub_pvt_messaging.png)
