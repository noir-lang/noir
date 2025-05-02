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

In order to use either version of the debugger, you will need to install recent enough versions of Noir, [Nargo](../getting_started/noir_installation.md) and vscode-noir:

- Noir & Nargo ≥0.28.0
- Noir's VS Code extension ≥0.0.11

:::info
At the moment, the debugger supports debugging binary projects, but not contracts.
:::

We cover the VS Code Noir debugger more in depth in [its VS Code debugger how-to guide](../how_to/debugger/debugging_with_vs_code.md) and [the reference](../reference/debugger/debugger_vscode.md).

The REPL debugger is discussed at length in [the REPL debugger how-to guide](../how_to/debugger/debugging_with_the_repl.md) and [the reference](../reference/debugger/debugger_repl.md).
