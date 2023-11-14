---
title: Workspaces
---

Workspaces are a feature of nargo that allow you to manage multiple related Noir packages in a single repository. A workspace is essentially a group of related projects that share common build output directories and configurations.

Each Noir project (with it's own Nargo.toml file) can be thought of as a package. Each package is expected to contain exactly one "named circuit", being the "name" defined in Nargo.toml with the program logic defined in `./src/main.nr`.

For a project with the following structure:

```tree
├── crates
│   ├── a
│   │   ├── Nargo.toml
│   │   └── src
│   │       └── main.nr
│   └── b
│       ├── Nargo.toml
│       └── src
│           └── main.nr
├── Nargo.toml
└── Prover.toml
```

You can define a workspace in Nargo.toml like so:

```toml
[workspace]
members = ["crates/a", "crates/b"]
default-member = "crates/a"
```

`members` indicates which packages are included in the workspace. As such, all member packages of a workspace will be processed when the `--workspace` flag is used with various commands or if a `default-member` is not specified.

`default-member` indicates which package various commands process by default.

Libraries can be defined in a workspace. We just don't have a way to consume libraries from inside a workspace as external dependencies right now.

Inside a workspace, these are consumed as `{ path = "../to_lib" }` dependencies in Nargo.toml.
