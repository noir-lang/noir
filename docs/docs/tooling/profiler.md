---
title: Profiler
description: Learn about the Noir Profiler, how to generate execution flamegraphs, identify bottlenecks, and visualize optimizations.
keywords: [profiling, profiler, flamegraph]
---

The profiler is a sampling profiler designed to analyze and visualize Noir programs. It assists developers to identify bottlenecks by mapping execution data back to the original source code.

## Installation

The profiler is automatically installed with Nargo starting noirup v0.1.4.

Check if the profiler is already installed by running `noir-profiler --version`. If the profiler is not found, update noirup and install the profiler by [reinstalling both noirup and Nargo](../getting_started_manually.md#installing-nargo).

## Usage

### Profiling ACIR opcodes

The profiler provides the ability to flamegraph a Noir program's ACIR opcode footprint. This is useful for _approximately_ identifying bottlenecks in constrained execution and proving of Noir programs.

:::note

"_Approximately_" because:
- Execution speeds depend on the constrained execution trace compiled from ACIR opcodes
- Proving speeds depend on how the proving backend of choice interprets the ACIR opcodes

:::

#### Create a demonstrative project

Let's start by creating a simple Noir program that converts a `u64` into its big-endian byte representation.

Run `nargo new program` to create a new project named _program_, then copy in the following source code:

```rust
fn main(num: u64) -> pub [u8; 8] {
    let mut out: [u8; 8] = [0; 8];
    for i in 0..8 {
        out[i] = (num >> (56 - (i as u64 * 8))) as u8;
    }
    out
}
```

Change into the project directory and compile the program using `nargo compile`. We are now ready to try out the profiler.

#### Flamegraphing

Let's take a granular look at our program's ACIR opcode footprint using the profiler, running:

```sh
noir-profiler opcodes --artifact-path ./target/program.json --output ./target/
```

The command generates a flamegraph in your _target_ folder that maps the number of ACIR opcodes to their corresponding locations in your program's source code.

Opening the flamegraph in a web browser will provide a more interactive experience, allowing you to click into different regions of the graph and examine them.

Flamegraph of the demonstrative project generated with Nargo v1.0.0-beta.22:

![ACIR Flamegraph Constrained u64 Conversion](@site/static/img/tooling/profiler/acir-flamegraph-u64-constrained.png)

The demonstrative project consists of 65 ACIR opcodes in total. From the flamegraph, we can see that the shift-and-cast expression used to extract each byte contributes a large portion of the opcode footprint.

With insight into our program's bottleneck, let's optimize it.

#### Visualizing optimizations

We can improve our program's constrained footprint using [unconstrained functions](../language/unconstrained.md).

Let's move the byte conversion into an unconstrained function, then constrain the returned bytes by reconstructing the original `u64`:

```rust
fn main(num: u64) -> pub [u8; 8] {
    // Safety: `out` is constrained by reconstructing `num` below.
    let out = unsafe { u64_to_u8(num) };

    let mut reconstructed_num = 0;
    for i in 0..8 {
        reconstructed_num += (out[i] as u64 << (56 - (8 * i as u64)));
    }
    assert(num == reconstructed_num);
    out
}

unconstrained fn u64_to_u8(num: u64) -> [u8; 8] {
    let mut out: [u8; 8] = [0; 8];
    for i in 0..8 {
        out[i] = (num >> (56 - (i as u64 * 8))) as u8;
    }
    out
}
```

Instead of performing the full byte conversion in a constrained context, we first compute the bytes inside an unconstrained function. Then, we assert in a constrained context that those bytes reconstruct the original input.

This brings the ACIR opcode count of our program down to 33 opcodes:

![ACIR Flamegraph Unconstrained u64 Conversion](@site/static/img/tooling/profiler/acir-flamegraph-u64-unconstrained.png)

The optimized artifact also contains a `u64_to_u8_0_brillig` function with 114 Brillig opcodes. This illustrates the usual tradeoff: moving work out of ACIR can reduce proving work, but it can introduce additional unconstrained execution work.

#### Searching

The `num >>` region in the constrained flamegraph contributes 43.08% of the ACIR opcode footprint.

Click "Search" in the top right corner of the flamegraph to start a search, for example `num >>`.

Check "Matched" in the bottom right corner to learn the percentage out of total opcodes associated with the search.

:::tip

Try searching for `u64_to_u8` in the optimized ACIR flamegraph. The search highlights the call boundary into the unconstrained function, while the full byte conversion work appears in the generated Brillig function.

:::

### Profiling proving backend gates

The profiler further provides the ability to flamegraph a Noir program's proving backend gates footprint. This is useful for fully identifying proving bottlenecks of Noir programs.

This feature depends on the proving backend you are using and whether it supports the profiler with a gate profiling API. We will use [Barretenberg](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg) as an example here.

#### Flamegraphing

Let's take a granular look at our program's proving backend gates footprint using the profiler, running:

```sh
noir-profiler gates --artifact-path ./target/program.json --backend-path bb --output ./target -- --include_gates_per_opcode
```

The `--backend-path` flag takes in the path to your proving backend binary.

The above command assumes you have Barretenberg (bb) installed and that its path is saved in your PATH. If that is not the case, you can pass in the absolute path to your proving backend binary instead.

Flamegraph of the optimized demonstrative project generated with bb v5.0.0-nightly.20260522:

![Gates Flamegraph Unconstrained u64 Conversion](@site/static/img/tooling/profiler/gates-flamegraph-u64-unconstrained.png)

The optimized demonstrative project reports 2,835 gates by opcode, with a backend circuit size of 2,859.

:::note

ACIR opcode counts are useful approximations, but they are not the same as proving backend gate counts.

For example, the optimized version reduces the ACIR opcode count from 65 to 33. The backend gates by opcode decrease from 3,758 to 2,835. The improvement is still meaningful, but it is not proportional because the backend decides how each opcode is translated into proving gates.

:::

#### Understanding bottlenecks

From the optimized gates flamegraph above, you will notice that `blackbox::range` contributes the majority of the backend gates. This comes from how Barretenberg UltraHonk uses lookup tables for its range gates under the hood, which comes with a considerable but fixed setup cost in terms of proving gates.

For comparison, this is the gates flamegraph of the constrained version of the same program:

![Gates Flamegraph Constrained u64 Conversion](@site/static/img/tooling/profiler/gates-flamegraph-u64-constrained.png)

The constrained version reports 3,758 gates by opcode, with a backend circuit size of 3,782.

Every proving backend interprets ACIR opcodes differently, so it is important to profile proving backend gates to get the full picture of proving performance.

### Profiling execution traces (unconstrained)

The profiler also provides the ability to flamegraph a Noir program's execution trace. This is useful for identifying execution bottlenecks of Noir programs.

The profiler supports profiling fully unconstrained Noir programs at this moment.

#### Updating the demonstrative project

Let's profile the execution trace for the same byte conversion by making `main` fully unconstrained:

```rust
unconstrained fn main(num: u64) -> pub [u8; 8] {
    let mut out: [u8; 8] = [0; 8];
    for i in 0..8 {
        out[i] = (num >> (56 - (i as u64 * 8))) as u8;
    }
    out
}
```

Since we are profiling the execution trace, we will also need to provide a set of inputs to execute the program with.

Run `nargo check` to generate a _Prover.toml_ file, then fill it in with:

```toml
num = "72623859790382856"
```

#### Flamegraphing

Let's take a granular look at our program's unconstrained execution trace footprint using the profiler, running:

```sh
noir-profiler execution-opcodes --artifact-path ./target/program.json --prover-toml-path Prover.toml --output ./target
```

This is similar to the `opcodes` command, except it additionally takes in the _Prover.toml_ file to profile execution with a specific set of inputs.

Flamegraph of the demonstrative project generated with Nargo v1.0.0-beta.22:

![Brillig Trace u64 Conversion](@site/static/img/tooling/profiler/brillig-trace-u64-unconstrained.png)

The execution trace above consists of 348 Brillig samples. Note that unconstrained Noir functions compile down to Brillig opcodes, which is what the counts in this flamegraph stand for, rather than constrained ACIR opcodes like in the previous sections.

#### Balancing proving and execution optimizations

Rewriting constrained operations with unconstrained operations like what we did in [the optimization section](#visualizing-optimizations) helps remove ACIR opcodes and can reduce proving work, but may introduce more Brillig opcodes and therefore more unconstrained execution work.

In the optimized constrained version, `noir-profiler opcodes` reports 33 ACIR opcodes for `main` and 114 Brillig opcodes for the generated `u64_to_u8_0_brillig` function. This is often a reasonable tradeoff when proving is the bottleneck, but it is worth measuring both sides if execution time also matters for your program.
