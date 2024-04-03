---
title: Requirements
description: Requirements which motivated Aztec's design for addresses and keys.
---

## Requirements for Keys

### Scenario

A common illustration in this document is Bob sending funds to Alice, by:

- creating a "note" for her;
- committing to the contents of that note (a "note hash");
- inserting that note hash into a utxo tree;
- encrypting the contents of that note for Alice;
- optionally encrypting the contents of that note for Bob's future reference;
- optionally deriving an additional "tag" (a.k.a. "clue") to accompany the ciphertexts for faster note discovery;
- broadcasting the resulting ciphertext(s) (and tag(s));
- optionally identifying the tags;
- decrypting the ciphertexts; storing the note; and some time later spending (nullifying) the note.

> Note: there is nothing to stop an app and wallet from implementing its own key derivation scheme. Nevertheless, we're designing a 'canonical' scheme that most developers and wallets can use.

### Authorization keys

Aztec has native account abstraction, so tx authentication is done via an account contract, meaning tx authentication can be implemented however the user sees fit. That is, authorization keys aren't specified at the protocol level.

A tx authentication secret key is arguably the most important key to keep private, because knowledge of such a key could potentially enable an attacker to impersonate the user and execute a variety of functions on the network.

**Requirements:**

- A tx authentication secret key SHOULD NOT enter Aztec software, and SHOULD NOT enter a circuit.
  - Reason: this is just best practice.

### Master & Siloed Keys

**Requirements:**

- All keys must be re-derivable from a single `seed` secret.
- Users must have the option of keeping this `seed` offline, e.g. in a hardware wallet, or on a piece of paper.
- All master keys (for a particular user) must be linkable to a single address for that user.
- For each contract, a siloed set of all secret keys MUST be derivable.
  - Reason: secret keys must be siloed, so that a malicious app circuit cannot access and emit (as an unencrypted event or as args to a public function) a user's master secret keys or the secret keys of other apps.
- Master _secret_ keys must not be passed into an app circuit, except for precompiles.
  - Reason: a malicious app could broadcast these secret keys to the world.
- Siloed secret keys _of other apps_ must not be passed into an app circuit.
  - Reason: a malicious app could broadcast these secret keys to the world.
- The PXE must prevent an app from accessing master secret keys.
- The PXE must prevent an app from accessing siloed secret keys that belong to another contract address.
  - Note: To achieve this, the PXE simulator will need to check whether the bytecode being executed (that is requesting secret keys) actually exists at the contract address.
- There must be one and only one way to derive all (current\*) master keys, and all siloed keys, for a particular user address.
  - For example, a user should not be able to derive multiple different outgoing viewing keys for a single incoming viewing key (note: this was a 'bug' that was fixed between ZCash Sapling and Orchard).
  - \*"current", alludes to the possibility that the user might wish to support rotating keys, but only if one and only one set of keys is derivable as "current".
- All app-siloed keys can all be deterministically linked back to the user's address, without leaking important secrets to the app.

#### Security assumptions

- The Aztec private execution client (PXE), precompiled contracts (vetted application circuits), and the kernel circuit (a core protocol circuit) can be trusted with master secret keys (_except for_ the tx authorization secret key, whose security assumptions are abstracted-away to wallet designers).

### Encryption and decryption

Definitions (from the point of view of a user ("yourself")):

- Incoming data: Data which has been created by someone else, and sent to yourself.
- Outgoing data: Data which has been sent to somebody else, from you.
- Internal Incoming data: Data which has been created by you, and has been sent to yourself.
  - Note: this was an important observation by ZCash. Before this distinction, whenever a 'change' note was being created, it was being broadcast as incoming data, but that allowed a 3rd party who was only meant to have been granted access to view "incoming" data (and not "outgoing" data), was also able to learn that an "outgoing" transaction had taken place (including information about the notes which were spent). The addition of "internal incoming" keys enables a user to keep interactions with themselves private and separate from interactions with others.

**Requirements:**

- A user can derive app-siloed incoming internal and outgoing viewing keys.
  - Reason: Allows users to share app-siloed keys to trusted 3rd parties such as auditors, scoped by app.
  - Incoming viewing keys are not considered for siloed derivation due to the lack of a suitable public key derivation mechanism.
- A user can encrypt a record of any actions, state changes, or messages, to _themselves_, so that they may re-sync their entire history of actions from their `seed`.

### Nullifier keys

Derivation of a nullifier is app-specific; a nullifier is just a `field` (siloed by contract address), from the pov of the protocol.

Many private application devs will choose to inject a secret "nullifier key" into a nullifier. Such a nullifier key would be tied to a user's public identifier (e.g. their address), and that identifier would be tied to the note being nullified (e.g. the note might contain that identifier). This is a common pattern in existing privacy protocols. Injecting a secret "nullifier key" in this way serves to hide what the nullifier is nullifying, and ensures the nullifier can only be derived by one person (assuming the nullifier key isn't leaked).

> Note: not all nullifiers require injection of a secret _which is tied to a user's identity in some way_. Sometimes an app will need just need a guarantee that some value will be unique, and so will insert it into the nullifier tree.

**Requirements:**

- Support use cases where an app requires a secret "nullifier key" (linked to a user identity) to be derivable.
  - Reason: it's a very common pattern.

#### Is a nullifier key _pair_ needed?

I.e. do we need both a nullifier secret key and a nullifier public key? Zcash sapling had both, but Zcash orchard (an upgrade) replaced the notion of a keypair with a single nullifier key. The [reason](https://zcash.github.io/orchard/design/keys.html) being:

- _"[The nullifier secret key's (nsk's)] purpose in Sapling was as defense-in-depth, in case RedDSA [(the scheme used for signing txs, using the authentication secret key ask)] was found to have weaknesses; an adversary who could recover ask would not be able to spend funds. In practice it has not been feasible to manage nsk much more securely than a full viewing key [(dk, ak, nk, ovk)], as the computational power required to generate Sapling proofs has made it necessary to perform this step [(deriving nk from nsk)] on the same device that is creating the overall transaction (rather than on a more constrained device like a hardware wallet). We are also more confident in RedDSA now."_

A nullifier public key might have the benefit (in Aztec) that a user could (optionally) provide their nullifier key nk to some 3rd party, to enable that 3rd party to see when the user's notes have been nullified for a particular app, without having the ability to nullify those notes.

- This presumes that within a circuit, the nk (not a public key; still secret!) would be derived from an nsk, and the nk would be injected into the nullifier.
- BUT, of course, it would be BAD if the nk were derivable as a bip32 normal child, because then everyone would be able to derive the nk from the master key, and be able to view whenever a note is nullified!
- The nk would need to be a hardened key (derivable only from a secret).

Given that it's acceptable to ZCash Orchard, we accept that a nullifier master secret key may be 'seen' by Aztec software.

### Auditability

Some app developers will wish to give users the option of sharing private transaction details with a trusted 3rd party.

> Note: The [archive](./../state/archive.md) will enable a user to prove many things about their transaction history, including historical encrypted logs. This feature will open up exciting audit patterns, where a user will be able to provably respond to questions without necessarily revealing their private data. However, sometimes this might be an inefficient pattern; in particular when a user is asked to prove a negative statement (e.g. "prove that you've never owned a rock NFT"). Proving such negative statements might require the user to execute an enormous recursive function to iterate through the entire tx history of the network, for example: proving that, out of all the encrypted events that the user _can_ decrypt, none of them relate to ownership of a rock NFT. Given this (possibly huge) inefficiency, these key requirements include the more traditional ability to share certain keys with a trusted 3rd party.

**Requirements:**

- "Shareable" secret keys.
  - A user can optionally share "shareable" secret keys, to enable a 3rd party to decrypt the following data:
    - Outgoing data, across all apps
    - Outgoing data, siloed for a single app
    - Incoming internal data, across all apps
    - Incoming internal data, siloed for a single app
    - Incoming data, across all apps
    - Incoming data, siloed for a single app, is **not** required due to lack of a suitable derivation scheme
  - Shareable nullifier key.
    - A user can optionally share a "shareable" nullifier key, which would enable a trusted 3rd party to see _when_ a particular note hash has been nullified, but would not divulge the contents of the note, or the circumstances under which the note was nullified (as such info would only be gleanable with the shareable viewing keys).
  - Given one (or many) shareable keys, a 3rd part MUST NOT be able to derive any of a user's other secret keys; be they shareable or non-shareable.
    - Further, they must not be able to derive any relationships _between_ other keys.
- No impersonation.
  - The sharing of any (or all) "shareable" key(s) MUST NOT enable the trusted 3rd party to perform any actions on the network, on behalf of the user.
  - The sharing of a "shareable" outgoing viewing secret (and a "shareable" _internal_ incoming viewing key) MUST NOT enable the trusted 3rd party to emit encrypted events that could be perceived as "outgoing data" (or internal incoming data) originating from the user.
- Control over incoming/outgoing data.
  - A user can choose to only give incoming data viewing rights to a 3rd party. (Gives rise to incoming viewing keys).
  - A user can choose to only give outgoing data viewing rights to a 3rd party. (Gives rise to outgoing viewing keys).
  - A user can choose to keep interactions with themselves private and distinct from the viewability of interactions with other parties. (Gives rise to _internal_ incoming viewing keys).

### Sending funds before deployment

**Requirements:**

- A user can generate an address to which funds (and other notes) can be sent, without that user having ever interacted with the network.
  - To put it another way: A user can be sent money before they've interacted with the Aztec network (i.e. before they've deployed an account contract). e.g their incoming viewing key can be derived.
- An address (user identifier) can be derived deterministically, before deploying an account contract.

### Note Discovery

**Requirements:**

- A user should be able to discover which notes belong to them, without having to trial-decrypt every single note broadcasted on chain.
- Users should be able to opt-in to using new note discovery mechanisms as they are made available in the protocol.

#### Tag Hopping

Given that this is our best-known approach, we include some requirements relating to it:

**Requirements:**

- A user Bob can non-interactively generate a sequence of tags for some other user Alice, and non-interactively communicate that sequence of tags to Alice.
- If a shared secret (that is used for generating a sequence of tags) is leaked, Bob can non-interactively generate and communicate a new sequence of tags to Alice, without requiring Bob nor Alice to rotate their keys.
  - Note: if the shared secret is leaked through Bob/Alice accidentally leaking one of their keys, then they might need to actually rotate their keys.

### Constraining key derivations

- An app has the ability to constrain the correct encryption and/or note discovery tagging scheme.
- An app can _choose_ whether or not to constrain the correct encryption and/or note discovery tagging scheme.
  - Reason: constraining these computations (key derivations, encryption algorithms, tag derivations) will be costly (in terms of constraints), and some apps might not need to constrain it (e.g. zcash does not constrain correct encryption).

### Rotating keys

- A user should be able to rotate their set of keys, without having to deploy a new account contract.
  - Reason: keys can be compromised, and setting up a new identity is costly, since the user needs to migrate all their assets. Rotating encryption keys allows the user to regain privacy for all subsequent interactions while keeping their identity.
  - This requirement causes a security risk when applied to nullifier keys. If a user can rotate their nullifier key, then the nullifier for any of their notes changes, so they can re-spend any note. Rotating nullifier keys requires the nullifier public key, or at least an identifier of it, to be stored as part of the note. Alternatively, this requirement can be removed for nullifier keys, which are not allowed to be rotated.

<!-- TODO: Allow to rotate nullifier keys or not? -->

### Diversified Keys

- Alice can derive a diversified address; a random-looking address which she can (interactively) provide to Bob, so that Bob may send her funds (and general notes).
  - Reason: By having the recipient derive a distinct payment address _per counterparty_, and then interactively provide that address to the sender, means that if two counterparties collude, they won't be able to convince the other that they both interacted with the same recipient.
- Random-looking addresses can be derived from a 'main' address, so that private to public function calls don't reveal the true `msg_sender`. These random-looking addresses can be provably linked back to the 'main' address.
  > Note: both diversified and stealth addresses would meet this requirement.
- Distributing many diversified addresses must not increase the amount of time needed to scan the blockchain (they must all share a single set of viewing keys).

### Stealth Addresses

Not to be confused with diversified addresses. A diversified address is generated by the recipient, and interactively given to a sender, for the sender to then use. But a stealth address is generated by the _sender_, and non-interactively shared with the recipient.

Requirement:

- Random-looking addresses can be derived from a 'main' address, so that private -> public function calls don't reveal the true `msg_sender`. These random-looking addresses can be provably linked back to the 'main' address.
  > Note: both diversified and stealth addresses would meet this requirement.
- Unlimited random-looking addresses can be non-interactively derived by a sender for a particular recipient, in such a way that the recipient can use one set of keys to decrypt state changes or change states which are 'owned' by that stealth address.
