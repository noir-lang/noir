---
title: Using the VS Code Debugger
description:
  Step by step guide on how to debug your Noir circuits with the VS Code Debugger configuration and features.
keywords:
  [
    Nargo,
    Noir CLI,
    Noir Debugger,
    VS Code,
    IDE,
  ]
sidebar_position: 0
---

This guide will show you how to use VS Code with the vscode-noir extension to debug a Noir project. 

Pre-requisites

- Noir > x
- Nargo > x
- vscode-noir > x
- A Noir project with a Nargo.toml, Prover.toml and at least one Noir (`.nr`) containing an entry point function (typically `main`).

Once you have installed Nargo and vscode-noir, you should have everything you need to debug your project. 

The easiest way to start debugging is to open the file you want to debug, and press `F5`. This will cause the debugger to launch, using your `Prover.toml` file as input.

You should see something like this:

1-started

Let's inspect the state of the program. For that, we open VS Code's _Debug pane_. Look for this icon:

2-icon

You will now see three categories of variables: Locals, Witness Map and Brillig Registers. 

3-debug-pane

1. Locals: variables of your program. At this point in execution this section is empty, but as we step through the code it will get populated by `x`, `result`, `digest`, etc. 

2. Witness map: these are initially populated from your project's Prover.toml file. In this example, they will be used to populate `x` and `result` at the beginning of the `main` function.

3. Brillig registers: these show the current state of the BrilligVM registers. 

Most of the time you will probably be focusing mostly on locals, as they represent the high level state of your program. 

You might be interested in inspecting the witness map and Brillig registers in case you are trying to solve a really low level issue in the compiler or runtime itself, so these concern mostly advanced users.

Let's step through the program, by using the debugger buttons or their corresponding keyboard shortcuts.

4-debugger-buttons

Now we can see in the variables pane that there's values for `digest`, `result` and `x`.

5-assert

We can also inspect the values of variables by directly hovering on them on the code.

6-hover

Let's set a break point at the `keccak256` function, so we can continue execution up to the point when it's first invoked without having to go one step at a time. We just need to click the to the right of the line number 18. Once the breakpoint appear, we can click continue or its keyboard shortcut (`F5` by default).

7-break

Now we are debugging the `keccak256` function, notice the _Call Stack pane_ at the lower right. This lets us inspect the current call stack of our process.

That covers most of the current debugger functionalities. Check out [Noir's VS Code debugger reference](../TODO LINK) for more details on how to configure the debugger.