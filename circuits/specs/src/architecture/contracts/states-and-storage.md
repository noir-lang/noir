# States & Storage

# States

There are 3 kinds of state to be aware of in Aztec 3, and these reflect the [function types](./function-types.md) of Aztec 3:

- Private States (L2)
- Public States (L2)
- L1 states (public by definition)


## Private States 

Private states are state variables which are private to a single user (or a group of users if we do multisig stuff). Each state is represented by a commitment (or a set of UTXO commitments) in the `privateDataTree`. A state can be edited by producing and submitting a nullifier for one (or multiple) commitments to the nullifier tree.

### Commitment preimage layout

This is just a suggestion for a general commitment preimage layout.

Note: app developers will technically have 'free reign' over the preimage layouts of their commitments (and similarly of their nullifiers). In practice, though, if they use Noir to produce their Aztec 3 circuits, the layouts will likely follow a prescribed format.

Suggestion:

A private state with key:value pair `storageSlot:value` could have layout:

`commitment = h(storageSlot, value, owner, creator, salt, inputNullifier)`

where:
- `storageSlot` - a unique slot for this particular private state, determined (most likely) by Noir. Assignment could be similar to the [storage slots in Solidity](https://docs.soliditylang.org/en/v0.8.9/internals/layout_in_storage.html), which are assigned based on the order variables are declared (with a few caveats for complex types).
- `value` - the secret value being stored.
- `owner` - A public key. Every secret has a secret holder, or 'owner' (otherwise it wouldn't be secret, or it would be known to no-one (which would be useless)). Note, this might not be needed for all apps. :question: Perhaps for some apps, knowledge of the commitment's `salt` will be enough to allow the state to be nullified (so no owner is required). 
- `creator` - the public key of the user who wrote this secret state to the `publicDataTree` (might not be needed for all apps!)
- `salt` - Every commitment needs some randomness to hide the contents.
- `inputNullifier` - This ensures uniqueness of the commitment. Perhaps this is an app-specific requirement that isn't needed in general; maybe other apps could ensure uniqueness in some other way? Not sure.

Note: in some cases, the `owner` and `value` might be the same, e.g. for NFTs (where NFT states are represented by `ownerOf[tokenId]`).

> More on `storageSlot`: Whereas the only 'private state variable' which exists for an "Aztec 2.0 join-split tx" is a user's balance (represented as a set of UTXO commitments), more generally a private contract might contain many private state variables, and a private circuit might read/write several of them.

> For mappings and arrays of secret variables (if in-scope), we might wish to use a more snark-friendly method for deriving storageSlots, since Solidity uses keccak hashing.

> A consistent commitment layout might be produced by Noir. Note, though, that apps don't necessarily _have_ to follow this layout; a app's circuit can do whatever it wants, as long as it conforms to the circuit Public Inputs ABI.


### Nullifier preimage layout

Perhaps a good layout for a nullifier would be:

`nullifier = h(commitment, user_private_key)`

> A consistent nullifier layout might be produced by Noir. Note, though, that apps don't necessarily _have_ to follow this layout; a app's circuit can do whatever it wants, as long as it conforms to the circuit Public Inputs ABI.

> Note: In Aztec 2 we also had an `is_real` boolean inside the nullifier, which was needed because every circuit _needed_ to produce two nullifiers, even if some were 'fake'. In Aztec 3, we aren't restricted in that way. A circuit can create any number of nullifiers (even none). Fake nullifiers can be added to a private kernel snark's output (in order to mask the function that was executed) by, for example, calling a special 'padding' circuit of another contract which produces dummy nullifiers in such a way that doesn't require an `is_real` component.

> Note: In Aztec 2, the Account Circuit can produce a nullifier which effectively nullifies _an entire set_ of account notes. Note however, that account notes and the functionality of an account circuit in Aztec 3 can be created using a _public circuit_ (see [below](#examples)). Account notes aren't really even private states.

### Preventing private state collisions 

Within a contract, private state collisions are prevented by including a `storageSlot` in the commitment's preimage. To prevent private state collisions between different contracts, we can do the following.

Let's call the each of the commitments produced by a private circuit an `innerCommitment`. The private kernel circuit will 'attach' to the `innerCommitment` the contract address of the private contract to which the private circuit belongs, to create an `outerCommitment`.

E.g. `outerCommitment = h(contractAddress, innerCommitment)`

### Preventing nullifier collisions


Similarly, we must do the same with nullifiers:

E.g. `outerNullifier = h(contractAddress, innerNullifier)`.

Otherwise, a malicious app could front-run valid nullifier submissions of other contracts.

## Public States

Public states are stored in a giant 254-heigh merkle tree; the `publicDataTree`. Each leaf of the tree is a key:value mapping, where:

- The key is `hash(contractAddress, storageSlot)`
- The value is the public state's `value`.

> Note: including the `contractAddress` prevents collisions _between_ contracts. The `contractAddress` is injected by the Public Kernel Circuit (not by the app!), to prevent an app from maliciously writing to another contract's state.

### Reserved storage slots

For each contract, the following storage slots (in the public data tree) are reserved (and understood by the kernel snarks):

- `0`: value is the leafIndex of the contractTree where this contract's vkTree is located.
    - :question: There's a suggestion to remove the vk tree, and place functions directly as leaves of the contracts tree (which would be renamed to "function tree"). Functions could then be updated by nullifying the old one and deploying a new function to the next empty leaf in the function tree. The `0` storage slot would then be the length of a dynamic array, whose ith element sits at hash(0)+i (much like Solidity storage layouts). However, with such a suggestion, each time a contract calls a function, a 'read emulation' would be needed, where the function is nullified and re-added to the tree for every call, which is untidy.
    - The suggestion also adds complexity, like: how do we manage function updates and protect users from having functions change without them realising?




## Examples

For those familiar with Aztec Connect, we can categorise its various state variables into the Aztec 3 framework. Although in Aztec 2, we stored most notes in the 'data tree' (an analogue of this spec's `privateDataTree`), notice that not all of those notes will belong in the `privateDataTree` in Aztec 3.

> See the related [examples](./function-types.md#examples) in the 'function types' section for Aztec3-categorisation of Aztec2's circuits.

> See also the detailed walkthrough example of deploying an ERC20 shielding protocol (effectively zk-money) to Aztec 3 [here](../../examples/erc20/erc20-shielding.md).

**Value Note**  
A value note forms _part_ of a **private** state. A user's erc20 balances is _partitioned_ across a set of value notes. The total balances is the sum of all not-yet-nullified value notes owned by the user.

**Account Note**  
A **public** state. The contents of this note (`account_alias_id, account_public_key, spending_public_key`) are all public values, so don't necessarily need to be private. But more importantly, in Aztec 2, the 'nullifier' doesn't nullify an account note with 'zero-knowledge' - that is to say, the nullifier is just a hash of the `account_alias_id`, which anyone can dictionary attack to learn who's account was just migrated. In Aztec 3, a user's 'current' account keys, nonce, and alias, could all be stored in the `publicDataTree` and overwritten whenever the user wants to migrate; there's no need for commitments & nullifiers (unless wanting to edit state under zero knowledge).

**Claim Note**  
A **public** state. A claim note contains all public values, so it could be stored as a struct (e.g.) in the `publicDataTree`. The 'nullification' of a claim note that happens in Aztec 2 doesn't hide which claim note has been nullified; notes and nullifiers are just used because there's no public state tree that can be edited.

**Defi Interaction Note**  
A **public** state. Similar to a claim note, the contents of a defi interaction note are all public values, so it could be stored as a struct (e.g.) in the `publicDataTree`. The 'nullification' of a defi interaction note that happens in Aztec 2 doesn't hide which note has been nullified; notes and nullifiers are just used because there's no public state tree that can be edited.

> So if Aztec 2 were migrated to Aztec 3, the only private state would be a user's balance; partitioned across a series of value notes.