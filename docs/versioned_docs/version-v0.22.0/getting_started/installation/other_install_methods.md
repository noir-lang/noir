---
title: Alternative Install Methods
description:
  There are different ways to install Nargo, the one-stop shop and command-line tool for developing Noir programs. This guide explains other methods that don't rely on noirup, such as compiling from source, installing from binaries, and using WSL for windows
keywords: [
   Installation
   Nargo
   Noirup
   Binaries
   Compiling from Source
   WSL for Windows
   macOS
   Linux
   Nix
   Direnv
   Shell & editor experience
   Building and testing
   Uninstalling Nargo
   Noir vs code extension
]
sidebar_position: 1
---


## Installation

The most common method of installing Nargo is through [Noirup](./index.md)

However, there are other methods for installing Nargo:

- [Binaries](#binaries)
- [Compiling from Source](#compile-from-source)
- [WSL for Windows](#wsl-for-windows)

### Binaries

See [GitHub Releases](https://github.com/noir-lang/noir/releases) for the latest and previous
platform specific binaries.

#### Step 1

Paste and run the following in the terminal to extract and install the binary:

> **macOS / Linux:** If you are prompted with `Permission denied` when running commands, prepend
> `sudo` and re-run it.

##### macOS (Apple Silicon)

```bash
mkdir -p $HOME/.nargo/bin && \
curl -o $HOME/.nargo/bin/nargo-aarch64-apple-darwin.tar.gz -L https://github.com/noir-lang/noir/releases/download/v0.6.0/nargo-aarch64-apple-darwin.tar.gz && \
tar -xvf $HOME/.nargo/bin/nargo-aarch64-apple-darwin.tar.gz -C $HOME/.nargo/bin/ && \
echo '\nexport PATH=$PATH:$HOME/.nargo/bin' >> ~/.zshrc && \
source ~/.zshrc
```

##### macOS (Intel)

```bash
mkdir -p $HOME/.nargo/bin && \
curl -o $HOME/.nargo/bin/nargo-x86_64-apple-darwin.tar.gz -L https://github.com/noir-lang/noir/releases/download/v0.6.0/nargo-x86_64-apple-darwin.tar.gz && \
tar -xvf $HOME/.nargo/bin/nargo-x86_64-apple-darwin.tar.gz -C $HOME/.nargo/bin/ && \
echo '\nexport PATH=$PATH:$HOME/.nargo/bin' >> ~/.zshrc && \
source ~/.zshrc
```

##### Linux (Bash)

```bash
mkdir -p $HOME/.nargo/bin && \
curl -o $HOME/.nargo/bin/nargo-x86_64-unknown-linux-gnu.tar.gz -L https://github.com/noir-lang/noir/releases/download/v0.6.0/nargo-x86_64-unknown-linux-gnu.tar.gz && \
tar -xvf $HOME/.nargo/bin/nargo-x86_64-unknown-linux-gnu.tar.gz -C $HOME/.nargo/bin/ && \
echo -e '\nexport PATH=$PATH:$HOME/.nargo/bin' >> ~/.bashrc && \
source ~/.bashrc
```

#### Step 2

Check if the installation was successful by running `nargo --version`. You should get a version number.

> **macOS:** If you are prompted with an OS alert, right-click and open the _nargo_ executable from
> Finder. Close the new terminal popped up and `nargo` should now be accessible.

### Option 3: Compile from Source

Due to the large number of native dependencies, Noir projects uses [Nix](https://nixos.org/) and [direnv](https://direnv.net/) to streamline the development experience. It helps mitigating issues commonly associated with dependency management, such as conflicts between required package versions for different projects (often referred to as "dependency hell").

Combined with direnv, which automatically sets or clears environment variables based on the directory, it further simplifies the development process by seamlessly integrating with the developer's shell, facilitating an efficient and reliable workflow for managing and deploying Noir projects with multiple dependencies.

#### Setting up your environment

For the best experience, please follow these instructions to setup your environment:

1. Install Nix following [their guide](https://nixos.org/download.html) for your operating system.
2. Create the file `~/.config/nix/nix.conf` with the contents:

```ini
experimental-features = nix-command
extra-experimental-features = flakes
```

3. Install direnv into your Nix profile by running:

```sh
nix profile install nixpkgs#direnv
```

4. Add direnv to your shell following [their guide](https://direnv.net/docs/hook.html).
   1. For bash or zshell, add `eval "$(direnv hook bash)"` or `eval "$(direnv hook zsh)"` to your ~/.bashrc or ~/.zshrc file, respectively.
5. Restart your shell.

#### Shell & editor experience

Now that your environment is set up, you can get to work on the project.

1. Clone the repository, such as:

```sh
git clone git@github.com:noir-lang/noir
```

> Replacing `noir` with whichever repository you want to work on.

2. Navigate to the directory:

```sh
cd noir
```

> Replacing `noir` with whichever repository you cloned.

3. You should see a **direnv error** because projects aren't allowed by default. Make sure you've reviewed and trust our `.envrc` file, then you need to run:

```sh
direnv allow
```

4. Now, wait awhile for all the native dependencies to be built. This will take some time and direnv will warn you that it is taking a long time, but we just need to let it run.

5. Once you are presented with your prompt again, you can start your editor within the project directory (we recommend [VSCode](https://code.visualstudio.com/)):

```sh
code .
```

6. (Recommended) When launching VSCode for the first time, you should be prompted to install our recommended plugins. We highly recommend installing these for the best development experience.

#### Building and testing

Assuming you are using `direnv` to populate your environment, building and testing the project can be done
with the typical `cargo build`, `cargo test`, and `cargo clippy` commands. You'll notice that the `cargo` version matches the version we specify in `rust-toolchain.toml`, which is 1.71.1 at the time of this writing.

If you want to build the entire project in an isolated sandbox, you can use Nix commands:

1. `nix build .` (or `nix build . -L` for verbose output) to build the project in a Nix sandbox.
2. `nix flake check` (or `nix flake check -L` for verbose output) to run clippy and tests in a Nix sandbox.

#### Without `direnv`

If you have hesitations with using direnv, you can launch a subshell with `nix develop` and then launch your editor from within the subshell. However, if VSCode was already launched in the project directory, the environment won't be updated.

Advanced: If you aren't using direnv nor launching your editor within the subshell, you can try to install Barretenberg and other global dependencies the package needs. This is an advanced workflow and likely won't receive support!

### Option 4: WSL (for Windows)

The default backend for Noir (Barretenberg) doesn't provide Windows binaries at this time. For that reason, Noir cannot be installed natively. However, it is available by using Windows Subsystem for Linux (WSL).

Step 1: Follow the instructions [here](https://learn.microsoft.com/en-us/windows/wsl/install) to install and run WSL.

step 2: Follow the [Noirup instructions](./index.md).

## Uninstalling Nargo

### Noirup

If you installed Noir with `noirup`, you can uninstall Noir by removing the files in `~/.nargo`, `~/nargo` and `~/noir_cache`.

```bash
rm -r ~/.nargo
rm -r ~/nargo
rm -r ~/noir_cache
```

### Nix

If you installed Noir with Nix or from source, you can remove the binary located at `~/.nix-profile/bin/nargo`.

```bash
rm ~/.nix-profile/bin/nargo
```
