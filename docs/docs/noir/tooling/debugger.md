---
title: Debugger
description: Learn about the Noir Debugger, in its REPL or VS Code versions.
keywords: [Nargo, VSCode, Visual Studio Code, REPL, Debugger]
sidebar_position: 2
---

# Noir Debugger

There are currently two ways of debugging Noir programs:

1. From VS Code, via the [vscode-noir](https://github.com/noir-lang/vscode-noir) extension. You can install it via the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir).
2. Via the REPL debugger, which ships with Nargo.

In order to use either version of the debugger, you will need to install recent enough versions of Noir, [Nargo](../../getting_started/installation) and vscode-noir:

- Noir 0.xx 
- Nargo 0.xx
- vscode-noir 0.xx

:::info
At the moment, the debugger supports debugging binary projects, but not contracts.
:::

## VS Code debugger quickstart

Once you installed Nargo and the vscode-noir extension, you can start debugging your Noir programs by simply opening a `.nr` file, clicking the debugger pane, and clicking _Run and debug_. Alternatively you can just use the `F5` keyboard shortcut.

You should be seeing something like this:

![Screencast of VS Code Noir Debugger](@site/static/img/debugger/debugger-intro.gif)
)

We cover the VS Code Noir debugger more in depth in [its how-to guide](../../how_to/debugger/debugging_with_vs_code.md) and [its reference](../../reference/debugger/debugger_vscode.md).

## REPL debugger quickstart

Let's debug a simple circuit:

```rust
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```

To start the REPL debugger, using a terminal, go to a Noir circuit's home directory. Then:

`$ nargo debug`

You should be seeing something similar to this in your terminal:

```
[main] Starting debugger
At ~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
> 
```


That's it! The debugger displays the current Noir code location, and it is now waiting for us to drive it. You can explore the available commands to drive the debugger with `help`, like here:

```
At ~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
> help
Available commands:

  break LOCATION:OpcodeLocation    add a breakpoint at an opcode location
  memory                           show Brillig memory (valid when executing unconstrained code)
  into                             step into to the next opcode
  next                             step until a new source location is reached
  delete LOCATION:OpcodeLocation   delete breakpoint at an opcode location
  step                             step to the next ACIR opcode
  registers                        show Brillig registers (valid when executing unconstrained code)
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

Other commands:

  help  Show this help message
  quit  Quit repl
```


We'll cover each command in depth in [the REPL debugger how-to](../../how_to/debugger/debugging_with_the_repl.md) and [its reference](../../reference/debugger/debugger_repl.md).
