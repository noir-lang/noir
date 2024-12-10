---
title: Workspaces
sidebar_position: 3
---

Workspaces are a feature of nargo that allow you to manage multiple related Noir packages in a single repository. A workspace is essentially a group of related projects that share common build output directories and configurations.

Each Noir project (with it's own Nargo.toml file) can be thought of as a package. Each package is expected to contain exactly one "named circuit", being the "name" defined in Nargo.toml with the program logic defined in `./src/main.nr`.

For a project with the following structure:

```tree
├── crates
│   ├── a
│   │   ├── Nargo.toml
│   │   └── Prover.toml
│   │   └── src
│   │       └── main.nr
│   └── b
│       ├── Nargo.toml
│       └── Prover.toml
│       └── src
│           └── main.nr
│
└── Nargo.toml
```

You can define a workspace in Nargo.toml like so:

```toml
[workspace]
members = ["crates/a", "crates/b"]
default-member = "crates/a"
```

`members` indicates which packages are included in the workspace. As such, all member packages of a workspace will be processed when the `--workspace` flag is used with various commands or if a `default-member` is not specified. The `--package` option can be used to limit
the scope of some commands to a specific member of the workspace; otherwise these commands run on the package nearest on the path to the
current directory where `nargo` was invoked.

`default-member` indicates which package various commands process by default.

Libraries can be defined in a workspace. Inside a workspace, these are consumed as `{ path = "../to_lib" }` dependencies in Nargo.toml.

Inside a workspace, these are consumed as `{ path = "../to_lib" }` dependencies in Nargo.toml.

Please note that nesting regular packages is not supported: certain commands work on the workspace level and will use the topmost Nargo.toml file they can find on the path; unless this is a workspace configuration with `members`, the command might run on some unintended package.
