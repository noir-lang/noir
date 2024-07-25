# BB

### Why is this needed?

Barretenberg is a library that allows one to create and verify proofs. One way to specify the circuit that one will use to create and verify
proofs over is to use the Barretenberg standard library. Another way, which pertains to this module is to supply the circuit description using 
an IR called [ACIR](https://github.com/noir-lang/acvm).

This binary will take as input ACIR and witness values described in the IR to create proofs.

### Installation

1. Install `bbup` the installation script by running this in your terminal:

    ```bash
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
    ```

2. Reload your terminal shell environment:

    macOS:
    ```bash
    source ~/.zshrc
    ```

    Linux:
    ```bash
    source ~/.bashrc
    ```

3. Install the version of `bb` compatible with your Noir version; with Noir v0.32.0 for example:

    ```bash
    bbup -v 0.46.1
    ```

    Check the version compatibility section below for how to identify matching versions.
    
4. Check if the installation was successful:

    ```bash
    bb --version
    ```

If installation was successful, the command would print the version of `bb` installed.

### Version compatibility with Noir

TODO: https://github.com/AztecProtocol/aztec-packages/issues/7511

For quick reference:
- Noir v0.32.0 <> BB v0.46.1
- Noir v0.31.0 <> BB v0.41.0

### Usage

TODO: https://github.com/AztecProtocol/aztec-packages/issues/7600

Full list of available commands:
https://github.com/AztecProtocol/aztec-packages/blob/1a97698071a667cd56510c7b7201373a9ac9c646/barretenberg/cpp/src/barretenberg/bb/main.cpp#L1361-L1493

#### FilePath vs Stdout

For commands which allow you to send the output to a file using `-o {filePath}`, there is also the option to send the output to stdout by using `-o -`.

### Maximum circuit size

Currently the binary downloads an SRS that can be used to prove the maximum circuit size. This maximum circuit size parameter is a constant in the code and has been set to $2^{23}$ as of writing. This maximum circuit size differs from the maximum circuit size that one can prove in the browser, due to WASM limits.
