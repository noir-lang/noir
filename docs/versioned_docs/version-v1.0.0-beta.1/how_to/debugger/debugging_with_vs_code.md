---
title: Using the VS Code Debugger
description:
  Step-by-step guide on how to debug your Noir circuits with the VS Code Debugger configuration and features.
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

#### Pre-requisites

- Nargo
- vscode-noir
- A Noir project with a `Nargo.toml`, `Prover.toml` and at least one Noir (`.nr`) containing an entry point function (typically `main`).

## Running the debugger

The easiest way to start debugging is to open the file you want to debug, and press `F5`. This will cause the debugger to launch, using your `Prover.toml` file as input.

You should see something like this:

![Debugger launched](@site/static/img/debugger/1-started.png)

Let's inspect the state of the program. For that, we open VS Code's _Debug pane_. Look for this icon:

![Debug pane icon](@site/static/img/debugger/2-icon.png)

You will now see two categories of variables: Locals and Witness Map.

![Debug pane expanded](@site/static/img/debugger/3-debug-pane.png)

1. **Locals**: variables of your program. At this point in execution this section is empty, but as we step through the code it will get populated by `x`, `result`, `digest`, etc. 

2. **Witness map**: these are initially populated from your project's `Prover.toml` file. In this example, they will be used to populate `x` and `result` at the beginning of the `main` function.

Most of the time you will probably be focusing mostly on locals, as they represent the high level state of your program. 

You might be interested in inspecting the witness map in case you are trying to solve a really low level issue in the compiler or runtime itself, so this concerns mostly advanced or niche users.

Let's step through the program, by using the debugger buttons or their corresponding keyboard shortcuts.

![Debugger buttons](@site/static/img/debugger/4-debugger-buttons.png)

Now we can see in the variables pane that there's values for `digest`, `result` and `x`.

![Inspecting locals](@site/static/img/debugger/5-assert.png)

We can also inspect the values of variables by directly hovering on them on the code.

![Hover locals](@site/static/img/debugger/6-hover.png)

Let's set a break point at the `keccak256` function, so we can continue execution up to the point when it's first invoked without having to go one step at a time. 

We just need to click the to the right of the line number 18. Once the breakpoint appears, we can click the `continue` button or use its corresponding keyboard shortcut (`F5` by default).

![Breakpoint](@site/static/img/debugger/7-break.png)

Now we are debugging the `keccak256` function, notice the _Call Stack pane_ at the lower right. This lets us inspect the current call stack of our process.

That covers most of the current debugger functionalities. Check out [the reference](../../reference/debugger/debugger_vscode.md) for more details on how to configure the debugger.
