---
title: Noir
---

## What is Noir

Zero-knowledge proofs (ZKP) have been gaining tremendous traction in the blockchain space, came up first as a privacy solution and increasingly as a scaling solution.

Noir is a Domain Specific Language for developing ZK-provable programs. The execution of Noir programs can be proved and verified without revealing all information involved in the process (e.g. program inputs), offering privacy benefits. It can also be verified with asymmetrically-less computation power on the verifier's end, offering scaling benefits.

Noir is designed for accessible and flexible development of provable programs. It abstracts away the need for developers to handcraft low-level circuits and aims at supporting different proving backends for different development needs.

> **Circuits:** Source code of ZK programs are sometimes referred to as circuits due to the underlying design and working principles of ZKP systems. For development purposes, you may consider the two terms interchangeable.

## Who is Noir for?

### Decentralized Application Developers

Noir offers one-click generation of a verifier smart contract for every Noir program, and hence could be a great choice for developing dApps that benefit from the use of ZKP.

### Protocol / Blockchain Developers

Noir is designed to be compilable into different arithmetic schemes of choice, hence can support different proving backends. It could be a great choice for infrastructural development that might be interested in switching the default PLONK-based proving system out due to specific limitations or requirements on the proving backend to be adopted.

## Workshop Video

Certain content of this guide is also covered in this workshop video:

[![](https://i.imgur.com/9CSN9Jo.jpeg)](https://www.youtube.com/watch?v=I5M8LhOECpM&t=2879s)

The code demonstrated in the video is available at:
https://github.com/vezenovm/basic_mul_noir_example

## Install Noir

### Prerequisites

- Install [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
- Install [Rust](https://www.rust-lang.org/tools/install)
- [Noir VS Code extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.noir-programming-language-syntax-highlighter) for syntax highlighting (optional)
- Install [Node.js](https://nodejs.org/en/download/) for TypeScript testing (optional)
- Install [Yarn](https://classic.yarnpkg.com/lang/en/docs/install/) for TypeScript testing (optional)

### Install Noir from Source

1. Clone the [Noir repository](https://github.com/noir-lang/noir):

```shell
git clone https://github.com/noir-lang/noir.git
```

2. Change directory into `nargo`:

```shell
cd noir/crates/nargo
```

3. Compile the binary and store it in your path:

```shell
cargo install --locked --path=.
```

If the compilation fails, go into `nargo/Cargo.toml` and replace `aztec_backend = ...` with the following:

```
aztec_backend = { optional = true, git = "https://github.com/noir-lang/aztec_backend", rev = "d91c69f2137777cec37f692f98d075ae10e7a584", default-features = false, features = [
    "wasm-base",
] }
```

> **Note:** This `aztec_backend` dependency utilizes the C++ backend's wasm executable instead of compiling from source.
>
> Noir compiles down to an intermediate representation, which can then compile down to any proof system that is compatible with this intermediate representation. Aztec's barretenberg proof system is supported, which is the backend dependency seen here.

4. Check if the installation was successful:

```shell
nargo help
```

For more information on Noir installation, check the [installation section](https://noir-lang.github.io/book/getting_started/install.html) of the Noir book.

## Noir Workflow

### Create a Noir Project

Navigate to a desired directory, and create a new Noir project with your preferred name:

```shell
nargo new {PROJECT_NAME}
```

> **Note:** Common practice would be to name it `circuits` for the directory to sit along other folders in your codebase.

Similar to Rust, you should now have `src/main.nr`, which contains the source code of your main program, and `Nargo.toml`, which contains your environmental options.

## Understanding Noir

The default `main.nr` generated should look like this:

```rust
fn main(x : Field, y : pub Field) {
	constrain x != y;
}
```

The first line of the program specifies its inputs:

```rust
x : Field, y : pub Field
```

Program inputs in Noir are private by default (e.g. `x`), but can be labeled public using the keyword `pub` (e.g. `y`).

> **Note:** Private inputs are known only to the prover, while public inputs are shared along the proof with the verifier. Most projects intend to implement the verifier as a public smart contract, hence public inputs are often considered to eventually be public knowledge.

The next line of the program specifies its body:

```rust
constrain x != y;
```

The keyword `constrain` can be interpreted as something similar to `assert` in other languages.

> **Note:** In the context of Noir, `constrain` ensures the satisfaction of the condition (e.g. `x != y`) is constrained by the proof generated from proving the execution of said program (i.e. if the condition was not met, the verifier would reject the proof as an invalid proof).

For more information on Noir's syntax, check the [language sections](https://noir-lang.github.io/book/language_concepts.html) of the Noir book.

You may refer to the [_Standard Noir Example_](https://github.com/vezenovm/basic_mul_noir_example) and [_Mastermind in Noir_](https://github.com/vezenovm/mastermind-noir) on GitHub for more inspiration on writing Noir programs as well.

### Generate Input Files

Change directory into your project folder and initiate building of input files:

```shell
cd {PROJECT_NAME}
nargo build
```

Two additional files would be generated in your project directory.

`Prover.toml` is used to specify the input values (and expected output values, if applicable) for executing and proving the program.

`Verifier.toml` is used to specify the input values for verifying the execution proof.

### Prove a Noir Program

Fill in the values in `Prover.toml`. For example:

```
x = "3"
y = "4"
```

Prove the valid execution of your Noir program with your preferred proof name:

```shell
nargo prove {PROOF_NAME}
```

A new folder `proofs` would then be generated in your project directory, containing the proof file `{PROOF_NAME}.proof`.

### Verify a Noir Program

Fill in the values in `Verifier.toml`. For example:

```
y = "4"
setpub = []
```

> **Note:** Aside from verifying the program's valid execution, the verifier of a Noir program also verifies the validity of public inputs (i.e. are the public inputs in the provided proof identical to what is expected on the verifier's ends). In this case, we expect `y = 4` and hence we specify so in `Verifier.toml`.

Verify your proof generated:

```shell
nargo verify {PROOF_NAME}
```

If the verification is successful, you should be prompted with `true` in the terminal:

```shell
$ nargo verify testProof
true
```

> **Note:** In production, the prover and the verifier are usually two separate entities. A prover would retrieve the necessary inputs, execute the Noir program, generate a proof and pass it to the verifier. The verifier would then retrieve the public inputs from usually external sources and verifies the validity of the proof against it.
>
> Take a private asset transfer for example. A user on browser as the prover would retrieve private inputs (e.g. the user's private key) and public inputs (e.g. the user's encrypted balance on-chain), compute the transfer, generate a proof and submit it to the verifier smart contract. The verifier contract would then draw the user's encrypted balance directly from the blockchain and verify the proof submitted against it. If the verification passes, additional functions in the verifier contract could trigger (e.g. approve the asset transfer).

## Advanced Techniques

### Generate a Verifier Contract

For certain applications, it may be desirable to run the verifier as a smart contract instead of on a local machine.

Generate a verifier Solidity contract for your Noir program:

```shell
nargo contract
```

### Working with Noir Programs in TypeScript

Noir programs can also be compiled, proved and verified in TypeScript, which could be useful for writing automated test scripts (e.g. [Hardhat](https://hardhat.org/) tests).

The following sections are based mainly on the [_Standard Noir Example_](https://github.com/vezenovm/basic_mul_noir_example), specifically on its test script [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts) as a demonstration of a typical TypeScript Noir testing workflow. The workshop video linked at the beginning of this guide is a great walkthrough of the code base.

### Prerequisites

Certain dependencies shall be installed for working with Noir programs using TypeScript:

```shell
yarn add @noir-lang/noir_wasm @noir-lang/barretenberg @noir-lang/aztec_backend
```

### Compiling

To begin testing a Noir program, it first needs to be compiled by calling `noir_wasm`'s `compile` function.

For example in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L32):

```javascript
const compiled_program = compile(
  path.resolve(__dirname, "../circuits/src/main.nr")
);
```

The `compiled_program` returned by the function contains the Abstract Circuit Intermediate Representation (ACIR) and the Application Binary Interface (ABI) of your Noir program. They shall be stored for proving your program later:

```javascript
let acir = compiled_program.circuit;
const abi = compiled_program.abi;
```

### Specifying Inputs

Having obtained the compiled program, the program inputs shall then be specified in its ABI.

_Standard Noir Example_ is a program that multiplies input `x` with input `y` and returns the result:

```noir
fn main(x: u32, y: pub u32) -> pub u32 {
    let z = x * y;
    z
}
```

Hence, one valid scenario for testing could be `x = 3`, `y = 4` and `return = 12` like in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L37):

```javascript
abi.x = 3;
abi.y = 4;
abi.return = 12;
```

> **Note:** Return values are also required to be specified, as they are merely syntax sugar of inputs with equality constraints.

### Initializing Prover & Verifier

Prior to proving and verifying, the prover and verifier have to first be initialized by calling `barretenberg`'s `setup_generic_prover_and_verifier` with your Noir program's ACIR.

For example in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L41):

```javascript
let [prover, verifier] = await setup_generic_prover_and_verifier(acir);
```

### Proving

The execution of the Noir program can then be proved by calling `barretenberg`'s `create_proof` function.

For example in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L43):

```javascript
const proof = await create_proof(prover, acir, abi);
```

### Verifying

The `proof` obtained can then be verified by calling `barretenberg`'s `verify_proof` function.

For example in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L45):

```javascript
const verified = await verify_proof(verifier, proof);
```

If the entire process is working as intended to be, `verify_proof` should return `true`. The test case can then be concluded with an assertion:

```javascript
expect(verified).eq(true);
```

Alternatively, the `proof` can be verified using a verifier smart contract as well.

For example in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L97):

```javascript
let Verifier: ContractFactory;
let verifierContract: Contract;

before(async () => {
    Verifier = await ethers.getContractFactory("TurboVerifier");
    verifierContract = await Verifier.deploy();
});

...

const sc_verified = await verifierContract.verify(proof);
expect(sc_verified).eq(true)
```

For additional inspiration on writing tests for Noir programs, check the full test scripts [`1_mul.ts` of _Standard Noir Example_](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts) and [`mm.ts` of _Mastermind in Noir_](https://github.com/vezenovm/mastermind-noir/blob/master/test/mm.ts) on GitHub respectively.

## Alternative Way of Compiling with `nargo`

Similar to how `nargo` can be used to prove and verify Noir programs, a Noir program can also be compiled manually in the CLI.

> **Note:** This approach is not recommended for usage outside debugging given the complexity to automate it for e.g. testing purposes.

Fill in the values in `Prover.toml`. For example:

```
x = "3"
y = "4"
return = "12"
```

Then compile your Noir program with your preferred build name:

```shell
nargo compile {BUILD_NAME}
```

A new folder `build` should then be generated in your project directory, containing `{BUILD_NAME}.acir` and `{BUILD_NAME}.tr`.

> **Note:** The `.acir` file is the ACIR of your Noir program, and the `.tr` file is the witness file. The witness file can be considered as program inputs parsed for your program's ACIR.
>
> If your program is designed for privacy, the prover should refrain from sharing the witness file with others and should delete it once the proof is generated to best protect the private inputs from public knowledge.

The compiled files can then be parsed for proving and verifying in TypeScript as well. For example in [`1_mul.ts`](https://github.com/vezenovm/basic_mul_noir_example/blob/master/test/1_mul.ts#L13):

```javascript
let acirByteArray = path_to_uint8array(path.resolve(__dirname, '../circuits/build/p.acir'));
let acir = acir_from_bytes(acirByteArray);

let witnessByteArray = path_to_uint8array(path.resolve(__dirname, '../circuits/build/p.tr'));
const barretenberg_witness_arr = await packed_witness_to_witness(acir, witnessByteArray);
...
const proof = await create_proof_with_witness(prover, barretenberg_witness_arr);
```

## Resources

### [📓 The Noir Book](https://noir-lang.github.io/book/)

The go-to guide of everything Noir.

### [📝 Noir Repo](https://github.com/noir-lang/noir)

The main repository of Noir development.

### [📝 Standard Noir Example](https://github.com/vezenovm/basic_mul_noir_example)

A basic example demonstrating Noir workflows.

### [📝 Mastermind in Noir](https://github.com/vezenovm/mastermind-noir)

Mastermind the game written in Noir.

### [📝 Private Proof of Membership in Noir](https://github.com/vezenovm/simple_shield)

The foundation to private token mixer, voting, airdrop, identity and more.

### [👾 Discord](https://discord.gg/aztec)

Join the channels:

- [`#🖤│noir`](https://discord.com/channels/563037431604183070/824700393677783080) to discuss Noir
- [`#🇨🇴│ethbogota`](https://discord.com/channels/563037431604183070/1021410163221086268) to discuss the ETHBogota Hackathon
