---
title: Dev Containers
description: Learn how to set up and use Noir development environments with Dev Containers in VS Code and GitHub Codespaces.
keywords: [Noir, Dev Container, Codespaces, Docker, VS Code, Development Environment]
sidebar_position: 5
---

This guide explains how to use Dev Containers for Noir development, enabling consistent development environments across different machines and cloud-based development through GitHub Codespaces.

## What are Dev Containers?

Dev Containers provide a full-featured development environment inside a Docker container. They ensure all developers work with the same tools, dependencies, and configurations, eliminating "works on my machine" issues.

For Noir development, Dev Containers offer:

- Pre-installed Noir toolchain (nargo, noirup)
- Pre-installed Barretenberg backend (bb, bbup)
- Consistent development experience across platforms
- Quick onboarding for new contributors

## Using Dev Containers Locally

### Prerequisites

- [Docker](https://www.docker.com/products/docker-desktop/) installed and running
- [Visual Studio Code](https://code.visualstudio.com/) or similar (ex. Cursor, Roo, etc)
- [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) or similar

### Setting up a Noir Dev Container

1. Create a `.devcontainer/devcontainer.json` file in your project root. This will add the [devcontainer feature](https://github.com/AztecProtocol/devcontainer-feature) built by Aztec Labs, and the Noir extension.

```json
{
  "name": "Noir Development",
  "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
  "features": {
    "ghcr.io/aztecprotocol/devcontainer-features/noir:1": {},
    "ghcr.io/aztecprotocol/devcontainer-features/barretenberg:1": {}
  },
  "customizations": {
    "vscode": {
      "extensions": [
        "noir-lang.vscode-noir"
      ]
    }
  }
}
```

2. Open your project in your IDE

3. When prompted, click "Reopen in Container" or use the Command Palette (`Ctrl/Cmd + Shift + P`) and select "Dev Containers: Reopen in Container"

4. The IDE will build the container and set up your development environment with Noir and Barretenberg pre-installed

## Using GitHub Codespaces

GitHub Codespaces provides cloud-hosted Dev Container environments, allowing you to develop Noir projects directly in your browser or in VS Code.

### Quick Start with Codespaces

The easiest way to get started is using the [Tiny Noir Codespace](https://github.com/aztecprotocol/tiny-noir-codespace) template:

1. Visit the [repository](https://github.com/aztecprotocol/tiny-noir-codespace)
2. Click the "Code" button and select "Create codespace on main"
3. GitHub will create a cloud-based development environment with Noir pre-configured

### Adding Codespaces to Your Project

To enable Codespaces for your own Noir project, use the same `.devcontainer/devcontainer.json` configuration shown above. When contributors visit your repository, they can create a Codespace with all necessary tools pre-installed.

## Available Devcontainer Features

The [AztecProtocol devcontainer features](https://github.com/AztecProtocol/devcontainer-feature) provide:

### Noir Feature
- Installs `nargo` CLI for compiling and testing Noir programs
- Installs `noirup` for managing Noir versions
- Automatically configures the latest stable version

### Barretenberg Feature
- Installs `bb` CLI for proving and verification
- Installs `bbup` for managing Barretenberg versions
- Provides backend support for Noir programs

## Configuration Options

You can customize the Dev Container setup by modifying the features in `devcontainer.json`:

```json
{
  "features": {
    "ghcr.io/aztecprotocol/devcontainer-features/noir:1": {
      // Feature-specific options (if available)
    },
    "ghcr.io/aztecprotocol/devcontainer-features/barretenberg:1": {
      // Feature-specific options (if available)
    }
  }
}
```

## Tips for Development

1. **Free Tier**: GitHub Codespaces offers up to 60 hours free per month for personal accounts

2. **Performance**: For resource-intensive operations like proving, local Dev Containers may perform better than Codespaces

3. **Persistence**: Changes made in a Dev Container are preserved in the container. Use version control to save your work

4. **Extensions**: The Noir VS Code extension is automatically installed, providing syntax highlighting, LSP support, and debugging capabilities

## Next Steps

- Set up a Dev Container for your Noir project
- Explore the [Noir VS Code extension](./language_server.md) features
- Learn about [debugging Noir programs](./debugger.mdx) in your containerized environment