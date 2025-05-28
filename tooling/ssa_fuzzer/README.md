# SSA Fuzzer

A fuzzing tool for testing and comparing ACIR and Brillig implementations based on [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

1) [How to use](#usage)
2) [How it works](#how-it-works)

## Overview

The SSA Fuzzer generates arbitrary SSA programs and executes them in order to find inconsistencies between ACIR and Brillig implementations.

1) Fuzzer generates test (random bytes);
2) This test converts into struct, that defines commands to generate program and initial witness;
3) Fuzzer creates ssa function builders in ACIR and Brillig runtimes;
4) Fuzzer applies commands to both builders (insert instruction, insert jmp_if etc.);
5) Fuzzer compiles ssa (from both ACIR and Brillig builders), runs it and compares results;
6) If results are different, fuzzer crashes, it's a bug.

There are two additional fuzzer modes:
1) Constrain idempotent morphing mode. There are several idempotent morphing commands defined (e.g `c = a + b - b`). In this mode fuzzer will create two more ACIR and Brillig builders, that will constrain the result of idempotent morphed variable. Then will compare result of execution in normal mode and in Constrain idempotent morphing mode. This was done in order to catch bugs described [here](https://github.com/noir-lang/noir/issues/8095). To enable it set `constrain_idempotent_morphing_enabled` to true in [fuzz target](fuzzer/src/fuzz_target.rs).
2) Constant execution mode. In this mode fuzzer will create two more ACIR and Brillig builders, that will execute all instructions in constant mode (instead of variables with insert constants into builders). Then will compare result of execution in normal mode and in Constant execution mode. This was done in order to catch bugs in constant_folding SSA pass. To enable it set `constant_execution_enabled` to true in [fuzz target](fuzzer/src/fuzz_target.rs).


## Usage

1. Setup environment:
```
cargo install cargo-fuzz
```

2. Run fuzzer:
```
cargo +nightly fuzz run base_target --fuzz-dir ./fuzzer
```

or in 5 threads
```
cargo +nightly fuzz run base_target --fuzz-dir ./fuzzer -- -jobs=5 -workers=5
```

3. Triage crashes:
```
TRIAGE=FULL/FINAL cargo +nightly fuzz run base_target --fuzz-dir ./fuzzer PATH_TO_CRASH
```
FULL mode will show all SSA passes, FINAL mode will show only the final SSA pass (After Dead Instruction Elimination (3)).

4. Minimize crashes:
```
cargo +nightly fuzz tmin base_target --fuzz-dir ./fuzzer PATH_TO_CRASH -runs=1000
```


## How it works
### TLDR
Creates ssa function builders in ACIR and Brillig runtimes, generates random sequence of [methods](src/builder.rs) that will be called for both builders, creates ssa from them, runs these ssa with `noir_ssa_executor` and compares results. If inconsistency found, fuzzer will crash, its a bug.



`cargo-fuzz` generates random sequences of bytes and converts them into `FuzzerData` struct.
```rs
struct FuzzerData {
    blocks: Vec<InstructionBlock>,
    commands: Vec<FuzzerCommand>,
    initial_witness: [WitnessValue; (NUMBER_OF_VARIABLES_INITIAL - 2) as usize],
    return_instruction_block_idx: usize,
}
```
This struct defines:
### `blocks`
Sequences of low-level ssa [instructions](fuzzer/src/instruction.rs), e.g. `Add`, `Store` etc.
### `commands`
Sequence of [FuzzerCommands](fuzzer/src/fuzz_lib/base_context.rs#L24-L39). Commands applied to both ACIR and Brillig builders.
```rs
enum FuzzerCommand {
    InsertSimpleInstructionBlock { instruction_block_idx: usize },
    MergeInstructionBlocks { first_block_idx: usize, second_block_idx: usize },
    InsertJmpIfBlock { block_then_idx: usize, block_else_idx: usize },
    InsertJmpBlock { block_idx: usize },
    SwitchToNextBlock,
}
```
#### `InsertSimpleInstructionBlock`
Insert instruction block into current SSA block.
#### `MergeInstructionBlocks`
Merges two instruction blocks and stores the result block.
#### `InsertJmpIfBlock`
Terminates current SSA block with jmp_if_else. Creates two new SSA blocks from chosen InstructionBlocks.
Switches current_block_context to then_branch.
Adds else_branch to the next_block_queue. If current SSA block is already terminated, skip.
Suppose we had program with such CFG:
```
   b0 <--- current block
```
This command creates two new SSA blocks.
```
   b0 <--- current block
  b1 b2
```
Inserts chosen instruction blocks into b1 and b2.
Then terminates b0 with jmp_if_else and switch context to b1. Condtion for jmp_if_else is the last defined boolean variable in the block.
```
       b0
 then ↙  ↘  else
     b1   b2
     ↑
     |
 current block
```
b2 is stored in next_block_queue.

#### `InsertJmpBlock`
Terminates current SSA block with jmp. Creates new SSA block from chosen InstructionBlock.
Switches current_block_context to jmp_destination.
Suppose we had program with such CFG:
```
   b0 <--- current block
```
This command creates new SSA block. And adds instructions from chosen InstructionBlock into it.
```
   b0 <--- current block
   b1
```
Terminates b0 with jmp and switch context to b1.
```
   b0
   ↓
   b1 <--- current block
```
#### `SwitchToNextBlock`
Suppose we had program with such CFG:
```
       b0
 then ↙  ↘  else
     b1   b2
     ↑
     |
 current block
```
This command adds current SSA block to the next_block_queue. Switches context to stored in next_block_queue.
```
       b0
 then ↙  ↘  else
     b1   b2
          ↑
          |
      current block
```


### `initial_witness`
Initial witness values for the program as `WitnessValue`.
### `return_instruction_block_idx`
Index of the instruction block that will inserted into return block.

This struct is then passed to [fuzz_target](fuzzer/src/fuzz_target.rs) function. It creates ssa function builders, and process `commands` as described above.
Then merges all ssa blocks into one (because noir programs can only have one return block and every ssa block mustn't have more than two successors), inserts return block with chosen instruction block and runs ssa with `noir_ssa_executor`.

## Example of hand-written `FuzzerData`

1) [Branching](fuzzer/src/fuzz_lib/fuzz_target_lib.rs#L110-L170)
2) [Mutable variables](fuzzer/src/fuzz_lib/fuzz_target_lib.rs#L186-L230)

