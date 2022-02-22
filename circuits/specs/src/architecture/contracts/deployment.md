# Contract Deployment

A contract is a collection of functions and state variables. Each 'function' is expressed as a circuit, and each circuit can be represented by its verification key. I.e. each verification key represents a callable function in the smart contract.

The set of functions of a contract is represented as a mini Merkle tree of verification keys - a `vkTree` - with the root node - `vkRoot` - being a leaf in the `contractTree`.


**Deployment topics:**
- Constructor functions (to populate initial state variables).
- Specifying an L2 contract address
- Distributing L2 contract data
- Linking to an L1 Portal Contract

These topics are reflected in the layout of the [Contract Deployment ABI](../../architecture/app-circuits/public-input-abis.md#contract-deployment-abi):
```js
publicInputs = {
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

    // Linking to an L1 Portal Contract
    portalContractAddress,
}
```

Note: the distribution of L2 data on-chain is optional and can be done by submitting a compression of the ACIR representation of a circuit as calldata.

## Constructor functions

Constructor functions can be called when deploying a contract, to populate the contract with some initial state variables. A private constructor can be called to populate private states, and a public constructor can be called to populate pulic sates. TODO: we might be able to get away with a single constructor.

The [contract deployment kernel circuit](../../architecture/kernel-circuits/contract-deployment-kernel.md) verifies the constructors' executions.

### Private constructor
A private constructor is only needed if we'd like private states to exist at the beginning of our contract's life. Since private states must be private _to_ a person (i.e. _owned_ by a person), such states would only be created at deployment if we want the _deployer_ to own something privately.

An example might be if someone was creating a completely private cryptocurrency directly on aztec's L2, and they minted the total supply at once for themselves (to distribute themselves). Then initially, the deployer of the contract might be in control of a single 'value note' representing the entire initial supply. They could then distribute value notes thereafter by running a private circuit for 'transferring value' multiple times.
As an extension of this example, the deployer might wish bake-into the private constructor the token distribution logic. Then with the private constructor, they could distribute value immediately and privately to a group of people by creating up-to 64 commitments (actual value TBD).

### Public constructor

Adds initial public state variables to the public data tree.



## L2 Contract Address

The contract address is calculated by the contract deployer, deterministically, as:

- `contractAddress = hash(deployerAddress, salt, vkRoot, constructorHash)`

> The EVM's CREATE2 does `contractAddress = hash(0xff, deployerAddress, salt, keccak(bytecode))` - we've taken this as inspiration.

- `deployerAddress` is included to prevent frontrunning of deployment requests, but it does reveal who is deploying the contract. To remain anonymous a user would have to use a burner address, or deploy a contract _through_ a private contract which can deploy contracts. :question: Why does CREATE2 include a deployerAddress?
- `salt` gives the deployer some 'choice' over the eventual contract address; they can loop through salts until they find an address they like.
- `vkRoot` is like the bytecode without constructors or constructor arguments. This allows people to validate the functions of the contract.
- `constructorHash = hash(privateConstructorPublicInputsHash, publicConstructorPublicInputsHash, privateConstructorVKHash, publicConstructorVKHash)` - this allows people to validate the initial states of the contract. (Note: this is similar to how the `bytecode` in create2 includes an encoding of the constructor arguments).

To prevent duplicate contract addresses existing, a 'nullifier' is submitted when each new contract is deployed. `newContractAddressNullifier = hash(newContractAddress)`. See [here](../kernel-circuits/contract-deployment-kernel.md#execution-logic).

In order to link a `contractAddress`, with with a leafIndex in the `contractTree`, we reserve the storageSlot `0` of each contract's public data tree storage to store that leafIndex.

The `contractAddress` is stored within the contract's leaf of the `contractTree`, so that the private kernel circuit may validate contract address <-> vk relationships.

> Aside: It would have been neat for a contract's address to be the leaf index of its mini Merkle tree (i.e. the root of the `vkTree`) in the`contractTree`. However, the leaf index is only known at the time the rollup provider runs the 'contract deployment kernel snark', whereas we need the contract address to be known earlier, at the time the deployer generates the private constructor proof. Without a contract address, the private constructor would not be able to: call other functions (and pass them a valid callContext); or [silo](./states-and-storage.md#preventing-private-state-collisions) newly created commitments.

## L1 Portal Contract

> (Referring to it as an "L1 Portal" Contract is pleonastic - a portal contract can _only_ be deployed to L1 by its definition, so saying "Portal Contract" is fine).

Each L2 contract will have its own Portal Contract deployed to L1, so that the contract may make [calls to L1](./l1-calls.md). See also the [example](../../examples/erc20/deployment.md#motivation), which gives more reasoning for the need for a Portal Contract.

### Deployment of Portal Contract

There are 3 options for linking an L2 contract with a portal contract, all with pros & cons. TODO: decide which is best. 

1. Deploy the L1 portal contract at the same time as deploying an L2 contract (by providing the L1 portal contract’s bytecode and CREATE2 parameters as public inputs to the Contract Deployment ABI, so that the RollupProcessor can deploy the portal contract).
2. Deploy the L1 portal contract first, tell the Rollup Processor the address, then deploy this L2 contract (providing the L1 address of the portal contract).
3. Deploy this L2 contract, note the deployer’s L1 user address, and allow that same person to later deploy an L1 portal contract and link it with this L2 contract.

Option 1 initially sounds nice and clean, but L1 contract deployment is _expensive_, so just a few portal contract deployments would use all the gas in an L1 block; not leaving enough room for logic to verify the L2 rollup etc. But the most difficult thing about this option would be we'd have to _await_ the success of the Portal Contract deployment attempt on L1 before allowing the L2 contract to be 'finalised', which would need callbacks and painful logic that we'd rather avoid (for contract deployment, at least) if we can.

Option 2 is a nice option, although it doesn't allow for deployment of the Portal Contract to be paid-for from the privacy of aztec's L2 - a user would need public Eth to deploy it. 

Option 3 is possible. The L1 portal contract's address would need to be known in advance, in order to deploy the L2 contract (because the portal contract address is embedded within the leaf of the contract tree). Then we'd need to figure out how to deploy and link the L1 contract.



To prevent two contracts from pointing to the same Portal Contract, a 'nullifier' is submitted when each new contract is deployed, as a way of 'reserving' that Portal Contract's address. `newPortalContractAddressNullifier = hash(newPortalContractAddress)`. See [here](../kernel-circuits/contract-deployment-kernel.md#execution-logic).

The `portalContractAddress` is stored within the contract's leaf of the `contractTree`, so that both the private & public kernel circuits may validate portal contract address <-> vk relationships.




## Distributing L2 contract data

A contract developer might not want to distribute the logic of their contract (e.g. the terms of any business agreement in the real world are always confidential). Therefore, distribution of data on-chain is _not_ enforced. On-chain distribution ensures permanent data availability of the contract's code, so that it can always be executed. But other means of distibution might also be acceptable to users. A developer can optionally submit a representation of their circuits on-chain as calldata for others to validate. A compression of the circuit's ACIR representation might be the most succinct representation of a circuit.

The reason we'd want to distribute a representation of the circuit's logic (rather than just verification keys or something) is because from the circuit logic, everyone can generate the VKs and proving keys of the circuit. The VKs can be used to validate the correctness of the submitted `vkRoot`. Any deployments which fail such validation can be rejected (not used) by users.

## See also...

- _Lots_ more detail is given in the [ERC20 walkthrough](../../examples/erc20/deployment.md).
- [Contract Deployment Public Inputs ABI](../app-circuits/public-input-abis.md#contract-deployment-abi).
- [Contract Deployment Kernel Circuit](../kernel-circuits/contract-deployment-kernel.md).