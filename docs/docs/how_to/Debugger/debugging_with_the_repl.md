---
title: Debugging Noir circuits with the REPL Debugger
description:
  Step by step guide on how to debug your Noir circuits with the REPL Debugger. 
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Debugger,
    REPL,
  ]
sidebar_position: 1
---

## Pre-requisites

In order to use the REPL debugger, first you need to install recent enough versions of Noir, Nargo and vscode-noir. 

The first versions of each that ship with support for the Noir Debugger are:

TODO-DEBUGGER SET THESE VERSIONS
-Nargo: x
-Noir: y
-vscode-noir: z

## Debugging a simple circuit

Let's debug a simple circuit:


TODO-DEBUGGER UPDATE ALL THE TERMINAL EXAMPLES

```
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```

To start the REPL debugger, using a terminal, go to a Noir circuit's home directory. Then:

`$ nargo debug`

You should be seeing this in your terminal:

```
[main] Starting debugger
At opcode 0: EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
At ~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
> 
```

The debugger displays the current opcode, and the corresponding Noir code location associated to it, and it is now waiting for us to drive it.

Let's first take a look at the available commands. For that we'll use the `help` command.

```
At ~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
> help
Available commands:

  break LOCATION:OpcodeLocation    add a breakpoint at an opcode location
  memory                           show Brillig memory (valid when executing a
                                   Brillig block)
  into                             step into to the next opcode
  next                             step until a new source location is reached
  delete LOCATION:OpcodeLocation   delete breakpoint at an opcode location
  step                             step to the next ACIR opcode
  registers                        show Brillig registers (valid when executing
                                   a Brillig block)
  regset index:usize value:String  update a Brillig register with the given
                                   value
  restart                          restart the debugging session
  witness                          show witness map
  witness index:u32                display a single witness from the witness map
  witness index:u32 value:String   update a witness with the given value
  continue                         continue execution until the end of the
                                   program
  opcodes                          display ACIR opcodes
  memset index:usize value:String  update a Brillig memory cell with the given
                                   value
  stacktrace                       display the current stack trace
  vars                             show variable values available at this point in execution

Other commands:

  help  Show this help message
  quit  Quit repl
```

The command menu is pretty self-explanatory. Some commands operate only at Brillig level, such as `memory`, `memset`, `registers`, `regset`. If you try to use them while execution is paused at an ACIR opcode, the debugger will simply inform you that you are not executing Brillig code:

```
> registers
Not executing a Brillig block
> 
```

Before continuing, we can take a look at the initial witness map:

```
> witness
_1 = 1
_2 = 2
> 
```

Cool, since `x==1`, `y==2`, and we want to check that `x != y`, our circuit should succeed. At this point we could intervene and use the witness setter command to change one of the witnesses. Let's set `y=3`, then back to 2:

```
> witness
_1 = 1
_2 = 2
> witness 2 3
_2 = 3
> witness
_1 = 1
_2 = 3
> witness 2 2
_2 = 2
> witness
_1 = 1
_2 = 2
> 
```

Let's take a look at this circuit's ACIR, using the `opcodes` command:

```
> opcodes
  0 -> EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
  1    BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
       |       outputs=[Simple(Witness(4))]
  1.0  |   JumpIfNot { condition: RegisterIndex(0), location: 3 }
  1.1  |   Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
  1.2  |   BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
  1.3  |   Stop
  2    EXPR [ (1, _3, _4) (1, _5) -1 ]
  3    EXPR [ (1, _3, _5) 0 ]
  4    EXPR [ (-1, _5) 0 ]
> 
```

**Note**: in future versions of the debugger, we could explore prettier and/or more compact formats to print opcodes.

So the next opcode will take us to Brillig execution. Let's step into opcode 1 so we can explore Brillig debugger commands.

```
> into
At opcode 1: BRILLIG: inputs: [Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
outputs: [Simple(Witness(4))]
[JumpIfNot { condition: RegisterIndex(0), location: 3 }, Const { destination: RegisterIndex(1), value: Value { inner: 1 } }, BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }, Stop]

At /~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
```

We can use the `opcodes` command to inspect the ACIR bytecode that the compiler produced for our program.

```
> opcodes
  0    EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
  1 -> BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
       |       outputs=[Simple(Witness(4))]
  1.0  |-> JumpIfNot { condition: RegisterIndex(0), location: 3 }
  1.1  |   Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
  1.2  |   BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
  1.3  |   Stop
  2    EXPR [ (1, _3, _4) (1, _5) -1 ]
  3    EXPR [ (1, _3, _5) 0 ]
  4    EXPR [ (-1, _5) 0 ]
> witness
_1 = 1
_2 = 2
_3 = 1
> 
```

We can see two arrow `->` cursors: one indicates where we are from the perspective of ACIR (opcode 1), and the other one shows us where we are in the context of the current Brillig block (opcode 1.0).

**Note**: REPL commands are autocompleted when not ambiguous, so `opcodes` can be run just with `op`, `into` with `i`, etc.

The next opcode to execute is a `JumpIfNot`, which reads from register 0. Let's inspect Brillig register state:

```
> op
  0    EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
  1 -> BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
       |       outputs=[Simple(Witness(4))]
  1.0  |-> JumpIfNot { condition: RegisterIndex(0), location: 3 }
  1.1  |   Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
  1.2  |   BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
  1.3  |   Stop
  2    EXPR [ (1, _3, _4) (1, _5) -1 ]
  3    EXPR [ (1, _3, _5) 0 ]
  4    EXPR [ (-1, _5) 0 ]
> registers
Brillig VM registers not available
```

Oops. This is unexpected, even though we were already in a Brillig block, we couldn't access the Brillig registers. This is a known issue: when just entering the Brillig block, the ACVM has not yet initialized the Brillig VM, so we can't introspect it.

**Note**: In order to solve this, we would have to change the way the ACVM works, or add special handling for this case (after all, the debugger does know we're at the first opcode of a Brillig block and could keep track of how registers will be initialized). At the time of writing, we haven't yet solved this case. 

For now, let's just step once more:

```
> i
At opcode 1.1: Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
> registers
0 = 1
>
```

Now we can see that register 0 was initialized with a value of 1, so the `JumpIfNot` didn't activate. After executing opcode 1, we should see register 1 gets a value of 1:

```
> i
At opcode 1.2: BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
> regist
0 = 1
1 = 1
>
```

The last operation will compute `Reg0 <- Reg1 / Reg0`: 

```
> i
At opcode 1.3: Stop
> registers
0 = 1
1 = 1
>
```

Once we step again, we'll be out of Brillig and back on ACVM territory. With a new witness `_4` corresponding to the result of the Brillig block execution:

```
> op
  0    EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
  1    BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
       |       outputs=[Simple(Witness(4))]
  1.0  |   JumpIfNot { condition: RegisterIndex(0), location: 3 }
  1.1  |   Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
  1.2  |   BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
  1.3  |   Stop
  2 -> EXPR [ (1, _3, _4) (1, _5) -1 ]
  3    EXPR [ (1, _3, _5) 0 ]
  4    EXPR [ (-1, _5) 0 ]
> wit
_1 = 1
_2 = 2
_3 = 1
_4 = 1
>
```

At any time, we might also decide to restart from the beginning:

```
> restart
Restarted debugging session.
At opcode 0: EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
At ~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
>
```

Let's set a breakpoint. For that, we can use the opcode id's listed by the `opcodes` command:

```
> opcodes
  0 -> EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
  1    BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
       |       outputs=[Simple(Witness(4))]
  1.0  |   JumpIfNot { condition: RegisterIndex(0), location: 3 }
  1.1  |   Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
  1.2  |   BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
  1.3  |   Stop
  2    EXPR [ (1, _3, _4) (1, _5) -1 ]
  3    EXPR [ (1, _3, _5) 0 ]
  4    EXPR [ (-1, _5) 0 ]
> break 1.2
Added breakpoint at opcode 1.2
```

Now we can have the debugger continue all the way to opcode 1.2:

```
> break 1.2
Added breakpoint at opcode 1.2
> continue
(Continuing execution...)
Stopped at breakpoint in opcode 1.2
At opcode 1.2: BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
> opcodes
  0    EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
  1 -> BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(3))], q_c: 0 })]
       |       outputs=[Simple(Witness(4))]
  1.0  |   JumpIfNot { condition: RegisterIndex(0), location: 3 }
  1.1  |   Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
  1.2  |-> BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
  1.3  |   Stop
  2    EXPR [ (1, _3, _4) (1, _5) -1 ]
  3    EXPR [ (1, _3, _5) 0 ]
  4    EXPR [ (-1, _5) 0 ]
> 
```

Let's continue to the end:

```
> continue
(Continuing execution...)
Finished execution
> q
[main] Circuit witness successfully solved
```

Upon quitting the debugger after a solved circuit, the resulting circuit witness gets saved, equivalent to what would happen if we had run the same circuit with `nargo execute`.


TODO-DEBUGGER: SHOW VARS WITH THE EXAMPLE ABOVE

Running `vars` will print the current variables in scope, and its current values:

```
At /mul_1/src/main.nr:6:5
  1    // Test unsafe integer multiplication with overflow: 12^8 = 429 981 696
  2    // The circuit should handle properly the growth of the bit size
  3    fn main(mut x: u32, y: u32, z: u32) {
  4        x *= y;
  5        x *= x; //144
  6 ->     x *= x; //20736
  7        x *= x; //429 981 696
  8        assert(x == z);
  9    }
> vars
y:UnsignedInteger { width: 32 }=Field(4), z:UnsignedInteger { width: 32 }=Field(2¹⁶×6561), x:UnsignedInteger { width: 32 }=Field(2⁴×9)
> 
```

Running `stacktrace` will print information about the current frame in the stacktrace:

TODO-DEBUGGER: SHOW VARS WITH THE EXAMPLE ABOVE

```
> stacktrace
Frame #0, opcode 12: EXPR [ (1, _5, _5) (-1, _6) 0 ]
At /1_mul/src/main.nr:6:5
  1    // Test unsafe integer multiplication with overflow: 12^8 = 429 981 696
  2    // The circuit should handle properly the growth of the bit size
  3    fn main(mut x: u32, y: u32, z: u32) {
  4        x *= y;
  5        x *= x; //144
  6 ->     x *= x; //20736
  7        x *= x; //429 981 696
  8        assert(x == z);
  9    }
> 
```

## Debugging with ACIR output


## Switching debugging instrumentation on and off

## Reference

With this guide we intend to give an overview of the most common features of the REPL debugger. For a comprehensive reference, check out [the REPL debugger reference page](../../reference/debugger_repl).
