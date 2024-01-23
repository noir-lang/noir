---
title: Debugger
description: Learn about the Noir Debugger, in its REPL or VS Code versions.
keywords: [Nargo, VSCode, Visual Studio Code, REPL, Debugger]
sidebar_position: 2
---

# Noir Debugger

There are currently two ways of debugging Noir programs:

1. From VS Code, via the [vscode-noir](https://github.com/noir-lang/vscode-noir) extension. You can install it via the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir).
2. By using the REPL debugger, which ships with Nargo.

In order to use either version of the debugger, you will need to install recent enough versions of Noir, Nargo and vscode-noir:

(TODO-DEBUGGER-DOCS: update to correct version number at the time of removing the feature flag)
- Noir 0.xx 
- Nargo 0.xx
- vscode-noir 0.xx

> **Note:** At the moment, the debugger supports debugging binary projects, but not contracts.


## Debugging with VS Code

Once you installed Nargo and the [vscode-noir](https://github.com/noir-lang/vscode-noir) extension, you can start debugging your Noir programs by simply opening a `.nr` file, clicking the debugger pane, and clicking `Run and debug`. Alternatively you can just use the `F5` keyboard shortcut.

You should be now seeing something like this:

(TODO-DEBUGGER-DOCS: replace gif)

![Screen Recording 2023-12-18 at 14 14 28](https://github.com/manastech/noir/assets/651693/36b4becb-953a-4158-9c5a-7a185673f54f)

We'll cover the VS Code debugger more in depth in (TODO-DEBUGGER-DOCS: guide) and (TODO-DEBUGGER-DOCS: reference) 

## REPL debugger quickstart

In order to use the REPL debugger, you will need to install a new enough version of Nargo. The debugger ships with Nargo since version 0.22 (TODO-DEBUGGER-DOCS: update to correct version number at the time of removing the feature flag).

Let's debug a simple circuit:

```
fn main(x : Field, y : pub Field) {
    assert(x != y);
}
```

To start the REPL debugger, using a terminal, go to a Noir circuit's home directory. Then:

`$ nargo debug`

You should be seeing something similar to this in your terminal:

```
[main] Starting debugger
At opcode 0: EXPR [ (-1, _1) (1, _2) (-1, _3) 0 ]
At ~/noir-examples/recursion/circuits/main/src/main.nr:2:12
  1    fn main(x : Field, y : pub Field) {
  2 ->     assert(x != y);
  3    }
> 
```


That's it! The debugger displays the current opcode and the corresponding Noir code location associated to it, and it is now waiting for us to drive it. You can explore the available commands to drive the debugger with `help`, like here:

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

Other commands:

  help  Show this help message
  quit  Quit repl
```


We'll cover each command in depth in (TODO-DEBUGGER-DOCS: link to REPL guide) and (TODO-DEBUGGER-DOCS: link to REPL REFERENCE). 
