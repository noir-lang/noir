# Build Image

To ensure a consistent environment for developers, and ease of getting started, we provide a development container.

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

## Building the build image

If for some reason you want to build the images such as devbox yourself, follow these steps:

### Install Docker

If you don't already have docker installed, follow this guide: https://docs.docker.com/engine/install

### Install earthly

We use earthly to build things, follow this guide: https://earthly.dev/get-earthly

### Build The Dev Container

If you want to build entirely from scratch, you can do:

```
$ earthly +devbox
```

This will take significant time and compute however, as it builds several toolchains from the ground up.
If you have a reasonable internet connection, leveraging the cache to avoid building maybe preferable.

```
$ earthly --use-inline-cache +devbox
```

### Building the sysbox

The sysbox is the image that internal aztec engineers run on the mainframe. A mainframe administrator can run:

```
$ earthly +sysbox
```

This will rebuild the sysbox image, and once users perform a `sudo halt` their box should reboot with the new image.
If the image tag changes, you'll need to update it in `/usr/local/bin/launch_sysbox`.
