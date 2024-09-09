---
title: Nargo Installation
description:
  nargo is a command line tool for interacting with Noir programs. This page is a quick guide on how to install Nargo through the most common and easy method, noirup
keywords: [
   Nargo
   Noir
   Rust
   Cargo
   Noirup
   Installation
   Terminal Commands
   Version Check
   Nightlies
   Specific Versions
   Branches
   Noirup Repository
]
pagination_next: getting_started/hello_noir/index
---

`nargo` is a tool for working with Noir programs on the CLI, providing you with the ability to start new projects, compile, execute and test Noir programs from the terminal.

The name is inspired by Rust's package manager `cargo`; and similar to Rust's `rustup`, Noir also has an easy installation script `noirup`.

## Installing Noirup

Open a terminal on your machine, and write:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
```

Close the terminal, open another one, and run

```bash
noirup
```

Done. That's it. You should have the latest version working. You can check with `nargo --version`.

You can also install nightlies, specific versions
or branches. Check out the [noirup repository](https://github.com/noir-lang/noirup) for more
information.

Now we're ready to start working on [our first Noir program!](../hello_noir/index.md)
