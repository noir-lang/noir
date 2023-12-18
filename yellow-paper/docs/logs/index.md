---
title: Logs
---

Logs on Aztec are similar to logs on Ethereum and their goal is to allow smart contracts to communicate arbitrary data to the outside world.
Logs are events which are emitted during contract function execution.
Aztec protocol gives users the following assurances:
1. The logs get published,
2. log integrity (the logs are not modified once emitted),
3. address of the source contract is verified to be correct (a contract can't impersonate another one).

:::warning Expand on how this is ensured in circuits once [this discussion](https://forum.aztec.network/t/issues-with-logs/2609/) is wrapped up.
:::

# Types
There are 2 kinds of logs in Aztec protocol: unencrypted and encrypted.

## Unencrypted
Unencrypted logs are used to communicate public information out of smart contracts.
Unencrypted logs can be emitted from both public and private functions.

:::info
Emitting unencrypted logs from private functions can be a privacy leak but we decided to not forbid it in-protocol because it might allow for interesting usecases like custom encryption schemes using FHE etc.
:::

## Encrypted
Encrypted logs can be emitted only from private functions.
This is because to encrypt the log we need to get a secret and it's impossible to privately manage secrets in public domain.

:::info
An important usecase of encrypted logs is delivery of notes (note commitment/hash preimage) to recipients.
:::

### Log encryption

:::warning
Expand here how exactly the logs are encrypted.
I (benesjan) am not up-to-date on what is the encryption end-game.
:::

# Encoding
Just like on Ethereum, logs are ABI encoded.

:::warning As far as I know the encoding will be happening in app circuit and won't be enforced by protocol. Should this section not be here for this reason?
:::