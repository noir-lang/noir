# Noir Debugger

There are currently two ways of debugging Noir programs, both in active development and in experimental phase:

1. The REPL debugger, which currently ships with Nargo behind a feature flag.
2. The VS Code extension, which hasn't still reached minimum viability, and so must be manually set up.

This README explains how to use each of them as well as specifying which features are currently mature and which ones are unstable.

## Supported project types

At the time of writing, the debugger supports debugging binary projects, but not contracts. At the end of this README, we'll elaborate on what the current state of Noir contract debugging is, and the pre-requisites to fulfil.


## REPL debugger

In order to use the REPL debugger, you will need to install a new enough version of Nargo. At the time of writing, the nightly version is 0.20.0, so we'll base this guide on it.

Let's debug a simple circuit:

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
  registerers                        show Brillig registerers (valid when executing
                                   a Brillig block)
  regset index:usize value:String  update a Brillig registerer with the given
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

The command menu is pretty self-explanatory. Some commands operate only at Brillig level, such as `memory`, `memset`, `registerers`, `regset`. If you try to use them while execution is paused at an ACIR opcode, the debugger will simply inform you that you are not executing Brillig code:

```
> registerers
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

Note: in future versions of the debugger, we could explore prettier or more compact formats to print opcodes.

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

In disassembly view:

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
> witness
_1 = 1
_2 = 2
_3 = 1
> 
```

We can see two arrow `->` cursors: one indicates where we are from the perspective of ACIR (opcode 1), and the other one shows us where we are in the context of the current Brillig block (opcode 1.0).

Note: REPL commands are autocompleted when not ambiguous, so `opcodes` can be run just with `op`, `into` with `i`, etc.

The next opcode to execute is a `JumpIfNot`, which reads from registerer 0. Let's inspect Brillig registerer state:

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
> registerers
Brillig VM registerers not available
```

Oops. This is unexpected, even though we were already in a Brillig block, we couldn't access the Brillig registerers. This is a known issue: when just entering the Brillig block, the ACVM has not yet initialized the Brillig VM, so we can't introspect it.

Note: In order to solve this, we would have to change the way the ACVM works, or add special handling for this case (after all, the debugger does know we're at the first opcode of a Brillig block and could keep track of how registerers will be initialized). At the time of writing, we haven't yet solved this case. 

For now, let's just step once more:

```
> i
At opcode 1.1: Const { destination: RegisterIndex(1), value: Value { inner: 1 } }
> registerers
0 = 1
>
```

Now we can see that registerer 0 was initialized with a value of 1, so the `JumpIfNot` didn't activate. After executing opcode 1, we should see registerer 1 gets a value of 1:

```
> i
At opcode 1.2: BinaryFieldOp { destination: RegisterIndex(0), op: Div, lhs: RegisterIndex(1), rhs: RegisterIndex(0) }
> register
0 = 1
1 = 1
>
```

The last operation will compute `Reg0 <- Reg1 / Reg0`: 

```
> i
At opcode 1.3: Stop
> registerers
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


# Testing experimental features

There's a number of features that are in active development and that can't yet be merged to the main branch for different reasons. In this section we detail what those features are and how to try them out.

## Build from experimental branch at fork

Build Nargo by pulling the source version from https://github.com/manastech/noir/tree/dap-with-vars.

This will result in a Nargo binary being written to `PROJECT_ROOT/target/debug/nargo`. We will use this path later, so keep it at hand or export it to a an env var. For example:

`export NARGO_EXP=PROJECT_ROOT/target/debug/nargo`

## About the experimental features

There are currently 2 experimental features in the debugger:

- Variables inspection
- Stacktrace inspection

NOTE: Supporting variables inspection requires extensive instrumentation of the compiler, handling all cases of variable creation, types, and value assignment. At the time of writing this README, some cases are still not supported. For example, if your program uses slices or references, this compiler version might panic when trying to compile them, or at some point during the debugger step-by-step execution. This is the main reason why this feature has not yet been merged into master. 

## Trying out REPL experimental features

To try out these features, go through the same steps as described at the REPL Debugger section above, but instead of using `nargo debug` use `$NARGO_EXP debug` (assuming you exported your custom built Nargo binary to NARGO_EXP). 

When entering `help` on this version, you'll find two new commands:

```
...
stacktrace                       display the current stack trace
...
vars                             show variable values available at this point
                                   in execution
```

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

## Testing the VS Code extension (experimental)

There is a fork of the official Noir Visual Studio extension which enables the debugger in VS Code. This fork is at: https://github.com/manastech/vscode-noir/tree/dap-support.

In this section, we'll explain how to test the VS Code Noir debugger combining that extension fork with the experimental features branch discussed above.

1. First, get a copy of the extension source code from https://github.com/manastech/vscode-noir/tree/dap-support.

2. Package the extension by running `npm run package`.

3. Open the root folder of the extension on VS Code.

4. From VS Code, press fn+F5. This will open a new VS Code window with the extension loaded from source. 

5. Go to Code -> Settings -> Extensions -> Noir Language Server. Look for the property `Nargo Path` and enter the path to the experimental build you got as a result of following the steps at [Trying out REPL experimental features](#trying-out-repl-experimental-features).

6. At the VS Code sidebar, go to the debugger section (see screenshot). Click "Add configuration". Overwrite the `projectFolder` property with the absolute path to the Nargo project you want to debug.

<img width="473" alt="Screenshot 2023-12-18 at 14 37 38" src="https://github.com/manastech/noir/assets/651693/cdad9ee1-8164-4c33-ab24-2584016088f0">

7. Go to a Noir file you want to debug. Navigate again to the debug section of VS Code, and click the "play" icon.

The debugger should now have started. Current features exposed to the debugger include different kinds of stepping interactions, variable inspection and stacktraces. At the time of writing, Brillig registerers and memory are not being exposed, but they will soon be.  

![Screen Recording 2023-12-18 at 14 14 28](https://github.com/manastech/noir/assets/651693/36b4becb-953a-4158-9c5a-7a185673f54f)

## Towards debugging contracts

### Contracts Runtime

The execution of Noir contracts depends on a foreign call execution runtime to resolve all the oracle calls that the contract functions depend on.

This means for the debugger to be usable with contracts we need to be able to do at least one of the following:

1. Let users mock out a foreign call executor, and run the debugger with it.
2. Instrument live environments, such as the Sandbox, so that calls and transactions can be driven by the debugger, which ultimately means the debugger would use the same foreign call executor a live Sandbox uses for normal execution of Noir circuits.

Both of these scenarios imply making the debugger available to language runtimes external to Noir. The Sandbox/PXE runs on JS runtimes, and an hypothetical mockable foreign executor could be in principle written in any language. So it seems the most promising way forward is to make sure that the debugger itself is consumable in JS runtimes.

