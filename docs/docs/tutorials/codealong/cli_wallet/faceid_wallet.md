---
title: FaceID Wallet (Mac Only)
---

In this tutorial, we will use Apple Mac's Secure Enclave to store the private key, and use it in Aztec's [CLI Wallet](../../../reference/developer_references/sandbox_reference/cli_wallet_reference.md). This enables fully private, native, and seedless account abstraction!

:::warning

Aztec is in active development and this has only been tested on MacOS. Please reach out if this tutorial does not work for you, and let us know your operating system.

:::

## Prerequisites

For this tutorial, we will need to have the the [Sandbox](../../../reference/developer_references/sandbox_reference/index.md) installed.

We also need to install Secretive, a nice open-source package that allows us to store keys on the Secure Enclave. You can head to the [secretive releases page](https://github.com/maxgoedjen/secretive/releases) and get the last release's `zip`, unzip and move to Applications, or use [Homebrew](https://brew.sh/):

```bash
brew install secretive
```

Open it from the Applications folder and copy the provided Socket Path (the one it tells you to add to your .ssh config). Export it as a terminal environment variable. For example:

```bash
export SSH_AUTH_SOCK="/Users/your_user/Library/Containers/com.maxgoedjen.Secretive.SecretAgent/Data/socket.ssh"
```

Let's also install `socat` which helps us manage the socket connections. If using Homebrew:

```bash
brew install socat
```

### Creating a key

We will create our private key, which will be stored in the Secure Enclave. Open Secretive, click the "+" sign and create a key with authentication. You can give it any name you like. Secretive will then store it in the Secure Enclave.

Make sure Secretive's "Secret Agent" is running.

:::info

The Secure Enclave is a protected chip on most recent iPhones and Macs and it's meant to be airgapped. It is not safe to use in production.

Fortunately, Aztec implements [Account Abstraction](../../../aztec/concepts/accounts#what-is-account-abstraction) at the protocol level. You could write logic to allow someone else to recover your account, or use a different key or keys for recovery.

:::

### Using the wallet

Now we can use the key in our wallet. Every account on Aztec is a contract, so you can write your own contract with its own account logic.

The Aztec team already wrote some account contract boilerplates we can use. One of them is an account that uses the `secp256r1` elliptic curve (the one the Secure Enclave uses).

Let's create an account in our wallet:

```bash
aztec-wallet create-account -a my-wallet -t ecdsasecp256r1ssh
```

This command creates an account using the `ecdsasecp256r1ssh` type and aliases it to `my-wallet`.

You should see a prompt like `? What public key to use?` with the public key you created in Secretive. Select this. If you see the message `Account stored in database with aliases last & my-wallet` then you have successfully created the account!

You can find other accounts by running `aztec-wallet create-account -h`.

### Using the wallet

You can now use it as you would use any other wallet. Let's create a simple token contract example and mint ourselves some tokens with this.

Create a new Aztec app with `npx aztec-app`:

```bash
npx aztec-app new -s -t contract -n token_contract token
```

This creates a new project, skips running the sandbox (`-s`), and clones the contract-only box (`-t`) called token_contract (`-n`). You should now have a `token_contract` folder. Let's compile our contract:

```bash
cd token_contract
aztec-nargo compile
```

Great, our contract is ready to deploy with our TouchID wallet:

```bash
aztec-wallet deploy --from accounts:my-wallet token_contract@Token --args accounts:my-wallet DevToken DTK 18 -a devtoken

You should get prompted to sign with TouchID or password. Once authorized, you should see `Contract stored in database with aliases last & devtoken`
```

Check [the reference](../../../reference/developer_references/sandbox_reference/cli_wallet_reference.md) for the whole set of commands, but these mean:

- --from is the sender: our account `my-wallet`. We use the alias because it's easier than writing the key stored in our Secure Enclave. The wallet resolves the alias and knows where to grab it.
- token_contract@Token is a shorthand to look in the `target` folder for our contract `token_contract-Token`
- --args are the arguments for our token contract: owner, name, ticker and decimals.
- -a tells the wallet to store its address with the "devtoken" alias, this way we can just use it later like `contracts:devtoken`

You should get a prompt to sign this transaction. You can now mint, transfer, and do anything you want with it:

```bash
aztec-wallet create-account -a new_recipient # creating a schnorr account
aztec-wallet send mint_public -ca last --args accounts:my-wallet 10 -f accounts:my-wallet # minting some tokens in public
aztec-wallet simulate balance_of_public -ca contracts:devtoken --args accounts:my-wallet -f my-wallet # checking that my-wallet has 10 tokens
aztec-wallet send transfer_public -ca contracts:devtoken --args accounts:my-wallet accounts:new_recipient 10 0 -f accounts:my-wallet # transferring some tokens in public
aztec-wallet simulate balance_of_public -ca contracts:devtoken --args accounts:new_recipient -f my-wallet # checking that new_recipient has 10 tokens
```

### What next

In this tutorial, we created an account with the Aztec's [CLI Wallet](../../../reference/developer_references/sandbox_reference/cli_wallet_reference.md), using the Apple Mac's Secure Enclave to store the private key.

You can use a multitude of authentication methods, for example with RSA you could use a passport as a recovery, or even as a signer in a multisig. All of this is based on the account contract.

Next step is then to [code your own account contract!](../contract_tutorials/write_accounts_contract.md)
