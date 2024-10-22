---
title: Generate a Solidity Verifier
description:
  Learn how to run the verifier as a smart contract on the blockchain. Compile a Solidity verifier
  contract for your Noir program and deploy it on any EVM blockchain acting as a verifier smart
  contract. Read more to find out
keywords:
  [
    solidity verifier,
    smart contract,
    blockchain,
    compiler,
    plonk_vk.sol,
    EVM blockchain,
    verifying Noir programs,
    proving backend,
    Barretenberg,
  ]
sidebar_position: 0
pagination_next: tutorials/noirjs_app
---

Noir has the ability to generate a verifier contract in Solidity, which can be deployed in many EVM-compatible blockchains such as Ethereum.

This allows for a powerful feature set, as one can make use of the conciseness and the privacy provided by Noir in an immutable ledger. Applications can range from simple P2P guessing games, to complex private DeFi interactions.

This guide shows you how to generate a Solidity Verifier and deploy it on the [Remix IDE](https://remix.ethereum.org/). It is assumed that:

- You are comfortable with the Solidity programming language and understand how contracts are deployed on the Ethereum network
- You have Noir installed and you have a Noir program. If you don't, [get started](../getting_started/quick_start.md) with Nargo and the example Hello Noir circuit
- You are comfortable navigating RemixIDE. If you aren't or you need a refresher, you can find some video tutorials [here](https://www.youtube.com/channel/UCjTUPyFEr2xDGN6Cg8nKDaA) that could help you.

## Rundown

Generating a Solidity Verifier contract is actually a one-command process. However, compiling it and deploying it can have some caveats. Here's the rundown of this guide:

1. How to generate a solidity smart contract
2. How to compile the smart contract in the RemixIDE
3. How to deploy it to a testnet

## Step 1 - Generate a contract

This is by far the most straightforward step. Just run:

```sh
nargo compile
```

This will compile your source code into a Noir build artifact to be stored in the `./target` directory, you can then generate the smart contract using the commands:

```sh
# Here we pass the path to the newly generated Noir artifact.
bb write_vk -b ./target/<noir_artifact_name>.json
bb contract
```

replacing `<noir_artifact_name>` with the name of your Noir project. A new `contract` folder would then be generated in your project directory, containing the Solidity
file `contract.sol`. It can be deployed to any EVM blockchain acting as a verifier smart contract.

You can find more information about `bb` and the default Noir proving backend on [this page](../getting_started/quick_start.md#proving-backend).

:::info

It is possible to generate verifier contracts of Noir programs for other smart contract platforms as long as the proving backend supplies an implementation.

Barretenberg, the default proving backend for Nargo, supports generation of verifier contracts, for the time being these are only in Solidity.
:::

## Step 2 - Compiling

We will mostly skip the details of RemixIDE, as the UI can change from version to version. For now, we can just open
<a href="https://remix.ethereum.org" target="_blank">Remix</a> and create a blank workspace.

![Create Workspace](@site/static/img/how-tos/solidity_verifier_1.png)

We will create a new file to contain the contract Nargo generated, and copy-paste its content.

:::warning

You'll likely see a warning advising you to not trust pasted code. While it is an important warning, it is irrelevant in the context of this guide and can be ignored. We will not be deploying anywhere near a mainnet.

:::

To compile our the verifier, we can navigate to the compilation tab:

![Compilation Tab](@site/static/img/how-tos/solidity_verifier_2.png)

Remix should automatically match a suitable compiler version. However, hitting the "Compile" button will most likely generate a "Stack too deep" error:

![Stack too deep](@site/static/img/how-tos/solidity_verifier_3.png)

This is due to the verify function needing to put many variables on the stack, but enabling the optimizer resolves the issue. To do this, let's open the "Advanced Configurations" tab and enable optimization. The default 200 runs will suffice.

:::info

This time we will see a warning about an unused function parameter. This is expected, as the `verify` function doesn't use the `_proof` parameter inside a solidity block, it is loaded from calldata and used in assembly.

:::

![Compilation success](@site/static/img/how-tos/solidity_verifier_4.png)

## Step 3 - Deploying

At this point we should have a compiled contract ready to deploy. If we navigate to the deploy section in Remix, we will see many different environments we can deploy to. The steps to deploy on each environment would be out-of-scope for this guide, so we will just use the default Remix VM.

Looking closely, we will notice that our "Solidity Verifier" is actually three contracts working together:

- An `UltraVerificationKey` library which simply stores the verification key for our circuit.
- An abstract contract `BaseUltraVerifier` containing most of the verifying logic.
- A main `UltraVerifier` contract that inherits from the Base and uses the Key contract.

Remix will take care of the dependencies for us so we can simply deploy the UltraVerifier contract by selecting it and hitting "deploy":

![Deploying UltraVerifier](@site/static/img/how-tos/solidity_verifier_5.png)

A contract will show up in the "Deployed Contracts" section, where we can retrieve the Verification Key Hash. This is particularly useful for double-checking that the deployer contract is the correct one.

:::note

Why "UltraVerifier"?

To be precise, the Noir compiler (`nargo`) doesn't generate the verifier contract directly. It compiles the Noir code into an intermediate language (ACIR), which is then executed by the backend. So it is the backend that returns the verifier smart contract, not Noir.

In this case, the Barretenberg Backend uses the UltraPlonk proving system, hence the "UltraVerifier" name.

:::

## Step 4 - Verifying

To verify a proof using the Solidity verifier contract, we call the `verify` function in this extended contract:

```solidity
function verify(bytes calldata _proof, bytes32[] calldata _publicInputs) external view returns (bool)
```

When using the default example in the [Hello Noir](../getting_started/quick_start.md) guide, the easiest way to confirm that the verifier contract is doing its job is by calling the `verify` function via remix with the required parameters. Note that the public inputs must be passed in separately to the rest of the proof so we must split the proof as returned from `bb`.

First generate a proof with `bb` at the location `./proof` using the steps in [get started](../getting_started/quick_start.md), this proof is in a binary format but we want to convert it into a hex string to pass into Remix, this can be done with the

```bash
# This value must be changed to match the number of public inputs (including return values!) in your program.
NUM_PUBLIC_INPUTS=1
PUBLIC_INPUT_BYTES=32*NUM_PUBLIC_INPUTS
HEX_PUBLIC_INPUTS=$(head -c $PUBLIC_INPUT_BYTES ./proof | od -An -v -t x1 | tr -d $' \n')
HEX_PROOF=$(tail -c +$(($PUBLIC_INPUT_BYTES + 1)) ./proof | od -An -v -t x1 | tr -d $' \n')

echo "Public inputs:"
echo $HEX_PUBLIC_INPUTS

echo "Proof:"
echo "0x$HEX_PROOF"
```

Remix expects that the public inputs will be split into an array of `bytes32` values so `HEX_PUBLIC_INPUTS` needs to be split up into 32 byte chunks which are prefixed with `0x` accordingly.

A programmatic example of how the `verify` function is called can be seen in the example zk voting application [here](https://github.com/noir-lang/noir-examples/blob/33e598c257e2402ea3a6b68dd4c5ad492bce1b0a/foundry-voting/src/zkVote.sol#L35):

```solidity
function castVote(bytes calldata proof, uint proposalId, uint vote, bytes32 nullifierHash) public returns (bool) {
  // ...
  bytes32[] memory publicInputs = new bytes32[](4);
  publicInputs[0] = merkleRoot;
  publicInputs[1] = bytes32(proposalId);
  publicInputs[2] = bytes32(vote);
  publicInputs[3] = nullifierHash;
  require(verifier.verify(proof, publicInputs), "Invalid proof");
```

:::info[Return Values]

A circuit doesn't have the concept of a return value. Return values are just syntactic sugar in Noir.

Under the hood, the return value is passed as an input to the circuit and is checked at the end of the circuit program.

For example, if you have Noir program like this:

```rust
fn main(
    // Public inputs
    pubkey_x: pub Field,
    pubkey_y: pub Field,
    // Private inputs
    priv_key: Field,
) -> pub Field
```

the `verify` function will expect the public inputs array (second function parameter) to be of length 3, the two inputs and the return value.

Passing only two inputs will result in an error such as `PUBLIC_INPUT_COUNT_INVALID(3, 2)`.

In this case, the inputs parameter to `verify` would be an array ordered as `[pubkey_x, pubkey_y, return`.

:::

:::tip[Structs]

You can pass structs to the verifier contract. They will be flattened so that the array of inputs is 1-dimensional array.

For example, consider the following program:

```rust
struct Type1 {
  val1: Field,
  val2: Field,
}

struct Nested {
  t1: Type1,
  is_true: bool,
}

fn main(x: pub Field, nested: pub Nested, y: pub Field) {
  //...
}
```

The order of these inputs would be flattened to: `[x, nested.t1.val1, nested.t1.val2, nested.is_true, y]`

:::

The other function you can call is our entrypoint `verify` function, as defined above.

:::tip

It's worth noticing that the `verify` function is actually a `view` function. A `view` function does not alter the blockchain state, so it doesn't need to be distributed (i.e. it will run only on the executing node), and therefore doesn't cost any gas.

This can be particularly useful in some situations. If Alice generated a proof and wants Bob to verify its correctness, Bob doesn't need to run Nargo, NoirJS, or any Noir specific infrastructure. He can simply make a call to the blockchain with the proof and verify it is correct without paying any gas.

It would be incorrect to say that a Noir proof verification costs any gas at all. However, most of the time the result of `verify` is used to modify state (for example, to update a balance, a game state, etc). In that case the whole network needs to execute it, which does incur gas costs (calldata and execution, but not storage).

:::

## A Note on EVM chains

Noir proof verification requires the ecMul, ecAdd and ecPairing precompiles. Not all EVM chains support EC Pairings, notably some of the ZK-EVMs. This means that you won't be able to use the verifier contract in all of them. You can find an incomplete list of which EVM chains support these precompiles [here](https://www.evmdiff.com/features?feature=precompiles).

For example, chains like `zkSync ERA` and `Polygon zkEVM` do not currently support these precompiles, so proof verification via Solidity verifier contracts won't work. Here's a quick list of EVM chains that have been tested and are known to work:

- Optimism
- Arbitrum
- Polygon PoS
- Scroll
- Celo
- BSC
- Blast L2
- Avalanche C-Chain
- Mode
- Linea
- Moonbeam

If you test any other chains, please open a PR on this page to update the list. See [this doc](https://github.com/noir-lang/noir-starter/tree/main/with-foundry#testing-on-chain) for more info about testing verifier contracts on different EVM chains.

## What's next

Now that you know how to call a Noir Solidity Verifier on a smart contract using Remix, you should be comfortable with using it with some programmatic frameworks, such as [hardhat](https://github.com/noir-lang/noir-starter/tree/main/vite-hardhat) and [foundry](https://github.com/noir-lang/noir-starter/tree/main/with-foundry).

You can find other tools, examples, boilerplates and libraries in the [awesome-noir](https://github.com/noir-lang/awesome-noir) repository.

You should also be ready to write and deploy your first NoirJS app and start generating proofs on websites, phones, and NodeJS environments! Head on to the [NoirJS tutorial](../tutorials/noirjs_app.md) to learn how to do that.
