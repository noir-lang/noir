---
title: Profiler
description: Learn about the Noir Profiler, how to generate execution flamegraphs, identify bottlenecks, and visualize optimizations.
keywords: [profiling, profiler, flamegraph]
sidebar_position: 3
---

The profiler is a sampling profiler designed to analyze and visualize Noir programs. It assists developers to identify bottlenecks by mapping execution data back to the original source code.

## Installation

The profiler is automatically installed with Nargo starting noirup v0.1.4.

Check if the profiler is already installed by running `noir-profiler --version`. If the profiler is not found, update noirup and install the profiler by [reinstalling both noirup and Nargo](../getting_started/quick_start.md#noir).

## Usage

### Create a demonstrative project

Let's start by creating a simple Noir program that aims to zero out an array past some dynamic index.

Run `nargo new program` to create a new project named _program_, then copy in the following as source code:

```rust
fn main(ptr: pub u32, mut array: [u32; 32]) -> pub [u32; 32] {
    for i in 0..32 {
        if i > ptr {
            array[i] = 0;
        }
    }
    array
}
```

Change directory into the project and compile the program using `nargo compile`. We are ready then to try the profiler out.

### Flamegraphing ACIR opcodes

Let's get a more granular look at our program's ACIR opcode footprint using the profiler, running:

```sh
noir-profiler opcodes --artifact-path ./target/program.json --output ./target/
```

The command will generate a flamegraph in your _target_ folder that maps the number of ACIR opcodes and their corresponding locations in your program source code.

Opening the flamegraph in a web browser will provide a more interactive experience, allowing you to click into different regions of the graph and examine them.

Flamegraph of the demonstrative project generated with Nargo v1.0.0-beta.2:

![ACIR Flamegraph Unoptimized](@site/static/img/tooling/profiler/acir-flamegraph-unoptimized.png)

The demonstrative project consists of 387 ACIR opcodes in total, which from the flamegraph we can see that the majority of them comes from the write to `array[i]`.

Knowing the insight on our program's bottleneck, let's optimize it.

#### Optimizing array writes with reads

We can improve our program's performance using [unconstrained functions](../noir/concepts/unconstrained.md).

Let's replace expensive array writes with array gets with the new code below:
```rust
fn main(ptr: pub u32, array: [u32; 32]) -> pub [u32; 32] {
    // Safety: Sets all elements after `ptr` in `array` to zero.
    let zeroed_array = unsafe { zero_out_array(ptr, array) };
    for i in 0..32 {
        if i > ptr {
            assert_eq(zeroed_array[i], 0);
        } else {
            assert_eq(zeroed_array[i], array[i]);
        }
    }
    zeroed_array
}

unconstrained fn zero_out_array(ptr: u32, mut array: [u32; 32]) -> [u32; 32] {
    for i in 0..32 {
        if i > ptr {
            array[i] = 0;
        }
    }
    array
}
```

Instead of writing our array in a fully constrained manner, we first write our array inside an unconstrained function and then assert every value in the array returned from the unconstrained function in a constrained manner.

This brings the ACIR opcodes count of our program down to a total of 284 opcodes:

![ACIR Flamegraph Optimized](@site/static/img/tooling/profiler/acir-flamegraph-optimized.png)

#### Searching in flamegraphs

The `i > ptr` region in the above image is highlighted purple as we were searching for it.

Click "Search" in the top right corner of the flamegraph to start a search (e.g. i > ptr).

Check "Matched" in the bottom right corner to learn the percentage out of total opcodes associated with the search (e.g. 43.3%).

:::tip

If you try searching for `memory::op` before and after the optimization, you will find that the search will no longer have matches after the optimization.

This comes from the optimization removing the use of a dynamic array (i.e. an array with a dynamic index, that is its values rely on witness inputs). After the optimized rewrite into reading two arrays from known constant indices, simple arithmetic operations replaces the original memory operations.

:::

### Flamegraphing unconstrained execution traces

The profiler also provides the ability to flamegraph a Noir program's unconstrained execution trace. This is particularly useful for searching bottlenecks in unconstrained programs and optimizing execution speeds.

#### Preparing the demonstrative project

Let's turn our demonstrative program into an unconstrained program by adding an `unconstrained` modifier to the main function:

```rust
unconstrained fn main(...){...}
```

Since we are profiling the execution trace, we will also need to provide a set of inputs to execute the program with.

Run `nargo check` to generate a _Prover.toml_ file, which you can fill it in with:

```toml
ptr = 1
array = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
```

#### Flamegraphing

Let's take a granular look at our program's unconstrained execution trace footprint using the profiler, running:

```sh
noir-profiler execution-opcodes --artifact-path ./target/program.json --prover-toml-path Prover.toml --output ./target
```

This is similar to the `opcodes` command, except it additionally takes in the _Prover.toml_ file to profile execution with a specific set of inputs.

Flamegraph of the demonstrative project generated with Nargo v1.0.0-beta.2:
![Brillig Trace "Optimized"](@site/static/img/tooling/profiler/brillig-trace-opt-32.png)

Note that unconstrained Noir functions compile down to Brillig opcodes, which is what the counts in this flamegraph stand for, rather than constrained ACIR opcodes like in the previous section.

:::tip

Optimizing constrained operations through unconstrained rewrites like what we did in [the previous section](#optimizing-array-writes-with-reads) helps remove ACIR opcodes (hence shorter proving times), but would introduce more Brillig opcodes (hence longer execution times).

For example, we can find a 13.9% match `new_array` in the flamegraph above.

In contrast, if we profile the pre-optimization demonstrative project:
![Brillig Trace Initial Program](@site/static/img/tooling/profiler/brillig-trace-initial-32.png)

You will notice that it does not consist any `new_array`, and executes a total of 1,582 Brillig opcodes (versus 2,125 Brillig opcodes post-optimization).

As new unconstrained functions were added, it is reasonable that the program would consist of more Brillig opcodes. That said, the tradeoff is often easily justifiable by the fact that proving speeds are more commonly the major bottleneck of Noir programs versus execution speeds.

This is however good to keep in mind in case you start noticing execution speeds being the bottleneck of your program, or if you are simply looking to optimize your program's execution speeds.

:::











### Flamegraphing proving backend gates

ACIR opcodes do not give us a full picture of where the cost of this program lies.
The `gates` command also accepts a backend binary. In the [quick start guide](../getting_started/quick_start.md#proving-backend) you can see how to get started with the [Barretenberg proving backend](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg).

Run the following command:
```sh
noir-profiler gates --artifact-path ./target/program.json --backend-path bb --output ./target
```
`--backend-path` accepts a path to the backend binary. In the above command we assume that you have the backend binary path saved in your PATH. If you do not, you will have to pass the binary's absolute path.

This produces the following flamegraph with 3,737 total backend gates (using bb v0.76.4):
![Gates Flamegraph Unoptimized](@site/static/img/tooling/profiler/gates-flamegraph-unoptimized.png)

Searching for ACIR `memory::op` opcodes, they look to cause about 18.2% of the backend gates.

You will notice that the majority of the backend gates come from the ACIR range opcodes. This is due to the way UltraHonk handles range constraints, which is the backend used in this example. UltraHonk uses lookup tables internally for its range gates. These can take up the majority of the gates for a small circuit, but whose impact becomes more meaningful in larger circuits. If our array was much larger, range gates would become a much smaller percentage of our total circuit.
Here is an example backend gates flamegraph for the same program in this guide but with an array of size 2048:
![Gates Flamegraph Unoptimized 2048](@site/static/img/tooling/profiler/gates-flamegraph-unoptimized-2048.png)
Every backend implements ACIR opcodes differently, so it is important to profile both the ACIR and the backend gates to get a full picture.

Now let's generate a graph for our optimized circuit with an array of size 32. We get the following flamegraph that produces 3,062 total backend gates:
![Gates Flamegraph Optimized](@site/static/img/tooling/profiler/gates-flamegraph-optimized.png)

In the optimized flamegraph, we searched for the backend gates due to `i > ptr` in the source code. The backend gates associated with this call stack were only 3.8% of the total backend gates. If we look back to the ACIR flamegraph, that same code was the cause of 43.3% ACIR opcodes. This discrepancy reiterates the earlier point about profiling both the ACIR opcodes and backend gates.

For posterity, here is the flamegraph for the same program with a size 2048 array:
![Gates Flamegraph Optimized 2048](@site/static/img/tooling/profiler/gates-flamegraph-optimized-2048.png)
