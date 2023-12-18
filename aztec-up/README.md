# The Aztec Installation Script

```
bash -i <(curl -s install.aztec.network)
```

That is all.

This will install into `~/.aztec/bin` a collection of scripts to help running aztec containers, and will update
a users `PATH` variable in their shell startup script so they can be found.

- `aztec` - The infrastructure container.
- `aztec-cli` - A command line tool for interacting with infrastructure.
- `aztec-nargo` - A build of `nargo` from `noir` that is guaranteed to be version aligned. Provides compiler, lsp and more.
- `aztec-sandbox` - A wrapper around docker-compose that launches services needed for sandbox testing.
- `aztec-up` - A tool to upgrade the aztec toolchain to the latest, or specific versions.

After installed, you can use `aztec-up` to upgrade or install specific versions.

```
VERSION=master aztec-up
```

This will install the container built from master branch.

```
VERSION=v1.2.3 aztec-up
```

This will install tagged release version 1.2.3.
