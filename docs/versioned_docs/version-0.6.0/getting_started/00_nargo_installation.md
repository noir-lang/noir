---
title: Nargo Installation
description:
  nargo is a command line tool for interacting with Noir programs (e.g. compiling, proving,
  verifying and more). Learn how to install and use Nargo for your projects with this comprehensive
  guide.
keywords: [Nargo, command line tool, Noir programs, installation guide, how to use Nargo]
---

`nargo` is a command line tool for interacting with Noir programs (e.g. compiling, proving,
verifying and more).

Alternatively, the interactions can also be performed in [TypeScript](../typescript).

### UltraPlonk

Nargo versions <0.5.0 of `aztec_backend` and `aztec_wasm_backend` are based on the TurboPlonk
version of Aztec Backend, which lacks efficient implementations of useful primitives (e.g. Keccak256 in 18k constraints, ECDSA verification in 36k constraints) that the UltraPlonk version offers.

## Installation

There are four approaches for installing Nargo:

- [Option 1: Noirup](#option-1-noirup)
- [Option 2: Binaries](#option-2-binaries)
- [Option 3: Install via Nix](#option-3-install-via-nix)
- [Option 4: Compile from Source](#option-4-compile-from-source)

Optionally you can also install [Noir VS Code extension] for syntax highlighting.

### Option 1: Noirup

If you're on OSX or Linux, the easiest way to start using Noir and Nargo is via noirup. Just open a
terminal and run:

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
```

Close the terminal, open another one, and run

```bash
noirup -v 0.6.0
```

Done, you should have the latest version working. You can check with `nargo --version`.

You can also install nightlies, specific versions
or branches, check out the [noirup repository](https://github.com/noir-lang/noirup) for more
information.

#### GitHub Actions

You can use `noirup` with GitHub Actions for CI/CD and automated testing. It is as simple as
installing `noirup` and running tests in your GitHub Action `yml` file.

See the
[config file](https://github.com/TomAFrench/noir-hashes/blob/master/.github/workflows/noir.yml) in
this repo containing hash functions in Noir for an example.

#### Nightly versions

To install the nightly version of Noir (updated daily) run:

```bash
noirup -n
```

### Option 2: Binaries

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

##### Windows (PowerShell)

Open PowerShell as Administrator and run:

```powershell
mkdir -f -p "$env:USERPROFILE\.nargo\bin\"; `
Invoke-RestMethod -Method Get -Uri https://github.com/noir-lang/noir/releases/download/v0.4.1/nargo-x86_64-pc-windows-msvc.zip -Outfile "$env:USERPROFILE\.nargo\bin\nargo-x86_64-pc-windows-msvc.zip"; `
Expand-Archive -Path "$env:USERPROFILE\.nargo\bin\nargo-x86_64-pc-windows-msvc.zip" -DestinationPath "$env:USERPROFILE\.nargo\bin\"; `
$Reg = "Registry::HKLM\System\CurrentControlSet\Control\Session Manager\Environment"; `
$OldPath = (Get-ItemProperty -Path "$Reg" -Name PATH).Path; `
$NewPath = $OldPath + ’;’ + "$env:USERPROFILE\.nargo\bin\"; `
Set-ItemProperty -Path "$Reg" -Name PATH –Value "$NewPath"; `
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
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

Check if the installation was successful by running `nargo --help`.

> **macOS:** If you are prompted with an OS alert, right-click and open the _nargo_ executable from
> Finder. Close the new terminal popped up and `nargo` should now be accessible.

For a successful installation, you should see something similar to the following after running the
command:

```sh
$ nargo --help

Noir's package manager

Usage: nargo <COMMAND>

Commands:
   check             Checks the constraint system for errors
   codegen-verifier  Generates a Solidity verifier smart contract for the program
   compile           Compile the program and its secret execution trace into ACIR format
   new               Create a new binary project
   execute           Executes a circuit to calculate its return value
   prove             Create proof for this program. The proof is returned as a hex encoded string
   verify            Given a proof and a program, verify whether the proof is valid
   test              Run the tests for this program
   gates             Counts the occurrences of different gates in circuit
   help              Print this message or the help of the given subcommand(s)
```

### Option 3: Install via Nix

Due to the large number of native dependencies, Noir projects can be installed via [Nix](https://nixos.org/).

#### Installing Nix

For the best experience, please follow these instructions to setup Nix:

1. Install Nix following [their guide](https://nixos.org/download.html) for your operating system.
2. Create the file `~/.config/nix/nix.conf` with the contents:

```ini
experimental-features = nix-command
extra-experimental-features = flakes
```

#### Install Nargo into your Nix profile

1. Use `nix profile` to install Nargo

```sh
nix profile install github:noir-lang/noir
```

### Option 4: Compile from Source

Due to the large number of native dependencies, Noir projects uses [Nix](https://nixos.org/) and [direnv](https://direnv.net/) to streamline the development experience.

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

3. You should see a __direnv error__ because projects aren't allowed by default. Make sure you've reviewed and trust our `.envrc` file, then you need to run:

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
with the typical `cargo build`, `cargo test`, and `cargo clippy` commands. You'll notice that the `cargo` version matches the version we specify in `flake.nix`, which is 1.66.0 at the time of this writing.

If you want to build the entire project in an isolated sandbox, you can use Nix commands:

1. `nix build .` (or `nix build . -L` for verbose output) to build the project in a Nix sandbox.
2. `nix flake check` (or `nix flake check -L` for verbose output) to run clippy and tests in a Nix sandbox.

#### Without `direnv`

If you have hesitations with using direnv, you can launch a subshell with `nix develop` and then launch your editor from within the subshell. However, if VSCode was already launched in the project directory, the environment won't be updated.

Advanced: If you aren't using direnv nor launching your editor within the subshell, you can try to install Barretenberg and other global dependencies the package needs. This is an advanced workflow and likely won't receive support!

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

[git]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
[rust]: https://www.rust-lang.org/tools/install
[noir vs code extension]:
  https://marketplace.visualstudio.com/items?itemName=noir-lang.noir-programming-language-syntax-highlighter
[homebrew]: https://brew.sh/
[cmake]: https://cmake.org/install/
[llvm]: https://llvm.org/docs/GettingStarted.html
[openmp]: https://openmp.llvm.org/
[barretenberg]: https://github.com/AztecProtocol/barretenberg
