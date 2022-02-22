# Deployment

Let's suppose our aztec contract will have 3 circuits (plus constructor circuits): deposit, transfer, withdraw.

> In reality these can be a single circuit, much like our join-split circuit, but we'll split it into 3 to show off more machinery of aztec 3.0. Suppose the proving keys and verification keys of these circuits have been generated and we're ready to deploy our aztec contract to Aztec 3.0.

## Private Constructor

See [here](../../architecture/contracts/deployment.md#private-constructor) for background.

In this example, we don't need to initialise any private states, so won't be using the private constructor.

## Public constructor

Let's have a think about Aztec 2.0 and what public states are initialised and tracked for the 'zk.money ERC20' functionality:
- `userPendingDeposits[] (mapping)`
- `depositProofApprovals[] (mapping)`
- `supportedAssets[]`
- `DEFAULT_ERC20_GAS_LIMIT = 75000;`
- `assetGasLimit[] (mapping)`

Since Aztec 3.0 will be agnostic to applications, we won't be able to store these states in the RollupProcessor contract anymore. So these app-specific states will need to either be stored in the aztec contract's portal contract, or in the L2 contract (in the publicDataTree). Both options are possible.

The `userPendingDeposits, depositProofApprovals` will both start out as empty mappings, so don't need to be initialised in the public constructor. We'll probably also store these on-chain in the L1 portal contract - another reason we won't be initialising them in this constructor.

The `supportedAssets, DEFAULT_ERC20_GAS_LIMIT, assetGasLimit` array _could_ be prepopulated with some values in the constructor, if we wish for these values to be stored in the aztec L2 publicDataTree, instead of in the L1 portal contract.

To show off as much machinery as possible, let's store `supportedAssets` in the publicDataTree and initialise it in the constructor. The other two gas-related values I'll ignore for now, since 'gas limit' stuff still needs some thinking.

:question: TODO: understanding gas limits for portal contract interactions is a complicated thing to spend time thinking about. The rollup provider really needs to know these limits before they process the rollup, in order to gauge their expected profits.

This example public constructor will simply be a Public Circuit (which follows the Public Circuit ABI) which sets the value of `supportedAssets`. Before we can set this value, we need to know its `storageSlot` in our new contract.

[Storage slots in Solidity](https://docs.soliditylang.org/en/v0.8.9/internals/layout_in_storage.html) are assigned based on the order variables are declared (with a few caveats for complex types). Aztec 3 is a bit more complicated, because storage is split across 3 places:
- On chain storage in the Portal Contract
- Public L2 storage in the publicDataTree (account-based storage model).
- Private L2 storage in the privateDataTree (UTXO storage model).

In this example, things will be stored as follows:

- Portal contract:
    - `userPendingDeposits[] (mapping)`
    - `depositProofApprovals[] (mapping)`
    - ~~`DEFAULT_ERC20_GAS_LIMIT = 75000;`~~ For simplicity, we'll ignore this here.
    - ~~`assetGasLimit[]`~~ - Fees & gas limits are tricky. I'll ignore for now.
- publicDataTree
    - `supportedAssets[]` - publicDataTree (for example's sake)
- privateDataTree
    - Join-split value notes are a representation of a user's `balance` (in ERC20 terms).
    - The ERC20 notion of a `balances[]` mapping can be converted into a UTXO-friendly notion for the privateDataTree. Each user's balance is 'partitioned' across set of commitments. The sum of a user's (not-yet-nullified) commitments' values is understood to represent `balances[userAddress]`. 


We're only considering writes to the publicDataTree in the public constructor. So what storage slots will be used in the tree?

The 0th storage slot of every contract are reserved (in the current spec) for:
0. The leafIndex of a contract's vkRoot in the contractTree.

So, `supportedAssets` can start at `storageSlot = 1`. It's a dynamic array, so we can perhaps copy Ethereum's method for deriving storage slots:

- The length of the array can be stored at `storageSlot = 1`.
- The `i`th index of the array can be stored at `storageSlot = hash(1) + i`.

Suppose we wish to support 3 assets at the beginning: ETH, DAI, WBTC with asset_id values `0, 1, 2` resp.

Then there are **two** `stateTransitions` for the publicDataTree which the public constructor circuit will expose (via the ABI):
- `[storgeSlot, oldValue, newValue]`:
    - `[hash(1) + 1, 0, 1]` for (`supportedAssets[1] = 1;`)
    - `[hash(1) + 2, 0, 2]` for (`supportedAssets[2] = 2;`)

Notice: we don't need to expose a state transition for ETH's asset_id, because it's intended value is `0`.

Here's the public constructor's nonzero public inputs (refer to the Public Circuit ABI for the rest):

```js
customPublicInputs,
customPublicOutputs,
stateTransitions: [
    [hash(1) + 1, 0, 1],
    [hash(1) + 2, 0, 2],
],
```

(The other public inputs aren't shown, because they're either 0, or global stuff like block timestamps).

> Note: although we can specify a recommended storage layout (which Noir will use), there's nothing technically stopping an app from deciding upon its own storage layout rules. Contracts' storage slots are siloed by hashing with the contract address (see the spec).


The `publicConstructorCallstackItem` looks like:

```js
publicConstructorCallstackItem: {
  functionSignature: concat(
      contractAddress,
      vkIndex = 0, // Constructor vks won't be added to the contract's vkTree.
                   // This `0` is treated as 'null' when `isConstructor = true`.
      isPrivate = false,
      isConstructor = true,
      isCallback = false,
  ),
  publicInputsHash, // hash of public constructor’s public inputs (see earlier section)
  callContext,
  isDelegateCall,
  isStaticCall,
}
```

## L2 contract address

Much like Ethereum's create2, L2 contract addresses are specified deterministically:

`newContractAddress := hash(deployerAddress, salt, vkRoot)`

`deployerAddress` can be a user or another contract.



## L1 Portal Contract

Refer to the [appendix](./appendix/portal-contract.md) for example pseudocode.
### Motivation

We need a way of depositing erc20 tokens into a shield contract (an L1 contract which holds the erc20 tokens in escrow whilst they're transferred privately via L2). With Aztec 1.0 & 2.0, the RollupProcessor contract served as the escrow contract, since there was only one application (zk-money). But for Aztec 3.0, we need to _silo_ the funds held in escrow on behalf of each L2 aztec contract. Why?

Imagine two similar erc20-shielding aztec contracts get deployed (by different developers) to Aztec 3.0. The contracts are both composed of deposit, transfer and withdraw circuits. Suppose one of the developers is malicious, and put a backdoor into their withdraw circuit to allow them to withdraw 'infinite' L1 tokens that are held in escrow. Then if all of the L1 funds of both L2 contracts were held in a single pool, the malicious dev could withdraw them all. So we definitely need L1 funds to be siloed for each L2 contract.

Why don't we track the siloed funds of the different L2 contracts in the RollupProcessor, still? Because Aztec 3.0 won't just support erc20 shielding. It'll support unlimited use cases, where different L2 contracts will want to manage different L1 states (not necessarily token balances). Therefore we'll put the onus of developing the logic of managing L1 states in the hands of the aztec contract developers. We'll allow devs to deploy an L1 'portal contract' which interacts with other L1 Ethereum contracts and manages L1 state for a particular L2 aztec contract. "Portal" because it links L2 & L1.

Note: portal contracts will be able to make calls to _any_ L2 function. But an L2 function will only be able to make calls to its portal contract; an L2 function cannot call other L1 contracts directly, because it doesn't have an L1 address. That's a nice way of thinking about Portal contracts actually - they provide an L1 address through which calls to other L1 contracts can be made.

### Portal Contract Deployment

See [here](../../architecture/contracts/deployment.md#l1-portal-contract-address) first.

Let's assume option 2 is available to us for this example. The deployer (the dev who's deploying this L2 contract) writes an L1 Portal Contract to handle calls to/from L2. The developer compiles this portal contract (along with constructor arguments) into bytecode. The address of the portal contract can be determined in advance using CREATE2.

```js
portalContractAddress = keccak256(0xff, deployingAddress, salt, keccak256(bytecode))[12:]
```

> Notice there's a constructor argument in the [portal contract example](./appendix/portal-contract.md), just to show off as much machinery as possible. We didn't really need this, because the Rollup Processor's address can be accessed via `msg.sender`. 'Bytecode' is understood to include the contructor arguments of a function. I.e. let bytecode be `abi.encodePacked(bytecode, abi.encode(args))`.

_Somehow_ (TODO) the deployer deploys the Portal Contract, and maybe registers it with the RollupProcessor. We need to prevent front-running of the Portal Contract address, whereby an attacker links the wrong L2 contract with the intended Portal Contract (TODO).


## Putting it all together and deploying the contract

The dev will:
- Write circuits in Noir
- Write Portal Contract
- Write loads of code (TypeScript probably) to build their app.
- Send a tx to deploy a contract to the rollup provider.
    - ```js
      call = {
          publicInputs: {
              // Constructor functions
              privateConstructorPublicInputsHash,
              publicConstructorPublicInputsHash,
              privateConstructorVKHash,
              publicConstructorVKHash,

              // L2 contract address (create2-like)
              contractAddress,
              vkRoot,

              // Distributing L2 contract data
              circuitDataKeccakHash,

              // Deploying an L1 Portal Contract
              portalContractAddress,
          },
          callContext, // not needed?
      }
      ```
      (the preimages of all these hashes will also need to be sent, so they may be fed in as private inputs to the contract deployment kernel snark).
- If there's a private constructor, the dev will need to execute that constructor circuit in their client and 'wrap' it in a private kernel snark proof. Then send that kernel proof to the rollup provider too (probably in the same send as above).
- The rollup provider will need to execute any public calls and public kernel snark recursions if that private call made any public calls.
- If there's a public constructor, the rollup provider will need to execute that and 'wrap' it in a public kernel snark.
- The rollup provider will then execute the Contract Deployment Kernel Circuit and generate a proof.
    - This circuit validates both constructor kernel snarks;
    - Then adds the contract data to the tree.
- Then that proof will be fed into the Base Rollup Circuit...
- ... which will then be fed into the Merge Rollup Circuit...
- ... which will eventually be sent to the RollupProcessor contract.
- The RollupProcessor might store a mapping between the Portal Contract's address and the Contract's address.





