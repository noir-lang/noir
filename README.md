# The Noir Programming Language

Noir is a Domain Specific Language for SNARK proving systems. It has been designed to use any ACIR compatible proving system.

**This implementation is in early development. It has not been reviewed or audited. It is not suitable to be used in production. Expect bugs!**

## Quick Start

Read the installation section [here](https://noir-lang.org/getting_started/nargo_installation).

Once you have read through the documentation, you can visit [Awesome Noir](https://github.com/noir-lang/awesome-noir) to run some of the examples that others have created.

## Current Features

Backends:

- Barretenberg via FFI
- Marlin via arkworks

Compiler:

- Module System
- For expressions
- Arrays
- Bit Operations
- Binary operations (<, <=, >, >=, +, -, \*, /, %) [See documentation for an extensive list]
- Unsigned integers
- If statements
- Structures and Tuples
- Generics

ACIR Supported OPCODES:

- Sha256
- Blake2s
- Schnorr signature verification
- MerkleMembership
- Pedersen
- HashToField

## Future Work

The current focus is to gather as much feedback as possible while in the alpha phase. The main focuses of Noir are _safety_ and _developer experience_. If you find a feature that does not seem to be in line with these goals, please open an issue!

Concretely the following items are on the road map:

- General code sanitization and documentation (ongoing effort)
- Prover and Verifier Key logic. (Prover and Verifier pre-process per compile)
- Fallback mechanism for backend unsupported opcodes
- Visibility modifiers
- Signed integers
- Backend integration: (Bulletproofs)
- Recursion
- Big integers

## Nargo CLI - pre-built

`nargo` - command line interface tool for interacting with Noir programs - allows compiling, proving, verifying, and more. Nightly binary builds can be found [here](https://github.com/noir-lang/noir/releases/tag/nightly). Please refer [noir-lang/build-nargo](https://github.com/noir-lang/build-nargo) to inspect how these are built for various platforms.

## Nargo CLI - install scripts

[noir-lang/noirup](https://github.com/noir-lang/noirup) repository contains install scripts for Linux, macOS, and Windows systems to allow easy installation.

## Minimum Rust version

This crate's minimum supported rustc version is 1.66.0.

## Working on this project

Due to the large number of native dependencies, this project uses [Nix](https://nixos.org/) and [direnv](https://direnv.net/) to streamline the development experience.

### Setting up your environment

For the best experience, please follow these instructions to setup your environment:
1. Install Nix following [their guide](https://nixos.org/download.html) for your operating system
2. Create the file `~/.config/nix/nix.conf` with the contents:
```ini
experimental-features = nix-command
extra-experimental-features = flakes
```
3. Install direnv into your Nix profile by running:
```sh
nix profile install nixpkgs#direnv
```
4. Add direnv to your shell following [their guide](https://direnv.net/docs/hook.html)
5. Restart your shell

### Shell & editor experience

Now that your environment is set up, you can get to work on the project.

1. Clone the repository, such as:
```sh
git clone git@github.com:noir-lang/noir
```
2. Navigate to the directory:
```sh
cd noir
```
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

### Building and testing

Assuming you are using `direnv` to populate your environment, building and testing the project can be done
with the typical `cargo build`, `cargo test`, and `cargo clippy` commands. You'll notice that the `cargo` version matches the version we specify in [flake.nix](./flake.nix), which is 1.66.0 at the time of this writing.

If you want to build the entire project in an isolated sandbox, you can use Nix commands:
1. `nix build .` (or `nix build . -L` for verbose output) to build the project in a Nix sandbox
2. `nix flake check` (or `nix flake check -L` for verbose output) to run clippy and tests in a Nix sandbox

### Building against a different local/remote version of Barretenberg

If you are working on this project and want a different version of Barretenberg (instead of the version this project is pinned against), you'll want to replace the lockfile version with your version. This can be done by running:

```sh
nix flake lock --override-input barretenberg /absolute/path/to/your/barretenberg
```

You can also point at a fork and/or branch on GitHub using:

```sh
nix flake lock --override-input barretenberg github:username/barretenberg/branch_name
```

__Note:__ You don't want to commit the updated lockfile, as it will fail in CI!

### Without direnv

If you have hesitations with using `direnv`, you can launch a subshell with `nix develop` and then launch your editor
from within the subshell. However, if VSCode was already launched in the project directory, the environment won't be updated.

__Advanced:__ If you aren't using `direnv` nor launching your editor within the subshell, you can try to install Barretenberg and other global dependencies the package needs. This is an advanced workflow and likely won't receive support!

## License

Noir is free and open source. It is distributed under a dual license. (MIT/APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
