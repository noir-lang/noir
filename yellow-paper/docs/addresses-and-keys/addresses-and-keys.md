---
sidebar_position: 1
---

$$
\gdef\sk{\color{red}{sk}}

\gdef\nskm{\color{red}{nsk_m}}
\gdef\tskm{\color{red}{tsk_m}}
\gdef\ivskm{\color{red}{ivsk_m}}
\gdef\ovskm{\color{red}{ovsk_m}}
\gdef\rivskm{\color{red}{rivsk_m}}
\gdef\rovskm{\color{red}{rovsk_m}}

\gdef\Npkm{\color{green}{Npk_m}}
\gdef\Tpkm{\color{green}{Tpk_m}}
\gdef\Ivpkm{\color{green}{Ivpk_m}}
\gdef\Ovpkm{\color{green}{Ovpk_m}}
\gdef\Rivpkm{\color{green}{Rivpk_m}}
\gdef\Rovpkm{\color{green}{Rovpk_m}}

\gdef\shareableivskm{\color{orange}{\widetilde{ivsk_{m}}}}
\gdef\shareableIvpkm{\color{green}{\widetilde{Ivpk_{m}}}}
\gdef\shareableovskm{\color{orange}{\widetilde{ovsk_{m}}}}


\gdef\address{\color{green}{address}}
\gdef\codehash{\color{green}{code\_hash}}


\gdef\nskapp{\color{red}{nsk_{app}}}
\gdef\tskapp{\color{red}{tsk_{app}}}
\gdef\ivskapp{\color{red}{ivsk_{app}}}
\gdef\ovskapp{\color{red}{ovsk_{app}}}
\gdef\rivskapp{\color{red}{rivsk_{app}}}
\gdef\rovskapp{\color{red}{rovsk_{app}}}

\gdef\Nkapp{\color{orange}{Nk_{app}}}

\gdef\Npkapp{\color{green}{Npk_{app}}}


\gdef\Ivpkapp{\color{green}{Ivpk_{app}}}


\gdef\Rivpkapp{\color{green}{Rivpk_{app}}}


\gdef\happL{\color{green}{h_{app}^L}}
\gdef\happn{\color{green}{h_{app}^n}}
\gdef\happiv{\color{green}{h_{app}^{iv}}}
\gdef\happriv{\color{green}{h_{app}^{riv}}}
\gdef\happrov{\color{green}{h_{app}^{rov}}}


\gdef\shareableivskapp{\color{orange}{\widetilde{ivsk_{app}}}}
\gdef\shareableIvpkapp{\color{green}{\widetilde{Ivpk_{app}}}}
\gdef\shareableovskapp{\color{orange}{\widetilde{ovsk_{app}}}}


\gdef\d{\color{green}{d}}
\gdef\Gd{\color{green}{G_d}}

\gdef\Ivpkappd{\color{violet}{Ivpk_{app,d}}}
\gdef\Rivpkappd{\color{violet}{Rivpk_{app,d}}}
\gdef\Pkappd{\color{violet}{Pk_{app,d}}}
\gdef\shareableIvpkappd{\color{violet}{\widetilde{Ivpk_{app,d}}}}


\gdef\ivskappstealth{\color{red}{ivsk_{app,stealth}}}
\gdef\Ivpkappdstealth{\color{violet}{Ivpk_{app,d,stealth}}}
\gdef\Pkappdstealth{\color{violet}{Pk_{app,d,stealth}}}
\gdef\shareableivskappstealth{\color{orange}{\widetilde{ivsk_{app,stealth}}}}
\gdef\shareableIvpkappdstealth{\color{violet}{\widetilde{Ivpk_{app,d,stealth}}}}

\gdef\hstealth{\color{violet}{h_{stealth}}}


\gdef\esk{\color{red}{esk}}
\gdef\Epk{\color{green}{Epk}}
\gdef\Epkd{\color{green}{Epk_d}}
\gdef\eskheader{\color{red}{esk_{header}}}
\gdef\Epkheader{\color{green}{Epk_{header}}}
\gdef\Epkdheader{\color{green}{Epk_{d,header}}}

\gdef\sharedsecret{\color{violet}{\text{S}}}
\gdef\sharedsecretmheader{\color{violet}{\text{S_{m,header}}}}
\gdef\sharedsecretappheader{\color{violet}{\text{S_{app,header}}}}


\gdef\hmencheader{\color{violet}{h_{m,enc,header}}}
\gdef\happencheader{\color{violet}{h_{app,enc,header}}}
\gdef\hmenc{\color{violet}{h_{m,enc}}}
\gdef\happenc{\color{violet}{h_{app,enc}}}
\gdef\incomingenckey{\color{violet}{h_{incoming\_enc\_key}}}


\gdef\plaintext{\color{red}{\text{plaintext}}}
\gdef\ciphertext{\color{green}{\text{ciphertext}}}
\gdef\ciphertextheader{\color{green}{\text{ciphertext\_header}}}
\gdef\payload{\color{green}{\text{payload}}}


\gdef\tagg{\color{green}{\text{tag}}}
\gdef\Taghs{\color{green}{\text{Tag}_{hs}}}

$$

# Proposal: Aztec Keys


## Requirements for Keys

:::info Disclaimer
This is a draft. These requirements need to be considered by the wider team, and might change significantly before a mainnet release.

Eventually, the key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “MAY”, and “OPTIONAL” in this document should align with the interpretations as described in RFC 2119
:::

These requirements take priority over optimisations.

Note: there is nothing to stop an app and wallet from implementing its own key derivation scheme. Nevertheless, we're designing a 'canonical' scheme that most developers and wallets can use.

### Authorisation keys

Tx authentication is done via a custom account contract, so authorisation keys aren't specified at the protocol level.

A tx authentication secret key is arguably the most important key to keep private, because knowledge of such a key could potentially enable an attacker to impersonate the user and execute a variety of functions on the network. 

Requirements:
- A tx authentication secret key SHOULD NOT enter Aztec software, and SHOULD NOT enter a circuit.
  - Reason: this is just best practice.


### Master & Siloed Keys

Requirements:
- All keys must be re-derivable from a single `seed` secret.
- Users must have the option of keeping this `seed` offline, e.g. in a hardware wallet, or on a piece of paper.
- All master keys (for a particular user) must be linkable to a single "user identifier" for that user.
  - Notice: we don't prescribe whether this "user identifier" must be a public key, or a field (i.e. an address). The below protocol suggestion has ended up with an address, but that's not to say it's required.
- For each contract, a siloed set of all keys MUST be derivable.
  - Reason: secret keys must be siloed, so that a malicious app circuit cannot access and emit (as an unencrypted event or as args to a public function) a user's master secret keys or the secret keys of other apps.
- Master _secret_ keys must not be passed into an app circuit.
  - Reason: a malicious app could broadcast these secret keys to the world.
- Siloed secret keys _of other apps_ must not be passed into an app circuit.
  - Reason: a malicious app could broadcast these secret keys to the world.
- The PXE must prevent an app from accessing master secret keys.
- The PXE must prevent an app from accessing siloed secret keys that belong to another contract address.
  - Note: To achieve this, the PXE simulator will need to check whether the bytecode being executed (that is requesting secret keys) actually exists at the contract address.
- There must be one and only one way to derive all (current*) master keys, and all siloed keys, for a particular "user identifier".
  - For example, a user should not be able to derive multiple different outgoing viewing keys for a single incoming viewing key (note: this was a 'bug' that was fixed between ZCash Sapling and Orchard).
  - \*"current", alludes to the possibility that the protocol might wish to support rotating of keys, but only if one and only one set of keys is derivable as "current".
- All app-siloed keys can all be deterministically linked back to the user's address, without leaking important secrets to the app.

### Account Contract Bytecode

Requirements:
- The (current) bytecode of an account contract must be discoverable, based on the "user identifier" (probably an address).
  - Reason: Although in 'private land', the king of the hill griefing problem prevents private account contract acir from being callable, there might still be uses where a _public_ function of an account contract might wish to be called by someone.
- The (current) bytecode of an account contract must therefore be 'tied to' to the "user identifier".
- The constructor arguments of an account contract must be discoverable.


### Sending funds before deployment

Requirements:
- A user can generate an address to which funds (and other notes) can be sent, without that user having ever interacted with the network.
  - To put it another way: A user can be sent money before they've interacted with the Aztec network (i.e. before they've deployed an account contract). e.g their incoming viewing key can be derived.
- An address (user identifier) can be derived deterministically, before deploying an account contract.

### Encryption and decryption

Definitions (from the point of view of a user ("yourself")):
- Incoming data: Data which has been created by someone else, and sent to yourself.
- Outgoing data: Data which has been sent to somebody else, from you.
- Internal Incoming data: Data which has been created by you, and has been sent to yourself.
  - Note: this was an important observation by ZCash. Before this distinction, whenever a 'change' note was being created, it was being broadcast as incoming data, but that allowed a 3rd party who was only meant to have been granted access to view "incoming" data (and not "outgoing" data), was also able to learn that an "outgoing" transaction had taken place (including information about the notes which were spent). The addition of "internal incoming" keys enables a user to keep interactions with themselves private and separate from interactions with others.

Requirements:
- A user can non-interactively encrypt state changes, actions, and messages to some other user.
- A user can encrypt a record of any actions, state changes, or messages, to _themselves_, so that they may re-sync their entire history of actions from their `seed`.
- A user can derive siloed keys-used-for-encryption on a _per app_ basis.
  - Reason: if one app-siloed key is compromised, it doesn't leak the private transaction data of other apps.
- If Bob's keys-used-for-encryption are leaked, it doesn't leak the details of Bob's interactions with Alice.
  - Note: I'm not sure we can achieve this, given that Bob can encrypt outgoing data for himself. If Bob didn't encrypt data for himself, this would be achieved.


### Auditability

Some app developers will wish to give users the option of sharing private transaction details with a trusted 3rd party.

Note: The block hashes tree will enable a user to prove many things about their historical transaction history, including historical encrypted event logs. This feature will open up exciting audit patterns, where a user will be able to provably respond to questions without necessarily revealing their private data. However, sometimes this might be an inefficient pattern; in particular when a user is asked to prove a negative statement (e.g. "prove that you've never owned a rock NFT"). Proving such negative statements might require the user to execute an enormous recursive function to iterate through the entire tx history of the network, for example: proving that, out of all the encrypted events that the user _can_ decrypt, none of them relate to ownership of a rock NFT. Given this (possibly huge) inefficiency, these key requirements include the more traditional ability to share certain keys with a trusted 3rd party.

Requirements:
- Shareable secret keys.
  - A user can optionally share "shareable" secret keys for decryption of the following data:
    - Incoming data, across all apps ($\shareableivskm$)
    - Incoming data, siloed for a single app ($\shareableivskapp$)
    - Outgoing data, across all apps ($\shareableovskm$)
    - Outgoing data, siloed for a single app ($\shareableovskapp$)
    - Incoming internal data, across all apps (TODO)
    - Incoming internal data, siloed for a single app (TODO)
- No impersonation.
  - The sharing of any (or all) "shareable" key(s) MUST NOT enable the trusted 3rd party to perform any actions on the network, on behalf of the user.
  - The sharing of a "shareable" outgoing viewing secret (and a "shareable" _internal_ incoming viewing key) MUST NOT enable the trusted 3rd party to emit encrypted events that could be perceived as "outgoing data" (or internal incoming data) originating from the user.
- Control over incoming/outgoing data.
  - A user can choose to only give incoming data viewing rights to a 3rd party. (Gives rise to incoming viewing keys).
  - A user can choose to only give outgoing data viewing rights to a 3rd party. (Gives rise to outgoing viewing keys).
  - A user can choose to keep interactions with themselves private and distinct from the viewability of interactions with other parties. (Gives rise to _internal_ incoming viewing keys).
- No leakage of non-shareable keys.
  - Given one (or many) shareable keys, a 3rd part MUST NOT be able to derive any other of a user's keys; be they shareable or non-shareable.
    - Further, they must not be able to derive any relationships _between_ other keys. (This point is inspired by an attack I found where using the same randomness to derive shareable incoming and outgoing keys, resulted in a 3rd party being able to deduce the _difference_ between master secret keys). 

Nice to have:
- Shareable nullifier key.
  - A user can optionally share a "shareable" nullifier key, which would enable a trusted 3rd party to see _when_ a particular note hash has been nullified, but would not divulge the contents of the note, or the circumstances under which the note was nullified (as such info would only be gleanable with the shareable viewing keys).
  - Note: essentially, imagine if you were given a nullifier key, and the nullifier is computed as `h(note_hash, nullifier_key)`. Then someone with the nullifier key could brute-force attempt to hash every `note_hash` in the Note Hashes Tree with the `nullifier_key` in search of matching nullifiers in the nullifier tree - at which point they learn "this note hash just got spent by Bob". It seemed like a good idea, but in practice it might not be useful (too little information to draw any meaning from) or efficient (too compute intensive).


### Note Discovery

Requirements:
- [Nice to have]: A "tagging" keypair that enables faster brute-force identification of owned notes.
  - Note: this is useful for rapid handshake discovery, but it is an optimisation, and has trade-offs.
- [Nice to have]: The ability to generate a sequence of tags between Alice and Bob, in line with our latest "Tag Hopping" ideas.

Considerations:
- There is no enshrined tagging scheme, currently. Whether to enshrine protocol functions (to enable calls to private account contract functions) or to let apps decide on tagging schemes, is an open debate.

#### Tag Hopping

Given that this is our best-known approach, we include some requirements relating to it:

Requirements:
- A user Bob can non-interactively generate a sequence of tags for some other user Alice, and non-interactively communicate that sequencer of tags to Alice.
- If a shared secret (that is used for generating a sequence of tags) is leaked, Bob can non-interactively generate and communicate a new sequence of tags to Alice, without requiring Bob nor Alice to rotate their keys.
  - Note: if the shared secret is leaked through Bob/Alice accidentally leaking one of their keys, then they might need to actually rotate their keys.

### Constraining key derivations

- An app has the ability to constrain the correct encryption and/or note discovery tagging scheme.
- An app can _choose_ whether or not to constrain the correct encryption and/or note discovery tagging scheme.
  - Reason: constraining these computations (key derivations, encryption algorithms, tag derivations) will be costly (in terms of constraints), and some apps might not need to constrain it (e.g. zcash does not constrain correct encryption).

### Rotating keys

This is currently being treated as 'nice to have', simply because it's difficult and causes many complexities.

Nice to haves:
- A user can change the following keys, at the _master key_ level:
  - Tx authentication key
  - Nullifier key


Considerations:
- No consideration has yet been given to enabling keys to be rotated on a _per app_ basis; that is, the ability to rotate the keys for one app, independently of all other app and master keys. 


### Nullifier keys

Derivation of a nullifier is app-specific; a nullifier is just a `field` (siloed by contract address), from the pov of the protocol.

Many private application devs will choose to inject a secret "nullifier key" into a nullifier. Such a nullifier key would be tied to a user's public identifier (e.g. their address), and that identifier would be tied to the note being nullified (e.g. the note might contain that identifier). This is a common pattern in existing privacy protocols. Injecting a secret "nullifier key" in this way serves to hide what the nullifier is nullifying, and ensures the nullifier can only be derived by one person (assuming the nullifier key isn't leaked).

The only alternative to this pattern is plume nullifiers, but there are tradeoffs, so we'll continue to provide support for non-plume nullifiers.

> Note: not all nullifiers require injection of a secret _which is tied to a user's identity in some way_. Alternative examples of nullifier derivations (serving other use cases) are: hashing a storage slot; hashing a note hash and a secret contained within the note hash; ...?

Requirements:
- Support use cases where an app requires a secret "nullifier key" (linked to a user identity) to be derivable.
  - Reason: it's a very common pattern.



#### Is a nullifier key _pair_ needed?

I.e. do we need both a nullifier secret key and a nullifier public key? Zcash sapling had both, but Zcash orchard (an upgrade) replaced the notion of a keypair with a single nullifier key. The [reason](https://zcash.github.io/orchard/design/keys.html) being:
- _"[The nullifier secret key's (nsk's)] purpose in Sapling was as defense-in-depth, in case RedDSA [(the scheme used for signing txs, using the authentication secret key ask)] was found to have weaknesses; an adversary who could recover ask would not be able to spend funds. In practice it has not been feasible to manage nsk much more securely than a full viewing key [(dk, ak, nk, ovk)], as the computational power required to generate Sapling proofs has made it necessary to perform this step [(deriving nk from nsk)] on the same device that is creating the overall transaction (rather than on a more constrained device like a hardware wallet). We are also more confident in RedDSA now."_

A nullifier public key might have the benefit (in Aztec) that a user could (optionally) provide their nullifier key nk to some 3rd party, to enable that 3rd party to see when the user's notes have been nullified for a particular app, without having the ability to nullify those notes.
    - This presumes that within a circuit, the nk (not a public key; still secret!) would be derived from an nsk, and the nk would be injected into the nullifier.
    - BUT, of course, it would be BAD if the nk were derivable as a bip32 normal child, because then everyone would be able to derive the nk from the master key, and be able to view whenever a note is nullified!
    - The nk would need to ba a hardened key (derivable only from a secret).

Given that it's acceptable to ZCash Orchard, we accept that a nullifier master secret key may be 'seen' by Aztec software.
    

### Diversified Addresses

This is perhaps a "nice to have", although it's a core feature of zcash keys.

Nice to have:
- Alice can derive a diversified address; a random-looking address which she can (interactively) provide to Bob, so that Bob may send her funds (and general notes).
  - Reason: By having the recipient derive a distinct payment address _per counterparty_, and then interactively provide that address to the sender, means that if two counterparties collude, they won't be able to convince the other that they both interacted with the same recipient.
- Distributing many diversified addresses must not increase the amount of time needed to scan the blockchain (they must all share a single set of viewing keys).


### Stealth Addresses

Not to be confised with diversified addresses. A diversified address is generated by the recipient, and interactively given to a sender, for the sender to then use. But a stealth address is generated by the _sender_, and non-interactively shared with the recipient.

Requirement:
- Random-looking addresses can be derived from a 'main' address, so that private -> public function calls don't reveal the true `msg_sender`. These random-looking addresses can be provably linked back to the 'main' address. 
  - This requirement can probably be satisfied with either/both stealth and diversified addresses.
- Unlimited random-looking addresses can be non-interactively derived by a sender for a particular recipient, in such a way that the recipient can use one set of keys to decrypt state changes or change states which are 'owned' by that stealth address.

:::warning
Problem: we can derive diversified/stealth _public keys_... but how to convert them into an _address_ (which would be important to have natural address-based semantics for managing state that is owned by a stealth/diversified address)?
:::

---

:::danger
There are lots of general encryption requirements which are not-yet listed here, such as preventing chosen plaintext/ciphertext attacks etc.
:::

:::danger
There are some more involved complications and considerations, which haven't all fully been considered. Relevant reading:
- [Derivation of an ephemeral secret from a note plaintext](https://zips.z.cash/zip-0212) (includes commentary on an attack that can link two diversified addresses).
- [In-band secret distribution](https://zips.z.cash/protocol/protocol.pdf) - p143 of the zcash spec.
:::

## Is this final?

No.

The 'topology' of the key derivations (i.e. the way the derivations of the keys interrelate, if you were to draw a dependency graph) is not constraint-optimised. There might be a better 'layout'.

Domain separation hasn't been considered in-depth, so take it with a pinch of salt.

Ephemeral key re-use needs to be considered carefully.

The requirements themselves might be adjusted (which might affect this key scheme significantly)


## Notation

- An upper-case first letter is used for elliptic curve points (all on the Grumpkin curve) (e.g. $\Ivpkm$).
- A lower-case first letter is used for scalars. (TODO: improve this. Some hash outputs might be 256-bit instead of a field element).
- $G$ is a generator point on the Grumpkin curve.
- A "cdot" ("$\cdot$") is used to denote scalar multiplication.
- "$+$" should be clear from context whether it's field or point addition.
- A tilde above a name is used to remind the reader that it's a randomised version of the 'tilde-less' name (e.g. $\shareableivskm$ is a randomisation of $\ivskm$).
- A function 'h()' is a placeholder for some as-yet-undecided hash function or pseudo-random function, the choice of which is tbd. Note: importantly, 'h()' is lazy notation, in that not all instances of h() imply the same hash function should be used.
- A subscript $m$ is used on keys to mean "master key".
- A subscript $app$ is used on keys to mean "an app-siloed key, derived from the master key and the app's contract address".
- A subscript $d$ is used on keys to mean "diversified". Although note: a diversifier value of $d = 1$ implies no diversification, as will often be the case in practice.


## Colour Key

Haha. Key. Good one.

- $\color{green}{green}$ = Publicly shareable information.
- $\color{red}{red}$ = Very secret information. A user MUST NOT share this information.
    - TODO: perhaps we distinguish between information that must not be shared to prevent theft, and information that must not be shared to preserve privacy?
- $\color{orange}{orange}$ = Secret information. A user MAY elect to share this information with a _trusted_ 3rd party, but it MUST NOT be shared with the wider world.
- $\color{violet}{violet}$ = Secret information. Information that is shared between a sender and recipient (and possibly with a 3rd party who is entrusted with viewing rights by the recipient).


## Master Keys


| Key | Derivation | Name | Where? | Comments |
|---|---|---|---|---|
$\sk$ | $\stackrel{\$}{\leftarrow} \mathbb{F}$ | secret key | TEE/ PXE | A seed secret from which all these other keys may be derived. For future reference (in case we modify the schemes), this $\sk$ doesn't need to enter a circuit if all keys can be provably linked/tethered to some fixed public key/address. |
$\nskm$ | h(0x01, $\sk$) | nullifier secret key | PXE, K | Gives developers the option of using a secret key to derive their apps' nullifiers. (Not all nullifiers require a secret key, e.g. plume nullifiers). |
$\tskm$ | h(0x02, $\sk$) | tagging secret key | PXE* | The "tagging" key pair can be used to flag "this ciphertext is for you", without requiring decryption. | 
$\ivskm$ | h(0x03, $\sk$) | incoming viewing secret key | PXE* | The owner of this secret key can derive ephemeral symmetric encryption keys, to decrypt ciphertexts which _have been sent to them_ (i.e. "incoming" data from the pov of the recipient). |  
$\ovskm$ | h(0x04, $\sk$) | outgoing viewing secret key | PXE* | The owner of this secret key can derive ephemeral symmetric encryption keys, to decrypt ciphertexts which _they have sent_ (i.e. "outgoing" data from the pov of the sender (and of the recipient, since they're the same person in this case)). This is useful if the user's DB is wiped, and they need to sync from scratch (starting with only $\sk$). |
$\rivskm$ | h(0x05, $\sk$) | randomising secret key, for incoming viewing keys | PXE* | This "randomising" key pair is included to enable a _shareable_ incoming viewing secret key for each app; $\shareableivskapp$. | 
$\rovskm$ | h(0x06, $\sk$) | randomising secret key, for outgoing viewing keys | PXE* | This "randomising" key pair is included to enable a _shareable_ incoming viewing secret key for each app; $\shareableovskapp$. | 
$\Npkm$ | $\nskm \cdot G$ | nullifier public key | | Only included so that other people can derive the user's address from some public information, in such a way that it's tied to the user's $\nskm$. 
$\Tpkm$ | $\tskm \cdot G$ | tagging public key | | The "tagging" key pair can be used to flag "this ciphertext is for you", without requiring decryption. |
$\Ivpkm$ | $\ivskm \cdot G$ | incoming viewing public key | | A 'sender' can use this public key to derive an app-siloed randomised incoming viewing key $\shareableIvpkapp$, which can then be used to derive an ephemeral symmetric encryption key, to encrypt a plaintext for some recipient. The data is "incoming" from the pov of the recipient. |
$\Ovpkm$ | $\ovskm \cdot G$ | outgoing viewing public key | | A user can use this key to derive an ephemeral encryption key, to encrypt data _for themselves_.
$\Rivpkm$ | $\rivskm \cdot G$ | randomising public key | | This "randomising" key pair is included to enable a _shareable_ incoming viewing secret key for each app; $\shareableivskapp$. |
$\Rovpkm$ | $\rovskm \cdot G$ | randomising public key | | This "randomising" key pair is included to enable a _shareable_ outgoing viewing secret key for each app; $\shareableovskapp$. |
||||||
$\shareableivskm$ | $\ivskm + \rivskm$ | master randomised incoming viewing secret key | PXE/ T3P | I'm not sure why I derived _master_ shareable incoming viewing keys. Perhaps a user might want to provide 3rd party access to every app they use? There will definitely be a need to encrypt the `contract_address` with $\Ivpkm$ (as a ciphertext header), as a way of efficiently conveying which siloed app keys to use to decrypt the rest of the ciphertext. But sharing this info with a 3rd party would leak all app activity. A 3rd party could instead not be given the master key, and they'd need to brute-force find the relevant siloed key (out of those they possess for the user) to use to decrypt the ciphertext. Anyway, the option is there.| 
$\shareableIvpkm$ | $\Ivpkm + \Rivpkm$<br />$= \shareableivskm \cdot G$ | master randomised incoming viewing public key |
$\shareableovskm$ | $\ovskm + \rovskm$ | master randomised incoming viewing secret key | PXE/ T3P | | 

> \*These keys could also be safely passed into the Kernel circuit, but there's no immediately obvious use, so "K" has been omitted, to make design intentions clearer.


## Address

| Key | Derivation | Name | Comments |
|---|---|---|---|
$\codehash$ | h(bytecode, constructor\_args, salt) |
$\address$ | h($\Npkm$, $\Tpkm$, $\Ivpkm$, $\Ovpkm$, $\Rivpkm$, $\Rovpkm$, $\codehash$) | address | This isn't an optimised derivation. It's just one that works. |


## Derive siloed keys

### Normal (non-hardened), app-siloed key derivations:

| Key | Derivation | Name | Where? | Comments |
|---|---|---|---|---|
$\happL$ | h($\address$, app\_address) | normal siloing key for app-specific keypair derivations | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation.<br />Note: the "L" is a lingering artifact carried over from the BIP-32 notation (where a 512-bit hmac output is split into a left and a right part), but notice there is no corresponding "R"; as a protocol simplification we do not derive BIP-32 chain codes.  |
$\happiv$ | h(0x03, $\happL$) | normal siloing key for an app-specific incoming viewing keypair | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation. |
$\happriv$ | h(0x05, $\happL$) | normal siloing key for an app-specific randomising keypair | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation. |
|||||
$\ivskapp$ | $\happiv + \ivskm$ | app-siloed incoming viewing secret key | PXE*, <br />Not App |
$\rivskapp$ | $\happriv + \rivskm$ | app-siloed randomising secret key | PXE*, <br />Not App |
$\Ivpkapp$ | $\happiv \cdot G + \Ivpkm = \ivskapp \cdot G$ | app-siloed incoming viewing public key |
$\Rivpkapp$ | $\happriv \cdot G + \Rivpkm = \rivskapp \cdot G$ | app-siloed randomising public key |
||||||
$\shareableivskapp$ | $\ivskapp + \rivskapp$ | app-siloed shareable/randomised incoming viewing secret key | PXE, T3P, App |Shareable with a trusted 3rd party. The $\ivskapp$ alone could be used by an adversary to reverse-derive the $\ivskm$ master key, which would give view access to all apps' keys. Randomisation, with $\rivskapp$, prevents this.<br />This has a further important use: it can be safely passed _into_ the app's circuit(s) in use cases which require proof of attempted decryption (e.g. negative reputation examples), (but note: $\ivskapp$ and $\rivskapp$ MUST NOT be passed into an app circuit). |
$\shareableIvpkapp$ | $\Ivpkapp + \Rivpkapp = \shareableivskapp \cdot G$ | app-siloed shareable/randomised incoming viewing public key | | It is this publicly-derivable public key which should be used by senders to derive an ephemeral symmetric encryption key, to encrypt a plaintext for some recipient. (The data is "incoming" from the pov of the recipient).<br />The non-randomised $\Ivpkapp$ should not be used for encryption in applications where a user might wish to have the option of providing some _trusted_ 3rd party with the ability to view that user's incoming data. |


> \*These keys could also be safely passed into the Kernel circuit, but there's no immediately obvious use, so "K" has been omitted, to make design intentions clearer.


### Hardened, app-siloed key derivations:

> Note: these derivations are definitely subject to change. It's a balance between keeping the kernel circuit simple, and keeping the key derivation scheme simple, with few constraints.

> NOTE: I have a better idea, but it would get rid of the $\Nkapp$. It's debatable whether the ability to share when a note has been nullified is useful in isolation, versus sharing the outgoing viewing key...


#### OPTION 1

| Key | Derivation | Name | Where? | Comments |
|---|---|---|---|---|
$\nskapp$ | $h(\nskm, \text{app\_address})$ | app-siloed nullifier secret key | PXE, K, App | Hardened; so only derivable by the owner of the master nullifier secret key. Hardened so as to enable the $\nskapp$ to be passed into an app circuit (without the threat of $\nskm$ being reverse-derivable). Only when a public key needs to be derivable by the general public is a normal (non-hardened) key used.<br />Deviates from 'conventional' hardened BIP-32-style derivation significantly, to reduce complexity and as an optimisation. |
$\Nkapp$ | $h(\nskapp)$ | Shareable nullifier key | PXE, K, T3P, App| If an app developer thinks some of their users might wish to have the option to enable some _trusted_ 3rd party to see when a particular user's notes are nullified, then this nullifier key might be of use. This $\Nkapp$ can be used in a nullifier's preimage, rather than $\nskapp$ in such cases, to enable said 3rd party to brute-force identify nullifications.<br />Note: this would not enable a 3rd party to view the contents of any notes; knowledge of the $\shareableivskapp$ would be needed for that.<br />Note: this is not a "public" key, since it must not be shared with the public. |
$\ovskapp$ | $h(\ovskm, \text{app\_address})$ |
$\shareableovskapp$ | $\ovskapp + \rovskapp$ | | | Note: Do we need a _different_ piece of randomness ($\rovskapp$) to compute $\shareableovskapp$, from the randomness ($\rivskapp$) used to compute $\shareableivskapp$.<br />Otherwise (if the same randomness $r$ was used for both derivations), a trusted 3rd party who'd been given both $\shareableovskapp$ and $\shareableivskapp$ wouldn've been able to compute $\shareableivskapp - \shareableovskapp = \ivskapp + r - \ovskapp - r = \ivskapp - \ovskapp$. And learning the difference of two secret keys seems like an unacceptable amount of leakage. |


#### OPTION 2

| Key | Derivation | Name | Where? | Comments |
|---|---|---|---|---|
$\happL$ | h($\address$, app\_address) | normal siloing key for app-specific keypair derivations | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation.<br />Note: the "L" is a lingering artifact carried over from the BIP-32 notation (where a 512-bit hmac output is split into a left and a right part), but notice there is no corresponding "R"; as a protocol simplification we do not derive BIP-32 chain codes.  |
$\happn$ | h(0x01, $\happL$) | normal siloing key for an app-specific nullifier keypair | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation. |
|
$\nskapp$ | $\happn + \nskm$ | normal (non-hardened) app-siloed nullifier secret key | PXE, K, <br />Not App |
$\Npkapp$ | $\happn \cdot G + \Npkm = \nskapp \cdot G$ | normal (non-hardened) app-siloed nullifier public key |
$\Nkapp$ | $\nskapp \cdot H$ | | PXE, K, App, T3P| For some point $H$ which is independent from $G$. TODO: validate that this is safe. |


## "Handshaking" (deriving a sequence of tags)

### Deriving a sequence of tags between Alice and Bob across all apps (at the 'master key' level)

:::warning
This glosses over the problem of ensuring Bob always uses the next tag in the sequence, and doesn't repeat or skip tags. See Phil's docs for further discussion on this topic.
:::

For Bob to derive a shared secret for Alice:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\esk_{hs}$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key, for handshaking | $hs$ = handshake.
$\Epk_{hs}$ | $\esk_{hs} \cdot G$ | Ephemeral public key, for handshaking |
$\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}$ | $\esk_{hs} \cdot \shareableIvpkm$ | Shared secret, for tagging | Here, we're illustrating the derivation of a shared secret (for tagging) using _master_ keys.<br />App developers could instead choose to use an app-specific key $\shareableIvpkapp$. See the next section.


Having derived a Shared Secret, Bob can now share it with Alice as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Taghs$ | $\esk_{hs} \cdot \Tpkm$ | Handshake message identification tag | Note: the tagging public key $\Tpkm$ exists as an optimisation, seeking to make brute-force message identification as fast as possible. In many cases, handshakes can be performed offchain via traditional web2 means, but in the case of on-chain handshakes, we have no preferred alternative over simply brute-force attempting to reconcile every 'Handshake message identification tag'. Note: this optimisation reduces the recipient's work by 1 cpu-friendly hash per message (at the cost of 255-bits to broadcast a compressed encoding of $\Taghs$). We'll need to decide whether this is the right speed/communication trade-off. | 
$\payload$ | [$\Taghs$, $\Epk_{hs}$] | Payload | This can be broadcast via L1.<br />Curve points can be compressed in the payload. |


Alice can identify she is the indended the handshake recipient as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Taghs$ | $\tskm \cdot \Epk_{hs}$ | Handshake message identification tag | Alice can extract $\Taghs$ and $\Epk_{hs}$ from the $\payload$ and perform this scalar multiplication on _every_ handshake message. If the computed $\Taghs$ value matches that of the $\payload$, then the message is indented for Alice.<br />Clearly, handshake transactions will need to be identifiable as such (to save Alice time), e.g. by revealing the contract address of some canonical handshaking contract alongside the $\payload$.<br />Recall: this step is merely an optimisation, to enable Alice to do a single scalar multiplication before moving on (in cases where she is not the intended recipient). |


If Alice successfully identifies that she is the indended the handshake recipient, she can proceed with deriving the shared secret (for tagging) as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}$ | $\shareableivskm \cdot \Epk_{hs}$ | Shared secret, for tagging |  |


A sequence of tags can then be derived by both Alice and Bob as:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\tagg_{m,i}^{Bob \rightarrow Alice}$ | $h(\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}, i)$ | The i-th tag in the sequence. |  |


This tag can be used as the basis for note retreival schemes. Each time Bob sends Alice a $\ciphertext$, he can attach the next unused $\tagg_{m,i}^{Bob \rightarrow Alice}$ in the sequence. Alice - who is also able to derive the next $\tagg_{m,i}^{Bob \rightarrow Alice}$ in the sequence - can make privacy-preserving calls to a server, requesting the $\ciphertext$ associated with a particular $\tagg_{m,i}^{Bob \rightarrow Alice}$.

> The colour key isn't quite clear for $\tagg_{m,i}^{Bob \rightarrow Alice}$. It will be a publicly-broadcast piece of information, but no one should learn that it relates to Bob nor Alice (except perhaps some trusted 3rd party whom Alice has entrusted with her $\shareableivskm$).

> TODO: Prevent spam (where a malicious user could observe the emitted tag $\tagg_{m,i}^{Bob \rightarrow Alice}$, and re-emit it many times via some other app-contract). Perhaps this could be achieved by emitting the tag as a nullifier (although this would cause state bloat).

> TODO: Bob can encrypt a record of this handshake, for himself, using his outgoing viewing key.

### Deriving a sequence of tags between Alice and Bob for a single app (at the 'app key' level)

For Bob to derive a shared secret for Alice:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\esk_{hs}$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key, for handshaking | $hs$ = handshake.
$\Epk_{hs}$ | $\esk_{hs} \cdot G$ | Ephemeral public key, for handshaking |
$\sharedsecret_{app,tagging}^{Bob \rightarrow Alice}$ | $\esk_{hs} \cdot \shareableIvpkapp$ | Shared secret, for tagging | Note: derivation of an app-specific tagging secret using $\shareableIvpkapp$ would enable a trusted 3rd party (if entrusted with $\shareableivskapp$) to identify Alice's notes more quickly, by observing the resulting $\tagg_{app,i}^{Bob \rightarrow Alice}$ values which would accompany each $\ciphertext$. 


Having derived a Shared Secret, Bob can now share it with Alice as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Taghs$ | $\esk_{hs} \cdot \Tpkm$ | Handshake message identification tag | Note: the tagging public key $\Tpkm$ exists as an optimisation, seeking to make brute-force message identification as fast as possible. In many cases, handshakes can be performed offchain via traditional web2 means, but in the case of on-chain handshakes, we have no preferred alternative over simply brute-force attempting to reconcile every 'Handshake message identification tag'. Note: this optimisation reduces the recipient's work by 1 cpu-friendly hash per message (at the cost of 255-bits to broadcast a compressed encoding of $\Taghs$). We'll need to decide whether this is the right speed/communication trade-off.<br />Note also: the _master_ tagging key $\Tpkm$ is being used in this illustration, rather than some app-specific tagging key, to make this message identification process most efficient (otherwise the user would have to re-scan all handshakes for every app they use). |
$\esk$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key, for encryption | TODO: perhaps just one ephemeral keypair could be used? |
$\Epk$ | $\esk \cdot G$ | Ephemeral public key, for encryption |
$\sharedsecret_{m,header}$ | $\esk \cdot \shareableIvpkm$ | Shared secret, for encrypting the ciphertext header. | The _master_ incoming viewing key is used here, to enable Alice to more-easily discover which contract address to use, and hence which app-specific $\shareableivskapp$ to use to ultimately derive the app-specific tag. |
$\hmencheader$ | h(0x01, $\sharedsecret_{m,header}$) | Incoming encryption key |
$\ciphertextheader$ | $enc_{\hmencheader}^{\shareableIvpkm}$(app_address)
$\payload$ | [$\Taghs$, $\Epk_{hs}$, $\Epk$, $\ciphertextheader$] | Payload | This can be broadcast via L1.<br />Curve points can be compressed in the payload. |


Alice can identify she is the indended the handshake recipient as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Taghs$ | $\tskm \cdot \Epk_{hs}$ | Handshake message identification tag | Alice can extract $\Taghs$ and $\Epk_{hs}$ from the $\payload$ and perform this scalar multiplication on _every_ handshake message. If the computed $\Taghs$ value matches that of the $\payload$, then the message is indented for Alice.<br />Clearly, handshake transactions will need to be identifiable as such (to save Alice time), e.g. by revealing the contract address of some canonical handshaking contract alongside the $\payload$.<br />Recall: this step is merely an optimisation, to enable Alice to do a single scalar multiplication before moving on (in cases where she is not the intended recipient). |


If Alice successfully identifies that she is the indended the handshake recipient, she can proceed with deriving the shared secret (for tagging) as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\sharedsecret_{m,header}$ | $\shareableivskm \cdot \Epk$ | Shared secret, for encrypting the ciphertext header |
$\hmencheader$ | h(0x01, $\sharedsecret_{m,header}$) | Incoming encryption key |
app_address | $decrypt_{\hmencheader}^{\shareableivskm}(\ciphertextheader)$ |
$\shareableivskapp$ | See derivations above. Use the decrypted app_address. | app-specific shareable/randomised incoming viewing secret key |
$\sharedsecret_{app,tagging}^{Bob \rightarrow Alice}$ | $\shareableivskapp \cdot \Epk_{hs}$ | Shared secret, for tagging |  |


A sequence of tags can then be derived by both Alice and Bob as:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\tagg_{app,i}^{Bob \rightarrow Alice}$ | $h(\sharedsecret_{app,tagging}^{Bob \rightarrow Alice}, i)$ | The i-th tag in the sequence. |  |


This tag can be used as the basis for note retreival schemes. Each time Bob sends Alice a $\ciphertext$ **for this particular app**, he can attach the next unused $\tagg_{app,i}^{Bob \rightarrow Alice}$ in the sequence. Alice - who is also able to derive the next $\tagg_{app,i}^{Bob \rightarrow Alice}$ in the sequence - can make privacy-preserving calls to a server, requesting the $\ciphertext$ associated with a particular $\tagg_{app,i}^{Bob \rightarrow Alice}$.

> TODO: Bob can encrypt a record of this handshake, for himself, using his outgoing viewing key.





### Deriving a sequence of tags from Bob to himself across all apps (at the 'master key' level)

The benefit of Bob deriving a sequence of tags for himself, is that he can re-sync his _outgoing_ transaction data more quickly, if he ever needs to in future.

There are many ways to do this:
- Copy the approach used to derive a sequence of tags between Bob and Alice (but this time do it between Bob and Bob, and use Bob's outgoing keys).
	- This would require a small modification, since we don't have app-siloed outgoing viewing _public_ keys (merely as an attempt to simplify the protocol...)
- Generate a very basic sequence of tags $\tagg_{app, i}^{Bob \rightarrow Bob} = h(\shareableovskapp, i)$ (at the app level) and $\tagg_{m, i}^{Bob \rightarrow Bob} = h(\shareableovskm, i)$.
	- Note: In the case of deriving app-specific sequences of tags, Bob might wish to also encrypt the app_address as a ciphertext header (and attach a master tag $\tagg_{m, i}^{Bob \rightarrow Bob}$), to remind himself of the apps that he should derive tags _for_.
- Lots of other approaches.






## Derive diversified address

A Diversified Address can be derived from Alice's keys, to enhance Alice's transaction privacy. If Alice's counterparties' databases are compromised, it enables Alice to retain privacy from such leakages. Basically, Alice must personally derive and provide Bob and Charlie with random-looking addresses (for Alice). Because Alice is the one deriving these Diversified Addresses (they can _only_ be derived by Alice), if Bob and Charlie chose to later collude, they would not be able to convince each-other that they'd interacted with Alice.

This is not to be confused with 'Stealth Addresses', which 'flip' who derives: Bob and Charlie would each derive a random-looking Stealth Address for Alice. Alice would then discover her new Stealth Addresses through decryption.

> All of the key information below is Alice's

Alice derives the following 'diversified', app-specific public keys, and sends them to Bob:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ |diversifier |
$\Gd$ | $\d \cdot G$ | diversified generator |
$\Ivpkappd$ | $\ivskapp \cdot \Gd$ | Diversified, app-siloed incoming viewing public key |
$\Rivpkappd$ | $\rivskapp \cdot \Gd$ | Diversified, app-siloed randomizing public key |

Bob can then compute Alice's Diversified Public Key as:

| Thing | Derivation | Name |
|---|---|---|
$\shareableIvpkappd$ | $\Ivpkappd + \Rivpkappd$ | Diversified, app-siloed shareable/randomised incoming viewing public key |
$\Pkappd$ | $\shareableIvpkappd$ | Alias: "Alice's Diversified Public Key" |


> Note: _master_ keys can also be diversified; just replace $app$ with $m$ in the above table of definitions. Some data (such as an app address) might need to be encrypted into a 'ciphertext header' with a master key (so as to enable the recipient to efficiently discover which app a ciphertext originated from, so they may then derive the correct siloed keys to use to decrypt the ciphertext).


> Notice: when $\d = 1$, $\Ivpkappd = \Ivpkapp$. Often, it will be unncessary to diversify the below data, but we keep $\d$ around for the most generality.


---






## Derive stealth address

> All of the key information below is Alice's

For Bob to derive a Stealth Address for Alice, Bob derives:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | Given by Alice | (Diversifier) | Remember, in most cases, $\d=1$ is sufficient.
$\Gd$ | $\d \cdot G$ | (Diversified) generator | Remember, when $\d = 1$, $\Gd = G$.
$\esk$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkd$ | $\esk \cdot \Gd$ | (Diversified) Ephemeral public key |
$\shareableIvpkappd$ | $\Ivpkappd + \Rivpkappd$ | (Diversified) App-siloed shareable/randomised incoming viewing public key |
$\sharedsecret_{app, stealth}$ | $\esk \cdot \shareableIvpkappd$ | Shared secret |
$\hstealth$ | h(0x02, $\sharedsecret_{app, stealth}$) | Stealth key |
$\Ivpkappdstealth$ | $\hstealth \cdot \Gd + \Ivpkappd$ | (Diversified) Stealth viewing public key |
$\Pkappdstealth$ | $\Ivpkappdstealth$ | Alias: "Alice's Stealth Public Key" |


Having derived a Stealth Address for Alice, Bob can now share it with Alice as follows:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\tagg_{m, i}^{Bob \rightarrow Alice}$ | See earlier in this doc. | | Derive the next tag in the $Bob\rightarrow Alice$ sequence.<br />Note: we illustrate with a _master_ tag sequence, but an app-specific tag sequence could also be used (in which case an encryption of the app_address in a ciphertext header wouldn't be required; it could just be inferred from the tag used). |
$\sharedsecret_{m,header}$ | $\esk \cdot \shareableIvpkm$ | | TODO: we might need to use a different ephemeral keypair from the one used to derive the stealth address. |
$\hmencheader$ | h(0x01, $\sharedsecret_{m,header}$) |
$\ciphertextheader$ | $enc^{\shareableIvpkm}_{\hmencheader}$(app\_address) | | TODO: diversify this? |
$\payload$ | [$\tagg_{m, i}^{Bob \rightarrow Alice}$, $\ciphertextheader$, $\Epkd$] |


Alice can learn about her new Stealth Address as follows. First, she would identify the transaction has intended for her, either by observing $\tagg_{m, i}^{Bob \rightarrow Alice}$ on-chain herself (and then downloading the rest of the payload which accompanies the tag), or by making a privacy-preserving request to a server, to retrieve the payload which accompanies the tag. Assuming the $\payload$ has been identified as Alice's, we proceed:

| Thing | Derivation | Name |
|---|---|---|
$\sharedsecret_{m,header}$ | $\shareableivskm \cdot \Epkd$ | Shared secret, for encrypting the ciphertext header |
$\hmencheader$ | h(0x01, $\sharedsecret_{m,header}$) | Incoming encryption key |
app_address | $decrypt_{\hmencheader}^{\shareableivskm}(\ciphertextheader)$ |
$\shareableivskapp$ | See derivations above. Use the decrypted app_address. | app-specific shareable/randomised incoming viewing secret key |
$\sharedsecret_{app, stealth}$ | $\shareableivskapp \cdot \Epkd$ |
$\hstealth$ | h(0x02, $\sharedsecret_{app, stealth}$) |
$\ivskappstealth$ | $\hstealth + \ivskapp$ |
$\Ivpkappdstealth$ | $\ivskappstealth \cdot \Gd$ |
$\Pkappdstealth$ | $\Ivpkappdstealth$ | Alias: "Alice's Stealth Public Key" |


Data can be encrypted to Alice's stealth address, BUT, if an app developer wishes to continue to give their users the option of enabling a trusted 3rd party to decrypt their incoming data, this stealth address must be randomised. Bob can derive a shareable/randomised stealth address as follows:

| Thing | Derivation | Name |
|---|---|---|
$\shareableIvpkappdstealth$ | $\Ivpkappdstealth + \Rivpkappd$

Alice can derive the same shareable/randomised stealth address as follows:

| Thing | Derivation | Name |
|---|---|---|
$\shareableivskappstealth$ | $(\hstealth + \ivskapp) + \rivskapp$<br />$= \ivskappstealth + \rivskapp$
$\shareableIvpkappdstealth$ | $\shareableivskappstealth \cdot \Gd$


## Derive nullifier

There are two options, alluding to the two options (above) for deriving siloed nullifier keys.

### OPTION 1

Let's assume a developer wants a nullifier of a note to be derived as:

`nullifier = h(note_hash, nullifier_key);`

... where the `nullifier_key` ($\Nkapp$) belongs to the 'owner' of the note, and where the 'owner' is some $\address$.

Here's how an app circuit could constrain the nullifier key to be correct:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Nkapp$ | h($\nskapp$) | App-siloed nullifier key | Recall an important point: the app circuit MUST NOT be given $\nskm$. Indeed, $\nskapp$ is derived (see earlier) as a _hardened_ child of $\nskm$, to prevent $\nskm$ from being reverse-derived by a malicious circuit. The linking of $\nskapp$ to $\nskm$ is deferred to the kernel circuit (which can be trusted moreso than an app).<br />Recall also: $\Nkapp$ is used (instead of $\nskapp$) solely as a way of giving the user the option of sharing $\Nkapp$ with a trusted 3rd party, to give them the ability to view when a note has been nullified (although I'm not sure how useful this is, given that it would require brute-force effort from that party to determine which note hash has been nullified, with very little additional information).<br />TODO: consider whether this $\Nkapp$ is needed. |
`nullifier` | h(note_hash, $\Nkapp$) |
$\address$ | h($\Npkm$, $\Tpkm$, $\Rivpkm$, $\Ivpkm$, $\Ovpkm$, $\codehash$) | address | Proof that the $\Npkm$ belongs to the note owner's $\address$.<br />This isn't an optimised derivation. It's just one that works. |

The app circuit exposes, as public inputs, a "nullifier key validation request":

| Thing | Derivation | Name | Comments |
|---|---|---|---|
`nullifier_key_validation_request` | app_address: app_address,<br />hardened_child_sk: $\nskapp$,<br />claimed_parent_pk: $\Npkm$ |


The kernel circuit can then validate the request (having been given $\nskm$ as a private input to the kernel circuit):

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\nskapp$ | $h(\nskm, \text{app\_address})$ |
$\Npkm$ | $\nskm \cdot G$ | nullifier public key |
| | | | Copy-constrain $\nskm$ with $\nskm$. | 


If the kernel circuit succeeds in these calculations, then the $\Nkapp$ has been validated as having a known secret key, and belonging to the $\address$.

> Note: It's ugly. I don't like having to pass such a specific and obscure request to the kernel circuit.

### OPTION 2

Let's assume a developer wants a nullifier of a note to be derived as:

`nullifier = h(note_hash, nullifier_key);`

... where the `nullifier_key` ($\Nkapp$) belongs to the 'owner' of the note, and where the 'owner' is some $\address$.

Here's how an app circuit could constrain the nullifier key to be correct:

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\happL$ | h($\address$, app\_address) | normal siloing key for app-specific keypair derivations | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation.<br />Note: the "L" is a lingering artifact carried over from the BIP-32 notation (where a 512-bit hmac output is split into a left and a right part), but notice there is no corresponding "R"; as a protocol simplification we do not derive BIP-32 chain codes.  |
$\happn$ | h(0x01, $\happL$) | normal siloing key for an app-specific nullifier keypair | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation. |
||||
$\Npkapp$ | $\happn \cdot G + \Npkm$ | normal (non-hardened) app-siloed nullifier public key |
||||
`nullifier` | h(note_hash, $\Nkapp$) | | $\Nkapp$ is exposed as a public input, and linked to the $\Npkapp$ via the kernel circuit. |
$\address$ | h($\Npkm$, $\Tpkm$, $\Rivpkm$, $\Ivpkm$, $\Ovpkm$, $\codehash$) | address | Proof that the $\Npkm$ belongs to the note owner's $\address$.<br />This isn't an optimised derivation. It's just one that works. |

> Recall an important point: the app circuit MUST NOT be given $\nskm$ nor $\nskapp$ in this option. Since $\nskapp$ is a normal (non-hardened) child, $\nskm$ could be reverse-derived by a malicious circuit. The linking of $\nskapp$ to $\Npkm$ is therefore deferred to the kernel circuit (which can be trusted moreso than an app).<br />Recall also: $\Nkapp$ is used (instead of $\nskapp$) solely as a way of giving the user the option of sharing $\Nkapp$ with a trusted 3rd party, to give them the ability to view when a note has been nullified (although I'm not sure how useful this is, given that it would require brute-force effort from that party to determine which note hash has been nullified, with very little additional information).<br />TODO: consider whether this $\Nkapp$ is needed.

The app circuit exposes, as public inputs, a "proof of knowledge of discrete log, request":

| Thing | Derivation | Name | Comments |
|---|---|---|---|
`pokodl_request` | [{<br />&nbsp;Anti_log: $\Nkapp$,<br />&nbsp;Base: $H$<br />},{<br />&nbsp;Anti_log: $\Npkapp$,<br />&nbsp;Base: $G$<br />}] | Proof of knowledge of discrete log, request. |


> Recall: $log_{base}(anti\_logarithm) = logarithm$ (TODO: this terminology might be confusing, especially since we're using additive notation).

The _kernel_ circuit can then validate the `pokodl_request` (having been given the requested logarithms (in this case $\nskapp$ twice) as private inputs):

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Nkapp$ | $\nskapp \cdot H$ |
$\Npkapp$ | $\nskapp \cdot G$ |
| | | | Copy-constrain $\nskapp$ with $\nskapp$.<br />This constraint makes the interface to the kernel circuit a bit uglier. The `pokodl_request` now needs to convey "and constrain the discrete logs of these two pokodl requests to be equal". Ewww. | 



If the kernel circuit succeeds in these calculations, then the $\Nkapp$ has been validated as having a known secret key, and belonging to the $\address$.


### OPTION 2, a failed optimisation attempt

<details>

A slight tidying of the interface to the kernel circuit (to avoid the 'copy constraint request') would be to make a _pairing_ request to the kernel circuit. Now, pairings are very expensive, but if GalacticGoblinHonk is capable of accumulating pairing requests and deferring them to later, it might not be too bad. It changes the request to the kernel from being "Check knowledge of these discrete logs, and optionally copy-constrain the discrete logs to be equal", to being "check these two pairings are equal". Ethereum has a precompile which does this exact request, so it's not unheard of! 

The downsides to this approach are:
- the SRS would need as many $\mathbb{G}_2$ points as $\mathbb{G}_1$ points (not sure what the SRS for Honk will include);
- $\Nkapp$ would be a large $\mathbb{G}_2$ point, which would cost more to include in a nullifier;
- **Grumpkin isn't pairing friendly so none of this option works!!!** I'm not emotionally ready to delete it yet.

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\happL$ | h($\address$, app\_address) | normal siloing key for app-specific keypair derivations | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation.<br />Note: the "L" is a lingering artifact carried over from the BIP-32 notation (where a 512-bit hmac output is split into a left and a right part), but notice there is no corresponding "R"; as a protocol simplification we do not derive BIP-32 chain codes.  |
$\happn$ | h(0x01, $\happL$) | normal siloing key for an app-specific nullifier keypair | | An intermediate step in a BIP-32-esque "normal" (non-hardened) child key derivation. |
|||||
$\Npkapp$ | $\happn \cdot G + \Npkm$ | normal (non-hardened) app-siloed nullifier public key |
|||||
`nullifier` | h(note_hash, $\Nkapp$) | | $\Nkapp$ is exposed as a public input, and linked to the $\Npkapp$ via the kernel circuit. |
$\address$ | h($\Npkm$, $\Tpkm$, $\Rivpkm$, $\Ivpkm$, $\Ovpkm$, $\codehash$) | address | Proof that the $\Npkm$ belongs to the note owner's $\address$.<br />This isn't an optimised derivation. It's just one that works. |


> Recall an important point: the app circuit MUST NOT be given $\nskm$ nor $\nskapp$ in this option. Since $\nskapp$ is a normal (non-hardened) child, $\nskm$ could be reverse-derived by a malicious circuit. The linking of $\nskapp$ to $\Npkm$ is therefore deferred to the kernel circuit (which can be trusted moreso than an app).<br />Recall also: $\Nkapp$ is used (instead of $\nskapp$) solely as a way of giving the user the option of sharing $\Nkapp$ with a trusted 3rd party, to give them the ability to view when a note has been nullified (although I'm not sure how useful this is, given that it would require brute-force effort from that party to determine which note hash has been nullified, with very little additional information).<br />TODO: consider whether this $\Nkapp$ is needed.

The app circuit exposes, as public inputs, a "pairing check request":

| Thing | Derivation | Name | Comments |
|---|---|---|---|
`pairing_request` | [[$\Npkapp$, $H$], [$G$, $\Nkapp$]] |


The _kernel_ circuit can then validate the `pairing_request`:

$e(\Npkapp, H) == e(G, \Nkapp)$, which is to check:

$e([\nskapp]_G, [1]_H) == e([1]_G, [\nskapp]_H)$, which is to check in the exponent:

$\nskapp \cdot 1 == 1 \cdot \nskapp$.

> To prove this check works, let's try to break it. A malicious actor has agency to choose whatever $\Nkapp$ they like, say $X := x \cdot H$.  
> That is, the malicious actor is attempting to double-spend, by generating a point $X$ whose discrete log isn't the $\nskapp$.
Consider, then, forwarding this `pairing_request` to the kernel circuit: [[$\Npkapp$, $H$], [$G$, $X$]].
>
> $e(\Npkapp, H) == e(G, X)$ is to check in the exponent:
>
> $\nskapp \cdot 1 == 1 \cdot x$, which cannot be satisfied unless $x = \nskapp$.

If the kernel circuit succeeds in these calculations, then the $\Nkapp$ has been validated as having the correct secret key, and belonging to the $\address$.

</details>

## Encrypt and tag an incoming message

Bob wants to send Alice a private message, e.g. the contents of a note, which we'll refer to as the $\plaintext$. Bob and Alice are using a "tag hopping" scheme to help with note discovery. Let's assume they've already handshaked to establish a shared secret $\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}$, from which a sequence of tags $\tagg_{m,i}^{Bob \rightarrow Alice}$ can be derived.

> Note: this illustration uses _master_ keys for tags, rather than app-specific keys for tags. App-specific keys for tags could be used instead, in which case a 'ciphertext header' wouldn't be needed for the 'app_address', since the address could be inferred from the tag.

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | Given by Alice | (Diversifier) | Remember, in most cases, $\d=1$ is sufficient.
$\Gd$ | $\d \cdot G$ | (Diversified) generator | Remember, when $\d = 1$, $\Gd = G$.
$\eskheader$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkdheader$ | $\eskheader \cdot \Gd$ | (Diversified) Ephemeral public key |
$\sharedsecret_{m,header}$ | $\esk_{header} \cdot \shareableIvpkm$ | Shared secret, for ciphertext header encryption | TODO: can we use the same ephemeral keypair for both the ciphertext header and the ciphertext?<br />TODO: diversify the $\shareableIvpkm$? |
$\hmencheader$ | h(0x01, $\sharedsecret_{m,header}$) |
$\ciphertextheader$ | $enc^{\shareableIvpkm}_{\hmencheader}$(app\_address) |  |  |
|||||
$\esk$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkd$ | $\esk \cdot \Gd$ | (Diversified) Ephemeral public key |
$\shareableIvpkappdstealth$ | $\Ivpkappdstealth + \Rivpkappd$ | (Diversified) (Stealth) App-siloed shareable/randomised incoming viewing public key
$\sharedsecret_{app,enc}$ | $\esk \cdot \shareableIvpkappdstealth$ | Shared secret, for ciphertext encryption |
$\happenc$ | h(0x01, $\sharedsecret_{app,enc}$) | Ephemeral incoming encryption key |
$\ciphertext$ | $enc^{\shareableIvpkappdstealth}_{\happenc}(\plaintext)$ |
$\payload$ | [$\tagg_{m, i}^{Bob \rightarrow Alice}$, $\ciphertextheader$, $\ciphertext$, $\Epkdheader$, $\Epkd$] |


Alice can learn about her new $\payload$ as follows. First, she would identify the transaction has intended for her, either by observing $\tagg_{m, i}^{Bob \rightarrow Alice}$ on-chain herself (and then downloading the rest of the payload which accompanies the tag), or by making a privacy-preserving request to a server, to retrieve the payload which accompanies the tag. Assuming the $\payload$ has been identified as Alice's, and retrieved by Alice, we proceed.

> Given that the tag in this illustration was derived from Alice's master key, the tag itself doesn't convey which app_address to use, to derive the correct app-siloed incoming viewing secret key that would enable decryption of the ciphertext. So first Alice needs to decrypt the $\ciphertextheader$ using her master key:

| Thing | Derivation | Name |
|---|---|---|
$\sharedsecret_{m,header}$ | $\shareableivskm \cdot \Epkdheader$ | Shared secret, for encrypting the ciphertext header |
$\hmencheader$ | h(0x01, $\sharedsecret_{m,header}$) | Incoming encryption key |
app_address | $decrypt_{\hmencheader}^{\shareableivskm}(\ciphertextheader)$ |
||||
$\shareableivskappstealth$ | See derivations above. Use the decrypted app_address. | app-specific shareable/randomised incoming viewing secret key |
$\sharedsecret_{app, enc}$ | $\shareableivskappstealth \cdot \Epkd$ |
$\happenc$ | h(0x01, $\sharedsecret_{app, enc}$) |
$\plaintext$ | $decrypt_{\happenc}^{\shareableivskappstealth}(\ciphertext)$ |


## Encrypt and tag an internal incoming message

TODO: describe internal key derivation

## Encrypt and tag an outgoing message

Bob wants to send himself a private message (e.g. a record of the outgoing notes that he's created for other people) which we'll refer to as the $\plaintext$. Let's assume Bob has derived a sequence of tags $\tagg_{m,i}^{Bob \rightarrow Alice}$ for himself (see earlier).

> Note: this illustration uses _master_ keys for tags, rather than app-specific keys for tags. App-specific keys for tags could be used instead, in which case a 'ciphertext header' wouldn't be needed for the 'app_address', since the address could be inferred from the tag.

> Note: rather than copying the 'shared secret' approach of Bob sending to Alice, we can cut a corner (because Bob is the sender and recipient, and so knows his own secrets).

> Note: if Bob has sent a private message to Alice, and he also wants to send himself a corresponding message:
> - he can likely re-use the ephemeral keypairs for himself.
> - he can include $\esk$ in the plaintext that he sends to himself, as a way of reducing the size of his $\ciphertext$ (since the $\esk$ will enable him to access all the information in the ciphertext that was sent to Alice).
> - TODO: can we use a shared public key to encrypt the $\ciphertextheader$, to reduce duplicated broadcasting of encryptions of the app_address?
>     - E.g. derive a shared secret, hash it, and use that as a shared public key?

> Note: the violet symbols should actually be orange here.

| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | Given by Alice | (Diversifier) | Remember, in most cases, $\d=1$ is sufficient.
$\Gd$ | $\d \cdot G$ | (Diversified) generator | Remember, when $\d = 1$, $\Gd = G$.
$\eskheader$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkdheader$ | $\eskheader \cdot \Gd$ | (Diversified) Ephemeral public key |
$\hmencheader$ | h(0x01, $\shareableovskm$, $\Epkdheader$) |
$\ciphertextheader$ | $enc_{\hmencheader}$(app\_address) |  |  |
|||||
$\esk$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkd$ | $\esk \cdot \Gd$ | (Diversified) Ephemeral public key |
$\happenc$ | h(0x01, $\shareableovskm$, $\Epkd$) | Ephemeral incoming encryption key |
$\ciphertext$ | $enc_{\happenc}(\plaintext)$ |
$\payload$ | [$\tagg_{m, i}^{Bob \rightarrow Bob}$, $\ciphertextheader$, $\ciphertext$, $\Epkdheader$, $\Epkd$] |


Alice can learn about her new $\payload$ as follows. First, she would identify the transaction has intended for her, either by observing $\tagg_{m, i}^{Bob \rightarrow Alice}$ on-chain herself (and then downloading the rest of the payload which accompanies the tag), or by making a privacy-preserving request to a server, to retrieve the payload which accompanies the tag. Assuming the $\payload$ has been identified as Alice's, and retrieved by Alice, we proceed.

> Given that the tag in this illustration was derived from Alice's master key, the tag itself doesn't convey which app_address to use, to derive the correct app-siloed incoming viewing secret key that would enable decryption of the ciphertext. So first Alice needs to decrypt the $\ciphertextheader$ using her master key:

| Thing | Derivation | Name |
|---|---|---|
$\hmencheader$ | h(0x01, $\shareableovskm$, $\Epkdheader$) |  |
app_address | $decrypt_{\hmencheader}(\ciphertextheader)$ |
||||
$\shareableovskapp$ | See derivations above. Use the decrypted app_address. | |
$\happenc$ | h(0x01, $\shareableovskm$, $\Epkd$) |
$\plaintext$ | $decrypt_{\happenc}(\ciphertext)$ |


> TODO: how does a user validate that they have successfully decrypted a ciphertext? Is this baked into ChaChaPoly1035, for example?



## Acknowledgements

Much of this is inspired by the [ZCash Sapling and Orchard specs](https://zips.z.cash/protocol/protocol.pdf). 