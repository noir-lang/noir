---
title: Language Server
description: Learn about the Noir Language Server, how to install the components, and configuration that may be required.
keywords: [Nargo, Language Server, LSP, VSCode, Visual Studio Code]
sidebar_position: 0
---

This section helps you install and configure the Noir Language Server.

The Language Server Protocol (LSP) has two components, the [Server](#language-server) and the [Client](#language-client). Below we describe each in the context of Noir.

## Language Server

The Server component is provided by the Nargo command line tool that you installed at the beginning of this guide.
As long as Nargo is installed and you've used it to run other commands in this guide, it should be good to go!

If you'd like to verify that the `nargo lsp` command is available, you can run `nargo --help` and look for `lsp` in the list of commands. If you see it, you're using a version of Noir with LSP support.

## Language Client

The Client component is usually an editor plugin that launches the Server. It communicates LSP messages between the editor and the Server. For example, when you save a file, the Client will alert the Server, so it can try to compile the project and report any errors.

Currently, Noir provides a Language Client for Visual Studio Code via the [vscode-noir](https://github.com/noir-lang/vscode-noir) extension. You can install it via the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir).

> **Note:** Noir's Language Server Protocol support currently assumes users' VSCode workspace root to be the same as users' Noir project root (i.e. where Nargo.toml lies).
>
> If LSP features seem to be missing / malfunctioning, make sure you are opening your Noir project directly (instead of as a sub-folder) in your VSCode instance.

When your language server is running correctly and the VSCode plugin is installed, you should see handy codelens buttons for compilation, measuring circuit size, execution, and tests:

![Compile and Execute](@site/static/img/codelens_compile_execute.png)
![Run test](@site/static/img/codelens_run_test.png)

You should also see your tests in the `testing` panel:

![Testing panel](@site/static/img/codelens_testing_panel.png)

### Configuration

- **Noir: Enable LSP** - If checked, the extension will launch the Language Server via `nargo lsp` and communicate with it.
- **Noir: Nargo Flags** - Additional flags may be specified if you require them to be added when the extension calls `nargo lsp`.
- **Noir: Nargo Path** - An absolute path to a Nargo binary with the `lsp` command. This may be useful if Nargo is not within the `PATH` of your editor.
- **Noir > Trace: Server** - Setting this to `"messages"` or `"verbose"` will log LSP messages between the Client and Server. Useful for debugging.
