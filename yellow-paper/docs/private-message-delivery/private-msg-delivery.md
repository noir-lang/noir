# Private Message Delivery

In Aztec, users need to pass private information between each other. Whilst Aztec enables users to share arbitrary private messages, we'll often frame the discussion towards a sender sharing the preimage of a private note with some recipient.

If Alice executes a function that generates a note for Bob:

1. Alice will need to **encrypt** that note such that Bob, and only Bob is able to decrypt it.
2. Alice will need to **broadcast** the encrypted note ciphertext so as to make it available for Bob to retrieve.
3. Alice will need to **broadcast a 'tag'** alongside the encrypted note ciphertext. This tag must be identifiable by Bob's chosen [note discovery protocol](./private-msg-delivery.md#note-discovery-protocol-selection) but not identifiable by any third party as "intended for Bob".

## Requirements

- **Users must be able to choose their note tagging mechanism**. We expect improved note discovery schemes to be designed over time. The protocol should be flexible enough to accommodate them and for users to opt in to using them as they become available. This flexibility should be extensible to encryption mechanisms as well as a soft requirement.
- **Users must be able to receive notes before interacting with the network**. A user should be able to receive a note just by generating an address. It should not be necessary for them to deploy their account contract in order to receive a note.
- **Applications must be able to safely send notes to any address**. Sending a note to an account could potentially transfer control of the call to that account, allowing the account to control whether they want to accept the note or not, and potentially bricking an application, since there are no catching exceptions in private function execution.
- **Addresses must be as small as possible**. Addresses will be stored and broadcasted constantly in applications. Larger addresses means more data usage, which is the main driver for cost. Addresses must fit in at most 256 bits, or ideally a single field element.
- **Total number of function calls should be minimized**. Every function call requires an additional iteration of the private kernel circuit, which adds several seconds of proving time.
- **Encryption keys should be rotatable**. Users should be able to rotate their encryption keys in the event their private keys are compromised, so that any further interactions with apps can be private again, without having to migrate to a new account.

## Constraining Message Delivery

The protocol will enable app developers to constrain the correctness of the following:

1. The encryption of a user's note.
2. The generation of the tag for that note.
3. The publication of that note and tag to the correct data availability layer.

Each app will define whether to constrain each such step. Encryption and tagging will be done through a set of [precompiled contracts](../addresses-and-keys/precompiles.md), each contract offering a different mechanism, and users will advertise their preferred mechanisms in a canonical [registry](../pre-compiled-contracts/registry.md).

The advantages of this approach are:

1. It enables a user to select their preferred [note discovery protocol](./private-msg-delivery.md#note-discovery-protocol-selection) and [encryption scheme](./private-msg-delivery.md#encryption-and-decryption).
2. It ensures that notes are correctly encrypted with a user's public encryption key.
3. It ensures that notes are correctly tagged for a user's chosen note discovery protocol.
4. It provides scope for upgrading these functions or introducing new schemes as the field progresses.
5. It protects applications from malicious unprovable functions.

## Note Discovery Protocol Selection

In order for a user to consume notes that belong to them, they need to identify, retrieve and decrypt them. A simple, privacy-preserving approach to this would be to download all of the notes and attempt decryption. However, the total number of encrypted notes published by the network will be substantial, making it infeasible for some users to do this. Those users will want to utilize a note discovery protocol to privately identify their notes.

Selection of the encryption and tagging mechanisms to use for a particular note are the responsibilty of a user's wallet rather than the application generating the note. This is to ensure the note is produced in a way compatible with the user's chosen note discovery scheme. Leaving this decision to applications could result in user's having to utilise multiple note discovery schemes, a situation we want to avoid.

## User Handshaking

Even if Alice correctly encrypts the note she creates for Bob and generates the correct tag to go with it, how does Bob know that Alice has sent him a note? Bob's note discovery protocol may require him to speculatively 'look' for notes with the tags that Alice (and his other counterparties) have generated. If Alice and Bob know each other then they can communicate out-of-protocol. But if they have no way of interacting then the network needs to provide a mechanism by which Bob can be alerted to the need to start searching for a specific sequence of tags.

To facilitate this we will deploy a canonical 'handshake' contract that can be used to create a private note for a recipient containing the sender's information (e.g. public key). It should only be necessary for a single handshake to take place between two users. The notes generated by this contract will be easy to identify, enabling users to retrieve these notes, decrypt them and use the contents in any deterministic tag generation used by their chosen note discovery protocol.

## Encryption and Decryption

Applications should be able to provably encrypt data for a target user, as part of private message delivery. As stated in the Keys section, we define three types of encrypted data, based on the sender and the recipient, from the perspective of a user:

Incoming data: data created by someone else, encrypted for and sent to the user.
Outgoing data: data created by the user to be sent to someone else, encrypted for the user.
Internal incoming data: data created by the user, encrypted for and sent to the user.
Encryption mechanisms support these three types of encryption, which may rely on different keys advertised by the user.

### Key Abstraction
To support different kinds of encryption mechanisms, the protocol does not make any assumptions on the type of public keys advertised by each user. Validation of their public keys is handled by the precompile contract selected by the user.

### Provable Decryption
While provable encryption is required to guarantee correct private message delivery, provable decryption is required for disclosing activity within an application. This allows auditability and compliance use cases, as well as being able to prove that a user did not execute certain actions. To support this, encryption precompiles also allow for provable decryption.

## Note Tagging

Note discovery schemes typically require notes to be accompanied by a stream of bytes generated specifically for the note discovery protocol. This 'tag' is then used in the procss of note identification. Whilst the tag itself is not sufficient to enable efficient note retrieval, the addition of it alongside the note enables the note discovery protocol to privately select a subset of the global set of notes to be returned to the user. This subset may still require some degree of trial-decryption but this is much more feasible given the reduced dataset.

When applications produce notes, they will need to call a protocol defined contract chosen by the recipient and request that a tag be generated. From the protocol's perspective, this tag will simply be a stream of bytes relevant only to the recipient's note discovery protocol. It will be up to the precompile to constrain that the correct tag has been generated and from there the protocol circuits along with the rollup contract will ensure that the tag is correctly published along with the note.

Constraining tag generation is not solely about ensuring that the generated tag is of the correct format. It is also necessary to constrain that tags are generated in the correct sequence. A tag sequence with duplicate or missing tags makes it much more difficult for the recipient to retrieve their notes. This will likely require tags to be nullified once used.