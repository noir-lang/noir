---
title: REPL Debugger
description:
  Noir Debugger REPL options and commands. 
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Debugger,
    REPL,
  ]
sidebar_position: 1
---

## Running the REPL debugger

`nargo debug [OPTIONS] [WITNESS_NAME]`

Runs the Noir REPL debugger. If a `WITNESS_NAME` is provided the debugger writes the resulting execution witness to a `WITNESS_NAME` file.

### Options

| Option                | Description                                                  |
| --------------------- | ------------------------------------------------------------ |
| `-p, --prover-name <PROVER_NAME>` | The name of the toml file which contains the inputs for the prover [default: Prover]|
| `--package <PACKAGE>` | The name of the package to debug                             |
| `--print-acir`        | Display the ACIR for compiled circuit                        |
| `--deny-warnings`     | Treat all warnings as errors                                 |
| `--silence-warnings`  | Suppress warnings                                            |
| `-h, --help`          | Print help                                                   |

None of these options are required.

:::note
Since the debugger starts by compiling the target package, all Noir compiler options are also available. Check out the [compiler reference](../nargo_commands.md#nargo-compile) to learn more about the compiler options.
:::

## REPL commands

Once the debugger is running, it accepts the following commands.

#### `help` (h)

Displays the menu of available commands.

```
> help
Available commands:

  opcodes                          display ACIR opcodes
  into                             step into to the next opcode
  next                             step until a new source location is reached
  out                              step until a new source location is reached
                                   and the current stack frame is finished
  break LOCATION:OpcodeLocation    add a breakpoint at an opcode location
  over                             step until a new source location is reached
                                   without diving into function calls
  restart                          restart the debugging session
  delete LOCATION:OpcodeLocation   delete breakpoint at an opcode location
  witness                          show witness map
  witness index:u32                display a single witness from the witness map
  witness index:u32 value:String   update a witness with the given value
  memset index:usize value:String  update a memory cell with the given
                                   value
  continue                         continue execution until the end of the
                                   program
  vars                             show variable values available at this point
                                   in execution
  stacktrace                       display the current stack trace
  memory                           show memory (valid when executing unconstrained code)                                 value
  step                             step to the next ACIR opcode

Other commands:

  help  Show this help message
  quit  Quit repl

```

### Stepping through programs

#### `next` (n)

Step until the next Noir source code location. While other commands, such as [`into`](#into-i) and [`step`](#step-s), allow for finer grained control of the program's execution at the opcode level, `next` is source code centric. For example:

```
3    ...
4    fn main(x: u32) {
5        assert(entry_point(x) == 2);
6        swap_entry_point(x, x + 1);
7 ->     assert(deep_entry_point(x) == 4);
8        multiple_values_entry_point(x);
9    }
```


Using `next` here would cause the debugger to jump to the definition of `deep_entry_point` (if available). 

If you want to step over `deep_entry_point` and go straight to line 8, use [the `over` command](#over) instead.

#### `over`

Step until the next source code location, without diving into function calls. For example:

```
3    ...
4    fn main(x: u32) {
5        assert(entry_point(x) == 2);
6        swap_entry_point(x, x + 1);
7 ->     assert(deep_entry_point(x) == 4);
8        multiple_values_entry_point(x);
9    }
```


Using `over` here would cause the debugger to execute until line 8 (`multiple_values_entry_point(x);`).

If you want to step into `deep_entry_point` instead, use [the `next` command](#next-n).

#### `out`

Step until the end of the current function call. For example:

```
  3    ...
  4    fn main(x: u32) {
  5        assert(entry_point(x) == 2);
  6        swap_entry_point(x, x + 1);
  7 ->     assert(deep_entry_point(x) == 4);
  8        multiple_values_entry_point(x);
  9    }
 10    
 11    unconstrained fn returns_multiple_values(x: u32) -> (u32, u32, u32, u32) {
 12    ...
 ...
 55    
 56    unconstrained fn deep_entry_point(x: u32) -> u32 {
 57 ->     level_1(x + 1)
 58    }

```

Running `out` here will resume execution until line 8.

#### `step` (s)

Skips to the next ACIR code. A compiled Noir program is a sequence of ACIR opcodes. However, an unconstrained VM opcode denotes the start of an unconstrained code block, to be executed by the unconstrained VM. For example (redacted for brevity):

```
0  BLACKBOX::RANGE [(_0, num_bits: 32)] [ ]
1 ->  BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 })] outputs=[Simple(Witness(1))]
	1.0  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(0) }
	1.1  |   Const { destination: RegisterIndex(0), value: Value { inner: 0 } }
	1.2  |   Const { destination: RegisterIndex(1), value: Value { inner: 0 } }
	1.3  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(2) }
	1.4  |   Call { location: 7 }
	...
	1.43 |   Return
2    EXPR [ (1, _1) -2 ]
```

The `->` here shows the debugger paused at an ACIR opcode: `BRILLIG`, at index 1, which denotes an unconstrained code block is about to start.

Using the `step` command at this point would result in the debugger stopping at ACIR opcode 2, `EXPR`, skipping unconstrained computation steps.

Use [the `into` command](#into-i) instead if you want to follow unconstrained computation step by step.

#### `into` (i)

Steps into the next opcode. A compiled Noir program is a sequence of ACIR opcodes. However, a BRILLIG opcode denotes the start of an unconstrained code block, to be executed by the unconstrained VM. For example (redacted for brevity):

```
0  BLACKBOX::RANGE [(_0, num_bits: 32)] [ ]
1 ->  BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 })] outputs=[Simple(Witness(1))]
	1.0  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(0) }
	1.1  |   Const { destination: RegisterIndex(0), value: Value { inner: 0 } }
	1.2  |   Const { destination: RegisterIndex(1), value: Value { inner: 0 } }
	1.3  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(2) }
	1.4  |   Call { location: 7 }
	...
	1.43 |   Return
2    EXPR [ (1, _1) -2 ]
``` 

The `->` here shows the debugger paused at an ACIR opcode: `BRILLIG`, at index 1, which denotes an unconstrained code block is about to start.

Using the `into` command at this point would result in the debugger stopping at opcode 1.0, `Mov ...`, allowing the debugger user to follow unconstrained computation step by step.

Use [the `step` command](#step-s) instead if you want to skip to the next ACIR code directly.

#### `continue` (c)

Continues execution until the next breakpoint, or the end of the program.

#### `restart` (res)

Interrupts execution, and restarts a new debugging session from scratch.

#### `opcodes` (o)

Display the program's ACIR opcode sequence. For example:

```
0  BLACKBOX::RANGE [(_0, num_bits: 32)] [ ]
1 ->  BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 })] outputs=[Simple(Witness(1))]
	1.0  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(0) }
	1.1  |   Const { destination: RegisterIndex(0), value: Value { inner: 0 } }
	1.2  |   Const { destination: RegisterIndex(1), value: Value { inner: 0 } }
	1.3  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(2) }
	1.4  |   Call { location: 7 }
	...
	1.43 |   Return
2     EXPR [ (1, _1) -2 ]
```

### Breakpoints

#### `break [Opcode]` (or shorthand `b [Opcode]`)

Sets a breakpoint on the specified opcode index. To get a list of the program opcode numbers, see [the `opcode` command](#opcodes-o). For example:

```
0  BLACKBOX::RANGE [(_0, num_bits: 32)] [ ]
1 ->  BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 })] outputs=[Simple(Witness(1))]
	1.0  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(0) }
	1.1  |   Const { destination: RegisterIndex(0), value: Value { inner: 0 } }
	1.2  |   Const { destination: RegisterIndex(1), value: Value { inner: 0 } }
	1.3  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(2) }
	1.4  |   Call { location: 7 }
	...
	1.43 |   Return
2    EXPR [ (1, _1) -2 ]
```

In this example, issuing a `break 1.2` command adds break on opcode 1.2, as denoted by the `*` character:

```
0  BLACKBOX::RANGE [(_0, num_bits: 32)] [ ]
1 ->  BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(0))], q_c: 0 })] outputs=[Simple(Witness(1))]
	1.0  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(0) }
	1.1  |   Const { destination: RegisterIndex(0), value: Value { inner: 0 } }
	1.2  | * Const { destination: RegisterIndex(1), value: Value { inner: 0 } }
	1.3  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(2) }
	1.4  |   Call { location: 7 }
	...
	1.43 |   Return
2    EXPR [ (1, _1) -2 ]
```

Running [the `continue` command](#continue-c) at this point would cause the debugger to execute the program until opcode 1.2.

#### `delete [Opcode]` (or shorthand `d [Opcode]`)

Deletes a breakpoint at an opcode location. Usage is analogous to [the `break` command](#).

### Variable inspection

#### vars

Show variable values available at this point in execution.

:::note
The ability to inspect variable values from the debugger depends on compilation to be run in a special debug instrumentation mode. This instrumentation weaves variable tracing code with the original source code. 

So variable value inspection comes at the expense of making the resulting ACIR bytecode bigger and harder to understand and optimize.

If you find this compromise unacceptable, you can run the debugger with the flag `--skip-debug-instrumentation`. This will compile your circuit without any additional debug information, so the resulting ACIR bytecode will be identical to the one produced by standard Noir compilation. However, if you opt for this, the `vars` command will not be available while debugging.
:::


### Stacktrace

#### `stacktrace`

Displays the current stack trace.


### Witness map

#### `witness` (w)

Show witness map. For example:

```
_0 = 0
_1 = 2
_2 = 1
```

#### `witness [Witness Index]`

Display a single witness from the witness map. For example:

```
> witness 1
_1 = 2
```

#### `witness [Witness Index] [New value]`

Overwrite the given index with a new value. For example:

```
> witness 1 3
_1 = 3
```


### Unconstrained VM memory

#### `memory`

Show unconstrained VM memory state. For example:

```
> memory
At opcode 1.13: Store { destination_pointer: RegisterIndex(0), source: RegisterIndex(3) }
...
> registers
0 = 0
1 = 10
2 = 0
3 = 1
4 = 1
5 = 2³²
6 = 1
> into
At opcode 1.14: Const { destination: RegisterIndex(5), value: Value { inner: 1 } }
...
> memory
0 = 1
>
```

In the example above: we start with clean memory, then step through a `Store` opcode which stores the value of register 3 (1) into the memory address stored in register 0 (0). Thus now `memory` shows memory address 0 contains value 1.

:::note
This command is only functional while the debugger is executing unconstrained code.
:::

#### `memset [Memory address] [New value]`

Update a memory cell with the given value. For example:

```
> memory
0 = 1
> memset 0 2
> memory
0 = 2
> memset 1 4
> memory
0 = 2
1 = 4
>
```

:::note
This command is only functional while the debugger is executing unconstrained code.
:::