# Noir contracts

This package contains the source code and the Aztec ABIs for the example contracts used in tests.

## Disclaimer

Please note that any example contract set out herein is provided solely for informational purposes only and does not constitute any inducement to use or deploy. Any implementation of any such contract with an interface or any other infrastructure should be used in accordance with applicable laws and regulations.

## Setup

### Installing Noir

- Install [noirup](https://github.com/noir-lang/noirup)
  ```
  curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
  ```
- Nix is already globally installed but path to this installation needs to be added in $HOME/.zshenv so correct configuration can be found by VSCode, do so with:
  ```
  echo -e '\n# Nix path set globally\nexport PATH="$HOME/.nix-profile/bin:/nix/var/nix/profiles/default/bin:$PATH"' >> $HOME/.zshenv
  ```
- Enable nix flake command in ~/.config/nix/nix.conf with commands:
  ```
  mkdir -p $HOME/.config/nix && echo -e '\nexperimental-features = nix-command\nextra-experimental-features = flakes' >> $HOME/.config/nix/nix.conf
  ```
- Install direnv into your Nix profile by running:
  ```
  nix profile install nixpkgs#direnv
  ```
- Add direnv to your shell following their guide
  ```
  echo -e '\n# Adds direnv initialization\neval "$(direnv hook zsh)"' >> $HOME/.zshenv
  ```
- VSCode needs to be resterted so direnv plugin can notice env changes with:
  ```
  kill -9 ps aux | grep $(whoami)/.vscode-server | awk '{print $2}'
  ```
- Restart shell

- Clone noir repo:

  ```
  git clone https://github.com/noir-lang/noir.git
  ```

- Checkout aztec3 branch

  ```
  cd noir
  git checkout aztec3
  ```

- Enable direnv

  ```
  direnv allow
  ```

- Restart shell

- Go to the noir dir and install Noir:
  ```
  cd noir
  noirup -p ./
  ```

### Building the contracts

- In the aztec-packages repository, go to the directory yarn-project/noir-contracts

- Use the `noir:build:all` script to compile the contracts you want and prepare the ABI for consumption

  ```
  yarn noir:build:all
  ```

  Alternatively you can run `yarn noir:build CONTRACT1 CONTRACT2...` to build a subset of contracts:

  ```
  yarn noir:build private_token public_token
  ```

  To view compilation output, including errors, run with the `VERBOSE=1` flag:

  ```
  VERBOSE=1 yarn noir:build private_token public_token
  ```

## Creating a new contract package

1. Go to `src/contracts` folder.
2. Create a new package whose name has to end with **\_contract**. E.g.:
   ```
   nargo new example_contract
   ```
3. Add the aztec dependency to `nargo.toml`:

   ```
   [package]
   authors = [""]
   compiler_version = "0.7.1"

   [dependencies]
   aztec = { path = "../../../../noir-libs/noir-aztec" }
   ```

4. Replace the content of the generated `example_contract/src/main.nr` file with your contract code.
5. Go to `noir-contracts` root folder and run `yarn noir:build example` to compile the contract.
6. Export the abi in `src/artifacts/index.ts` to be able to use the contract in the rest of the project:
   ```
   import ExampleContractJson from './example_contract.json' assert { type: 'json' };
   export const ExampleContractAbi = ExampleContractJson as ContractAbi;
   ```
7. ???
8. Profit.
