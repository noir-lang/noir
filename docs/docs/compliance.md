---
title: Compliance
---

# Compliance

In contrast to common misconceptions, privacy does not equate non-compliance. Ultimately, we believe in a future where users can easily generate zero-knowledge proofs to demonstrate compliance **without exposing any personal information** throughout the process. In that future, users have full agency over the intermediaries they choose to interact with, while centralized institutions (e.g. CEXs, tax authorities) can still maintain compliance with local legislations without accessing sensitive user information.

As we work towards that future, the measures below serve to limit the introduction of illicit funds into the Aztec Network and provide users with means to demonstrate compliance of their individual accounts, while preserving the ability of users to interact with Ethereum services using the Aztec Network.

## Network Compliance

The aim of network compliance measures is to limit the introduction of illicit funds (e.g. exploited funds from hacks) into the Aztec Network. They are designed around commonly seen illicit asset transfer patterns (e.g. large sums, time-sensitive).

### Block Deposit Cap

#### Address-specific Cap

A deposit amount cap is enforced on a per address, per asset, per rollup block basis at the smart contract level and on the [zk.money](https://zk.money/) frontend. It serves as:

- The first line of defense that hinders illicit deposits, and
- A measure to reduce users' exposed risks while Aztec is still experimental software

The cap limits the maximum amount of pending deposits that an Ethereum address can make at any instance. It is initially enforced at 5 ETH / 10,000 DAI and is adjustable depending on needs.

### Daily Deposit Cap

#### IP-specific Cap

A deposit rate cap is enforced on a per IP address, daily basis at the sequencer level. It serves as the second line of defense against illicit deposits that might attempt to circumvent the block deposit cap by making successive deposits.

The cap limits the maximum number of deposits from an IP address on each day and is reset at UTC midnight. The IP addresses stored for this purpose are deleted upon reset.

#### Network-wide Cap

A deposit amount cap is enforced on a network-wide, per asset, rolling daily basis at the smart contract level. It serves as the third line of defense against illicit deposits that might attempt to circumvent the above caps by splitting deposits across accounts.

The cap limits the maximum amount of pending deposits that can be made to the Aztec Network as a whole across 24-hour periods. Every pending deposit would consume a portion of the network-wide deposit quota that linearly refills at a rate of `network cap / 24 hours`. It is initially enforced at 1,000 ETH / 1,000,000 DAI and is adjustable depending on needs.

## User Compliance

The aim of user compliance measures is to provide users with means to demonstrate compliance of their individual accounts to e.g. auditors, government authorities and courts.

### Viewing Key Sharing

All Aztec accounts are created with [viewing keys](glossary.md#viewing-key) that guard viewing access to details of all transactions received and sent with the accounts (e.g. sender, receiver, asset type, amounts). In order to demonstrate compliance, users can share their viewing keys with whomever requesting viewing access to their Aztec transactions.

Users can currently retrieve their viewing keys using one of the following tools:

- [Hosted Frontend Boilerplate](https://aztec-frontend-boilerplate.netlify.app/)
- [Aztec CLI](https://github.com/critesjosh/azteccli)
- [Aztec SDK](sdk/usage/add-account.md#account-keys)

Research and development of better compliance demonstration tooling is in progress.
