---
title: Tooling
Description: This section provides information about the various tools and utilities available for Noir development. It covers IDE tools, Codespaces, and community projects.
Keywords: [Noir, Development, Playground, IDE Tools, Language Service Provider, VS Code Extension, Codespaces, noir-starter, Community Projects, Awesome Noir Repository, Developer Tooling]
---

Noir is meant to be easy to develop with. For that reason, a number of utilities have been put together to ease the development process as much as feasible in the zero-knowledge world.

## IDE tools

When you install Nargo, you're also installing a Language Service Provider (LSP), which can be used by IDEs to provide syntax highlighting, codelens, warnings, and more.

The easiest way to use these tools is by installing the [Noir VS Code extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir).

## Codespaces

Some Noir repos have leveraged Codespaces in order to ease the development process. You can visit the [noir-starter](https://github.com/noir-lang/noir-starter) for an example.

## GitHub Actions

You can use `noirup` with GitHub Actions for CI/CD and automated testing. It is as simple as
installing `noirup` and running tests in your GitHub Action `yml` file.

See the
[config file in the Noir repo](https://github.com/TomAFrench/noir-hashes/blob/master/.github/workflows/noir.yml) for an example usage.

## Community projects

As an open-source project, Noir has received many contributions over time. Some of them are related with developer tooling, and you can see some of them in [Awesome Noir repository](https://github.com/noir-lang/awesome-noir#dev-tools)
