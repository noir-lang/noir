# THIS PAGE IS OUT OF DATE

# L1 --> L2 / L2 --> L1 calls


Although the dedicated pages on each circuit contain ABIs & logic relating to interactions with L1, it's a pretty complicated subject, so this is a page which focuses on L1 interaction logic alone.


## L1 --> L2 calls

See the [deposit](../../examples/erc20/deposit.md) example for more details.


## L2 --> L1 calls


Calling L1 is tricky. Aztec Connect did a lot of the thinking, and here we're just trying to generalise those ideas beyond the single 'defi interaction' interface.

Firstly, in order for a contract (or user) to make a call to an L1 function, they need an L1 address. That's one of the main purposes of the Portal Contract; to provide an Aztec L2 contract with an L1 contract address through which it can make calls to other L1 functions. So an L2 contract MUST make L1 calls _through_ its Portal Contract. I.e. _all_ calls from an L2 contract will be _to_ that contract's Portal Contract. It's then up to the logic of the Portal Contract to forward calls to other L1 contracts' functions.

Now, an L2 circuit might both make state changes _and_ make calls to L1. The L2 circuit's execution MUST NOT be considered 'final', until the L1 calls have been mined, as only then can we know if the L1 calls were successful/unsuccessful. That means any state changes which the L2 circuit is proposing to make (to either the `privateDataTree` or the `publicDataTree`) MUST NOT be included (as final states) in the trees until the L1 execution completes. Furthermore, the L2 circuit might also be 'awaiting' some values or on-chain state updates from the L1 call(s) before being able to insert new L2 states into the trees.

> An example is Aztec Connect's defi interactions. You can think of the 'defi deposit' circuit as calling a bridge on L1. We _want_ to create states containing the user's share of funds _returned_ by the L1 bridge. But the defi deposit circuit cannot make such state updates. So it instead creates a partial commitment (which in Aztec 3 we can add to a state variable in the public data tree). It's only after the L1 tx is completed that we know the value to inject into that partial (or 'pending') state. The claim circuit then injects the value and adds the _finalised_ state to the privateDataTree. See more on this example [here](./function-types.md#examples).

As a generalisation for Aztec 3, we we can think of the claim circuit as a 'callback function', which is executed after the result of the L1 tx is known.

We outline a general protocol below. Here are the main differences from Aztec Connect:
- For each L1 call, two 'callback functions' are submitted; one to be executed upon success, and one upon failure of the L1 tx.
- Whereas 'defi deposits' are summed in the rollup circuit in Aztec Connect, that cannot happen in Aztec 3, because app specific logic must not live in a rollup circuit. Instead, such app-specific logic would need to be in a public circuit. See the dedicated example [TODO: EXAMPLE] for more details.
- In Aztec Connect, the rollup circuit expects a defi interaction note to have a specific format (that of a defi interaction). In Aztec 3, the rollup circuit has no knowledge of the number of values contained in an L1 result; it deals with already-compressed (hashed) leaves which contain callback and result data.
- In Aztec Connect, there's no callback in case a withdrawal's L1 call fails. If it fails, a user has spent money on L2, but hasn't received money on L1 (they've lost money effectively). In Aztec 3, we could specify a failure callback which could reinstate the user's L2 balance (for example).



### Protocol for L2 --> L1 calls.


#### Overview

- L2 function makes call to L1
- Two callback hashes (see below) get encoded into a single leaf of the `l1ResultsTree` - one for success and one for failure.
  - A callback hash contains a:
    - function selector
    - a set of `emittedPublicInputs` that must be emitted by the callback.
  - Leaves without data relating to the success/failure of the L1 call are considered "pending".
- The RollupProcessor.sol makes an L1 call for every callstack item on the `l1Callstack`.
- The return data of each L1 call is sha256-hashed into an `l1ResultHash` (singular).
  - If the L1 call failed, this is also encoded in the `l1ResultHash`.
- Those hashes are all sha256-hashed together into a single `l1ResultsHash` (plural), to save on storage costs. The RollupProcessor.sol stores this for reference, so it may do a comparison against the next rollup.
- In the very next rollup, the rollup provider is _forced_ to execute an "L1 Results Copier Circuit" which copies all L1 results of the previous rollup over to L2, so that L2 circuits may access the results more easily.
  - The circuit "finalises" each "pending" leaf of the `l1ResultsTree`  by adding the `l1ResultHash` to the leaf. The leaf will be structured in such a way that a success result will be stored against the success callback.
  - The RollupProcessor.sol will check the "L1 Results Copier Circuit" was executed correctly, by checking:
    - The `l1ResultsHash` matches that which was stored.
    - The leaf indices of the `l1ResultsTree` which were updated, were the correct leaves to update.
- A user can then call the relevant L2 callback function of a particular 'completed' L1 call.
  - Public/Private circuit ABIs contain an `isCallback` indicator, to enable the kernel/rollup circuits to perform extra checks on the validity of this callback, relative to the data in the `l1ResultsTree`.
  - The callback circuit will expose:
    - its `emittedPublicInputs`
    - The `l1ResultHash` of the L1 call to which this callback is responding. Note, there's no enforcement of _how_ this app-specific circuit makes use of this `l1ResultsHash` or the `emittedPublicInputs`.
- The kernel snark which processes the callback circuit will:
  - Recalculate this callback's `callbackHash`.
  - 'Complete' it with the callback's exposed `l1ResultsHash`.
  - Check this 'finalised' callback leaf exists in the `l1ResultsTree`.
  - Expose the root of the old `l1ResultsTree` that was used in the membership check.
  - Calculate and expose a `callbackNullifier`, to prevent this L1 interaction's callback from being called again.


#### More details

When an L2 circuit makes a call to L1, it exposes the following data to the kernel circuit (amongst other things):
- [`l1CallStackItem`](./transactions.md#l1-callstack-items)
- [`callbackStackItem`](#callbackstackitem)

##### `callback`

A call to a callback looks like any other call, except certain public inputs are omitted from the callStackItem's calculation (and hence also the callbackCallHash's calculation), since stuff like the L1 Result aren't known at the time the callback is added to the L1 Results Tree.
  

> Note: we must enforce (in the kernel circuits) that a callback can only be made to a function of the _same_ contract which made the L1 call in the first place. This is to prevent malicious callbacks which are able to add state to any contract's storage space. (At least, I think this restriction is required...?).


##### `callbackCallHash`

A special type of `callHash`, which omits values which aren't known at the time the callback is added to the L1 Results Tree.


##### `callbackStackItem`

```js
callbackStackItem = {
    callbackPublicKey,
    successCallbackCallHash,
    failureCallbackCallHash,
    successResultArgMapAcc,
}
```
Where:
- `callbackPublicKey` allows a callback to be nullified in a private circuit (by the owner of the `callbackPrivateKey`), so that it isn't revealed _which_ callback has been executed. The `callbackPrivateKey` must be provided to the private kernel circuit in such cases, for the callback's execution to be permitted.
- `successCallbackCallHash` - details of the callback that must be called if the L1 call is successful.
- `failureCallbackCallHash` - details of the callback that must be called if the L1 call fails.
- `successResultArgMapAcc` - a basic accumulator value which details how elements of the L1 Result's array should be mapped to positions of the success callback's arguments. This ensures the L1 Result is fed into the callback's arguments correctly. (See more below).

##### `callbackStackItemHash`

```js
callbackStackItemHash = hash(callbackStackItem)
```



##### L1 Results Tree

The 'Base Rollup Circuit' will need to add details of the L1 call's callbacks to the `l1ResultsTree`. Notice, we add callback data to the results tree _before_ the L1 results are known. Once an L1 result is known, the same leaf of the tree will be updated with the result.


A leaf's value is a hash of data of the form:

`{ isFinalised, l1ResultHash, callbackStackItemHash }`

- Before the L1 result is known:
  - `leafValue = callbackStackItemHash`
- After the L1 result is known:
  - `leafValue = hash(l1ResultHash || 0, callbackStackItemHash)`

> Note: these hashes might need different Pedersen generators to avoid collisions.

> Note: if the L1 call failed or ran out of gas, then `l1ResultHash = 0` will be returned by the RollupProcessor contract. 


```
Example showing 3 stages of leaves of the l1ResultsTree.


                         /         \             /         \             /
                      /   \       /   \       /   \       /   \       /   \
leafIndex            0     1     2     3     4     5     6     7     8     9   ...
callbackPublicKey    -     0x..  -     -     -     0x..  -     -     -     -  
successCallbackHash  0x..  0x..  0x..  0x..  0x..  0x..  0x..  -     -     -
failureCallbackHash  0x..  0x..  0x..  0x..  0x..  0x..  0x..  -     -     -
successResultArgMap  0x..  0x..  0x..  0x..  0x..  0x..  0x..  -     -     -
successResultHash    0x..  0x..  -     -     -     -     -     -     -     -
isFinalised          1     1     1     0     0     0     0     -     -     -
^                    ________________  ______________________  ______________
Data in each leaf    Completed L1      L1 calls which haven't  Empty leaves.
is hashed.           results.          been updated with
                                       a result yet.
```

Notes:
- Notice: there's no failure result hash, because 'failure' includes the possibility that the L1 call ran out of gas or failed in such a way that the RollupProcessor.sol wasn't able to hash anything.
  - This is why we allow `customPublicInputs` to be exposed by the callback function when it's executed. Such data can then be used to revert state changes on L2 if no result was provided by L1.



The RollupProcessor.sol will keep track of (for the `l1ResultsTree`):
- `firstEmptyLeafIndex`
- `firstPendingCallbackLeafIndex` - the left-most leafIndex whose entry is pending (i.e. which hasn't been finalised with an L1 result yet).
- These will be used to ensure the rollup provider is updating the tree in the correct places, within the circuits.




##### `successResultArgMapAcc`

Eventually, we'd like to call L1 functions and await a promise (which will eventually resolve to the L1 result). The syntax might look something like the below, where we intentially use the result in a tricky way:

```js
uint x;

L1Promise promise = await L1_Portal_Contract.my_l1_function(amount_a);

promise.then(
    result => fn1(result[1], x, result[0], result[2]), // on success, the result will 
                                                       // effectively be an array of
                                                       // values.
    fn2(x), // results cannot be passed to the failure
                             // callback.
);

function fn1(uint a, uint b, uint c, uint d) {
  // do stuff
}

function fn2(uint a) {
  // do stuff
}
```

Notice how, upon success of the `promise`, the `result` array (in this example) is passed to `fn1` in a funny order. This is what the `successResultArgMapAcc` is for (until we find a better method). For now, we can create a simple 'accumulator' (although it's not a one-way function) using prime numbers. Later, we can do something cleverer (although anything more than 10 L1 Results and plookup might not be an option, since 11! = 40 million mapping combos). With this 'simple' approach, we can allow up to 10 result values to be returned (since `max := 29^10 * 23^9 * 19^8 * 17^7 * 13^6 * 11^5 * 7^4 * 5^3 * 3^2 * 2^1` doesn't overflow (it's 214-bits), but `31^11 * max` is 269-bits. Actually, initially let's allow 8 L1 Results 

```    
result index:                   0, 1, 2, 3,  4,  5,  6,  7,    
... maps to arg position:       3, 1, 4, -,  -,  -,  -,  -,   (indexed from 1)      
prime for this arg position:    2, 3, 5, 7, 11, 13, 17, 19,


successResultArgMapAcc = 2^3 * 3^1 * 5^4 = 15000 // we can do powers, since the power is
                                                 // always bounded by 8. It's messy
                                                 // though.
```

When the callback is eventually executed, it'll expose its `customPublicInputs`, the `successResultArgMapAcc`, and the `l1ResultHash` to the kernel circuit. In this 'simple' solution, we can also pass binary decompositions of the arg positions (`l1SuccessResultArgPositions`) and the underlying values of the `l1ResultHash` (`l1ResultValues`) as private inputs to the kernel circuit. Using these, we can ensure the results were mapped to the correct argument positions of the callback circuit. 

## Alternative approach to callbacks

The approach to callbacks described in this book (at the moment) is that a callback can only be executed once, and then it's 'nullified' from the L1 Results Tree by the kernel circuit (or rollup circuit). That means an L1 result can only be 'referred-to' once, by a single callback. This differs from Aztec Connect's claim circuit functionality, where the L1 Result (the defi interaction note) can be referred-to many times by the claim circuit; it's up to the claim circuit to ensure it has logic to allow multiple 'nullifications' of the L1 result. We _could_ adopt a similar approach; leaving handling of nullification of an L1 Result to the callback circuit (rather than nullifying in the kernel), but that would require much more care from users to ensure the nullifier is well defined.

**Pros** (of the approach currently described in this book):
- The kernel snarks handle all callback handling logic and L1 Result nullification logic, so it's simpler for users (devs).
  - A 'nullifier' of an L1 Result can always be understood to be a 'one time use' thing.
  - In the 'alternative approach', 'nullification' logic (of L1 Results) is up to the developer.
- The L1 Result callbacks are structured similarly to other languages, e.g. in Rust, a Result has an 'OK' and an 'Err' callback, and the callback can only be called once.

**Cons**
- To achieve defi-bridge logic with this book's approach, there needs to be an extra step: 
  - The L1 Result callback would need to copy the L1 Result (the 'total amounts' from the defi interaction) into the public data tree.
  - Then individual 'claim circuits' could be executed - they'd lookup the 'total amounts' from the public data tree (not from the L1 Result tree, since an L1 result can only be referred-to once, before it gets nullified).
- The 'alternative approach' is perhaps more flexible if a dev knows what they're doing.
- The 'alternative approach' might make kernel and rollup snarks slightly cleaner.