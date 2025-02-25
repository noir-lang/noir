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

### Flamegraphing unconstrained opcodes

The profiler also enables developers to generate a flamegraph of the unconstrained execution trace. For unconstrained functions Noir compiles down to Brillig bytecode, thus we will be seeing a flamegraph of Brillig opcodes, rather than ACIR opcodes.

Let's take our initial program and simply add an `unconstrained` modifier before main (e.g. `unconstrained fn main`). Then run the following command:
```sh
noir-profiler execution-opcodes --artifact-name ./target/program.json --prover_toml_path Prover.toml --output ./target
```
This matches the `opcodes` command, except that now we need to accept a _Prover.toml_ file to profile execution with a specific set of inputs.

You can use these values for the _Prover.toml_:

```toml
ptr = 1
array = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
```

We will get the following flamegraph with 1,582 opcodes executed:
![Brillig Trace Initial Program](@site/static/img/tooling/profiler/brillig-trace-initial-32.png)

Circuit programming (ACIR) is an entirely different execution paradigm compared to regular programming. To demonstrate this point further, let's generate an execution trace for our optimized ACIR program once we have modified `main` to be `unconstrained`.

We then get the following flamegraph with 2,125 opcodes executed:
![Brillig Trace "Optimized"](@site/static/img/tooling/profiler/brillig-trace-opt-32.png)

In the above graph we are searching for `new_array`, which shows up zero matches in the initial program. In the unconstrained environment, the updated program essentially just adds extra unnecessary checks. Thus, we see a longer execution trace.

`execution-opcodes` is useful for when you are searching for bottlenecks in unconstrained code. This can be especially meaningful for optimizing witness generation. Even though unconstrained execution helps us skip proving steps, we still need to compute the relevant inputs/outputs outside of the circuit before proving.

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
