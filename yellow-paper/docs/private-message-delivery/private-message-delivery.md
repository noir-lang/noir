---
sidebar_position: 1
---

# Private Message Delivery

Maintaining the core tenet of privacy within the Aztec Network imposes a number of requirements related to the transfer of notes from one user to another. If Alice executes a function that generates a note for Bob:

1. Alice will need to **encrypt** that note such that Bob, and only Bob is able to decrypt it.
2. Alice will need to **broadcast** the encrypted note so as to make it available for Bob to retrieve.
3. Alice will need to **broadcast a 'tag'** alongside the encrypted note. This tag must be identifiable by Bob's chosen [note discovery protocol](./note-discovery.md) but not identifiable by any third party.

## Requirements

- **Users must be able to choose their note tagging mechanism**. We expect improved note discovery schemes to be designed over time. The protocol should be flexible enough to accommodate them and for users to opt in to using them as they become available. This flexibility should be extensible to encryption mechanisms as well as a soft requirement.
- **Users must be able to receive notes before interacting with the network**. A user should be able to receive a note just by generating an address. It should not be necessary for them to deploy their account contract in order to receive a note.
- **Applications must be able to safely send notes to any address**. Sending a note to an account could potentially transfer control of the call to that account, allowing the account to control whether they want to accept the note or not, and potentially bricking an application, since there is no catching exceptions in private function execution.
- **Addresses must be as small as possible**. Addresses will be stored and broadcasted constantly in applications. Larger addresses means more data usage, which is the main driver for cost. Addresses must fit in at most 256 bits, or ideally a single field element.
- **Total number of function calls should be minimized**. Every function call requires an additional iteration of the private kernel circuit, which adds several seconds of proving time.
- **Encryption keys should be rotatable**. Users should be able to rotate their encryption keys in the event their private keys are compromised, so that any further interactions with apps can be private again, without having to migrate to a new account.

## Constraining Message Delivery

The protocol will allow constraining:

1. The encryption of a user's note.
2. The generation of the tag for that note.
3. The publication of that note to the correct data availability layer.

Each app will define whether to constrain each step in private message delivery. Encryption and tagging will be done through a set of precompiled contracts, each contract offering a different mechanism, and users will advertise their preferred mechanisms in a canonical registry.

The advantages of this approach are:

1. It enables a user to select their preferred [note discovery protocol](./note-discovery.md) and [encryption scheme](./encryption-and-decryption.md).
2. It ensures that notes are correctly encrypted with a user's public encryption key.
3. It ensures that notes are correctly tagged for a user's chosen [note discovery protocol](./note-discovery.md).
4. It provides scope for upgrading these functions or introducing new schemes as the field progresses.
5. It protects applications from malicious unprovable functions.
