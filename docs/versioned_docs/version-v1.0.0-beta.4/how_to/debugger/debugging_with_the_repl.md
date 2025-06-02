---
title: Using the REPL Debugger
description:
  Step-by-step guide on how to debug your Noir circuits with the REPL Debugger. 
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Debugger,
    REPL,
  ]
sidebar_position: 1
---

#### Pre-requisites

In order to use the REPL debugger, first you need to install recent enough versions of Nargo and vscode-noir. 

## Debugging a simple circuit

Let's debug a simple circuit:

```rust
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```

To start the REPL debugger, using a terminal, go to a Noir circuit's home directory. Then:

`$ nargo debug`

You should be seeing this in your terminal:

```
[main] Starting debugger
At ~/noir-examples/recursion/circuits/main/src/main.nr:1:9
  1 -> fn main(x : Field, y : pub Field) {
  2        assert(x != y);
  3    }
> 
```

The debugger displays the current Noir code location, and it is now waiting for us to drive it.

Let's first take a look at the available commands. For that we'll use the `help` command.

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
  memory                           show memory (valid when executing unconstrained code)
  step                             step to the next ACIR opcode

Other commands:

  help  Show this help message
  quit  Quit repl

```

Some commands operate only for unconstrained functions, such as `memory` and `memset`. If you try to use them while execution is paused at an ACIR opcode, the debugger will simply inform you that you are not executing unconstrained code:

```
> memory
Unconstrained VM memory not available
> 
```

Before continuing, we can take a look at the initial witness map:

```
> witness
_0 = 1
_1 = 2
>
```

Cool, since `x==1`, `y==2`, and we want to check that `x != y`, our circuit should succeed. At this point we could intervene and use the witness setter command to change one of the witnesses. Let's set `y=3`, then back to 2, so we don't affect the expected result:

```
> witness
_0 = 1
_1 = 2
> witness 1 3
_1 = 3
> witness
_0 = 1
_1 = 3
> witness 1 2
_1 = 2
> witness
_0 = 1
_1 = 2
>
```

Now we can inspect the current state of local variables. For that we use the `vars` command. 

```
> vars
>
```

We currently have no vars in context, since we are at the entry point of the program. Let's use `next` to execute until the next point in the program.

```
> vars
> next
At ~/noir-examples/recursion/circuits/main/src/main.nr:1:20
  1 -> fn main(x : Field, y : pub Field) {
  2        assert(x != y);
  3    }
> vars
x:Field = 0x01
```

As a result of stepping, the variable `x`, whose initial value comes from the witness map, is now in context and returned by `vars`.

```
> next
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
> vars
y:Field = 0x02
x:Field = 0x01
```

Stepping again we can finally see both variables and their values. And now we can see that the next assertion should succeed.

Let's continue to the end:

```
> continue
(Continuing execution...)
Finished execution
> q
[main] Circuit witness successfully solved
```

Upon quitting the debugger after a solved circuit, the resulting circuit witness gets saved, equivalent to what would happen if we had run the same circuit with `nargo execute`.

We just went through the basics of debugging using Noir REPL debugger. For a comprehensive reference, check out [the reference page](../../reference/debugger/debugger_repl.md).
