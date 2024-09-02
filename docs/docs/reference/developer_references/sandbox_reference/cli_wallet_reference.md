---
title: CLI Wallet
tags: [sandbox, wallet, cli]
---

For development, it may be useful to deploy, transact, or create notes in a non-programmatic way. You can use Aztec's CLI Wallet for thing such as:

- Deploying contracts
- Sending transactions
- Bridging L1 [Fee Juice](../../../protocol-specs/gas-and-fees/fee-juice.md) into Aztec
- Pushing arbitrary [notes](../../../guides/developer_guides/smart_contracts/writing_contracts/notes/index.md) to your PXE
- Creating [authwits](../../../guides/developer_guides/smart_contracts/writing_contracts/authwit.md)
- Aliasing info and secrets for further usage

:::info

At any time, you can get an updated version of the existing commands and subcommands by adding `-h`. For example:

```bash
aztec-wallet create-account -h
```

:::

## Aliases

The CLI wallet makes extensive use of aliases, that is, when an address, artifact, secret, or other information is given a name that can be later used to reference it.

Aliases have different types like `address` or `artifact` or `contract`. You can see a list of these types by running the help command `aztec-wallet alias -h`. You can then specify a type with the `:` character whenever needed. For example `accounts:master_yoda` or `artifacts:light_saber`.

:::tip

The wallet writes to the `last` alias if it's likely that you use that same alias in the next command.

It will also try to determine which type is expected. For example, if the alias `master_yoda` is an account, you don't need to prepend `account:` if, for example, you're deploying a contract.

You can create arbitrary aliases with the `alias` command. For example `aztec-wallet alias accounts test_alias 0x2c37902cdade7710bd2355e5949416dc5e43a16e0b13a5560854d2451d92d289`.


## Account Management

The wallet comes with some options for account deployment and management. You can register and deploy accounts, or only register them, and pass different options to serve your workflow.

### create-account

Generates a secret key and deploys an account contract.

#### Example

```bash
aztec-wallet create-account -a master_yoda
```

### Deploy account

Deploy an account that is already registered (i.e. your PXE knows about it) but not deployed. Most times you should pass an alias or address registered in the PXE by passing the `-f` or `--from` flag.

#### Example

```bash
$ aztec-wallet create-account --register-only -a master_yoda
...
$ aztec-wallet deploy-account -f master_yoda
```

### Deploy

You can deploy a [compiled contract](../../../guides/developer_guides/smart_contracts/how_to_compile_contract.md) to the network.

You probably want to look at flags such as `--init` which allows you to specify the [initializer function](../../../guides/developer_guides/smart_contracts/writing_contracts/initializers.md) to call, or `-k` for the [encryption public key](../../../aztec/concepts/accounts/keys.md#incoming-viewing-keys) if the contract is expected to have notes being encrypted to it.

You can pass arguments with the `--arg` flag.

#### Example

This example compiles the Jedi Code and deploys it from Master Yoda's account, initializing it with the parameter "Grand Master" and aliasing it to `jedi_order`. Notice how we can simply pass `master_yoda` in the `--from` flag (because `--from` always expects an account):

```bash
aztec-nargo compile
aztec-wallet deploy ./target/jedi_code.nr --arg accounts:master_yoda --from master_yoda --alias jedi_order
```

### Send

This command sends a transaction to the network by calling a contract's function. Just calling `aztec-wallet send` gives you a list of options, but you probably want to pass `--from` as the sender, `--contract-address` for your target's address, and `--args` if it requires arguments.

#### Example

```bash
aztec-wallet send --from master_yoda --contract-address jedi_order --args "luke skywalker" train_jedi
```

Again, notice how it's not necessary to pass `contracts:jedi_order` as the wallet already knows that the only available type for `--contract-address` is a contract.

### Manage authwits

You can use the CLI wallet to quickly generate [Authentication Witnesses](../../../guides/developer_guides/smart_contracts/writing_contracts/authwit.md). These allow you to authorize the caller to execute an action on behalf of an account. They get aliased into the `authwits` type.

### In private

The authwit management in private is a two-step process: create and add. It's not too different from a `send` command, but providing the caller that can privately execute the action on behalf of the caller.

#### Example

An example for authorizing an operator (ex. a DeFi protocol) to call the transfer_from action (transfer on the user's behalf):

```bash
aztec-wallet create-authwit transfer_from accounts:coruscant_trader -ca contracts:token --args accounts:jedi_master accounts:coruscant_trader 20 secrets:auth_nonce -f accounts:jedi_master -a secret_trade

aztec-wallet add-authwit authwits:secret_trade accounts:jedi_master -f accounts:coruscant_trader
```

### In public

A similar call to the above, but in public:

```bash
aztec-wallet authorize-action transfer_public accounts:coruscant_trader -ca contracts:token --args accounts:jedi_master accounts:coruscant_trader 20 secrets:auth_nonce -f accounts:jedi_master
```

### Simulate

Simulates a transaction instead of sending it. This allows you to obtain i.e. the return value before sending the transaction.

#### Example

```bash
aztec-wallet simulate --from master_yoda --contract-address jedi_order --args "luke_skywalker" train_jedi
```

### Bridge Fee Juice

The wallet provides an easy way to mint the fee-paying asset on L1 and bridging it to L2. We call it Fee Juice and you can read more about it in the [protocol specs](../../../protocol-specs/gas-and-fees/fee-juice.md).

Using the sandbox, there's already a Fee Juice contract that manages this enshrined asset. You can optionally mint more Juice before bridging it.

#### Example

This example mints and bridges 1000 units of fee juice and bridges it to the `master_yoda` recipient on L2.

```bash
aztec-wallet bridge-fee-juice --mint 1000 master_yoda
```

### Add Note

The Add Note method makes it easy to store notes on your local PXE if they haven't been broadcasted yet. For example, if a JediMember note was sent to you, and you want to spend it on another transaction, you can use this method with the `--transaction-hash` flag to pass the transaction hash that contains the note.

It expects `name` and `storageFieldName`. For example, if the `#[storage]` struct had a `available_members: PrivateMutable<JediMember>` property:

```bash
aztec-note add-note JediMember available_members -a master_yoda -ca jedi_order -h 0x00000
```
