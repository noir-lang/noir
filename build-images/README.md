# Build Image

To ensure a consistent environment for developers, and ease of getting started, we provide a development container.

## Install Docker

If you don't already have docker installed, follow this guide: https://docs.docker.com/engine/install

## Visual Studio Code

If you use vscode, the simplest thing to do is install the "Dev Containers" plugin, and open the repo.
You'll be prompted to reload in a dev container, at which point you can open a terminal and bootstrap.
You can connect to your container from outside vscode with e.g.: `docker exec -ti <container_id> /bin/zsh`

Your repo will be mounted at `/workspaces/aztec-packages`, and your home directory is persisted in a docker volume.

## Running Independently

If you don't use vscode, you can simply run `./run.sh` to create and drop into the container.

Your repo will be mounted at `/workspaces/aztec-packages`, and your home directory is persisted in a docker volume.

## GitHub Codespaces

This is also compatible with GitHub codespaces. Visit the repo at `http://github.com/aztecprotocol/aztec-packages`.
Press `.`, and open a terminal window. You will be prompted to create a new machine.
You can then continue to work within the browser, or reopen the codespace in your local vscode.
