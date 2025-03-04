---
title: Known limitations
description:
  An overview of known limitations of the current version of the Noir debugger
keywords:
  [
    Nargo,
    Noir Debugger,
    VS Code,
  ]
sidebar_position: 2
---

# Debugger Known Limitations

There are currently some limits to what the debugger can observe. 

## Mutable references

The debugger is currently blind to any state mutated via a mutable reference. For example, in:

```
let mut x = 1;
let y = &mut x;
*y = 2;
```

The update on `x` will not be observed by the debugger. That means, when running `vars` from the debugger REPL, or inspecting the _local variables_ pane in the VS Code debugger, `x` will appear with value 1 despite having executed `*y = 2;`.

## Variables of type function or mutable references are opaque

When inspecting variables, any variable of type `Function` or `MutableReference` will render its value as `<<function>>` or `<<mutable ref>>`.

## Debugger instrumentation affects resulting ACIR
                      
In order to make the state of local variables observable, the debugger compiles Noir circuits interleaving foreign calls that track any mutations to them. While this works (except in the cases described above) and doesn't introduce any behavior changes, it does as a side effect produce bigger bytecode. In particular, when running the command `opcodes` on the REPL debugger, you will notice Unconstrained VM blocks that look like this:

```
...
5    BRILLIG inputs=[Single(Expression { mul_terms: [], linear_combinations: [], q_c: 2 }), Single(Expression { mul_terms: [], linear_combinations: [(1, Witness(2))], q_c: 0 })]
       |       outputs=[]
  5.0  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(0) }
  5.1  |   Mov { destination: RegisterIndex(3), source: RegisterIndex(1) }
  5.2  |   Const { destination: RegisterIndex(0), value: Value { inner: 0 } }
  5.3  |   Const { destination: RegisterIndex(1), value: Value { inner: 0 } }
  5.4  |   Mov { destination: RegisterIndex(2), source: RegisterIndex(2) }
  5.5  |   Mov { destination: RegisterIndex(3), source: RegisterIndex(3) }
  5.6  |   Call { location: 8 }
  5.7  |   Stop
  5.8  |   ForeignCall { function: "__debug_var_assign", destinations: [], inputs: [RegisterIndex(RegisterIndex(2)), RegisterIndex(RegisterIndex(3))] }
...
```               
                                    
If you are interested in debugging/inspecting compiled ACIR without these synthetic changes, you can invoke the REPL debugger with the `--skip-instrumentation` flag or launch the VS Code debugger with the `skipConfiguration` property set to true in its launch configuration. You can find more details about those in the [Debugger REPL reference](debugger_repl.md) and the [VS Code Debugger reference](debugger_vscode.md).

:::note
Skipping debugger instrumentation means you won't be able to inspect values of local variables.
:::

