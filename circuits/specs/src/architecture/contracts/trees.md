# Trees

See the [sexy diagrams](https://drive.google.com/file/d/1gCFhE78QhfEboF0hq3scb4vAU1pE0emH/view?usp=sharing) also (open with -> diagrams.net).

Eventually those diagrams will be included in this book, but it's a bit of faff, so we won't do it until things stabilise. I'd keep the tree diagrams handy in a tab of your browser as you read through this book.

**Private state:**
* `privateDataTree`: stores encrypted UTXO objects (equiv. to current note tree)
* `nullifierTree`: nullifiers for the private data tree

**Public state:**
* `publicDataTree`: each leaf is a public value, which may be updated.Each leaf is a key:value mapping that maps contract address + storage slot -> value. Note this tree's data can only be read/written by the rollup provider, since only the rollup provider can know the very-latest state of the tree when processing a tx.
* `l1ResultsTree`: stores callback function data and the results of L2 --> L1 calls.
* ~~`accountsTree`: stores Aztec 3 user accounts~~ <-- plan is to use Ethereum keypairs instead

**Functions:**
* `contractTree`: each leaf contains an Aztec 3 contract's `vkTree`, the contract's address, and its associated portal contract address[^1].
* `vkTree`: a clear name to give to an aztec contract's mini tree of verification keys (because the term 'contract tree' would always get confused with the `contractTree`)

**Historic tree roots:**
* `privateDataTreeRootsTree`: for membership checks of old roots of the `privateDataTree`
* `contractTreeRootsTree`: for membership checks of old roots of the `contractTree`

**Valid Kernel Snark VKs**
* `privateKernelVKTree`: a tree of VKs for various sizes of private kernel circuit (where 'size' refers to the number of public inputs; namely commitments and nullifiers).
* `publicKernelVKTree`: a tree of VKs for various sizes of public kernel circuit.

**Validium**
We also have Validium versions of some trees (NOT YET SPEC'D)
* `vPrivateDataTree`: Validium version of data tree (data not stored on-chain. i.e. the cheap version for devs that don't want to pay for Ethereum's data availability guarantees)
* `vNullifierTree`: nullifiers for `vPrivateDataTree`
* `vPublicDataTree`: cheapo Validium version of `publicDataTree`


[^1]: Consult the diagram. The `portalContractAddress` is stored within a contract's leaf in the contractTree. This is because both private and public calls should be able to make calls to L1, and so both the private and public kernel snarks need to be able to access the correct `portalContractAddress` for a given contract's address. Similarly, the contract address also needs to be stored somewhere a private circuit can access (so not in the public data tree).