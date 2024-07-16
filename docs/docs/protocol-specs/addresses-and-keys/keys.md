---
title: Default Keys Specification
description: Specification for default privacy keys format and derivation, and nullifier derivation.
---

## Cheat Sheet

import Image from "@theme/IdealImage";

The protocol does not enforce the usage of any of the following keys, and does not enforce the keys to conform to a particular length or algorithm. Users are expected to pick a set of keys valid for the encryption and tagging precompile they choose for their account.

<!-- prettier-ignore -->
| Cat. | Key | Derivation | Link |
|---|---|---|---|
| Seed | $\seed$ | $$\stackrel{\$}{\leftarrow} \mathbb{F}$$ | [Seed](#seed) |
| | $\sk$ | $$\stackrel{\$}{\leftarrow} \mathbb{F}$$ | [Master Secret Key](#master-secret-key) |
|||||
| Master Secret Keys | $\nskm$ | $\text{poseidon2}(\text{``az\_nsk\_m''}, \sk)$ | [Master Nullifier Secret Key](#master-nullifier-secret-key) |
| | $\ovskm$ | $\text{poseidon2}(\text{``az\_ovsk\_m''}, \sk)$ | [Master Outgoing Viewing Secret Key](#master-outgoing-viewing-secret-key) |
| | $\ivskm$ | $\text{poseidon2}(\text{``az\_ivsk\_m''}, \sk)$ | [Master Incoming Viewing Secret Key](#master-incoming-viewing-secret-key) |
| | $\tskm$ | $\text{poseidon2}(\text{``az\_tsk\_m''}, \sk)$ | [Master Tagging Secret Key](#master-tagging-secret-key) |
|||||
| Master Public Keys | $\Npkm$ | $\nskm \cdot G$ | [Master Nullifier Public Key](#master-nullifier-public-key) |
| | $\Ovpkm$ | $\ovskm \cdot G$ | [Master Outgoing Viewing Public Key](#master-outgoing-viewing-public-key) |
| | $\Ivpkm$ | $\ivskm \cdot G$ | [Master Incoming Viewing Public Key](#master-incoming-viewing-public-key) |
| | $\Tpkm$ | $\tskm \cdot G$ | [Master Tagging Public Key](#master-tagging-public-key) | 
||||
| Hardened App-Siloed Secret Keys | $\nskapp$ | $\text{poseidon2}(\text{``az\_nsk\_app''}, \text{app\_address}, \nskm)$ | [Hardened, App-siloed Nullifier Secret Key](#app-siloed-nullifier-secret-key) |
| | $\ovskapp$ | $\text{poseidon2}(\text{``az\_ovsk\_app''}, \text{app\_address}, \ovskm)$ | [Hardened, App-siloed Outgoing Viewing Secret Key](#app-siloed-outgoing-viewing-secret-key) |
|||||
| Other App-siloed Keys| $\Nkapp$ | $\text{poseidon2}(\text{``az\_nk\_app''}, \nskapp)$ | [App-siloed Nullifier Key](#app-siloed-nullifier-key) |


## Colour Key

- $\color{green}{green}$ = Publicly shareable information.
- $\color{red}{red}$ = Very secret information. A user MUST NOT share this information.
  - TODO: perhaps we distinguish between information that must not be shared to prevent theft, and information that must not be shared to preserve privacy?
- $\color{orange}{orange}$ = Secret information. A user MAY elect to share this information with a _trusted_ 3rd party, but it MUST NOT be shared with the wider world.
- $\color{violet}{violet}$ = Secret information. Information that is shared between a sender and recipient (and possibly with a 3rd party who is entrusted with viewing rights by the recipient).

## Diagrams

<!-- TODO: Update diagrams -->

:::danger
Diagram is out of date vs the content on this page
:::

<Image img={require("/img/protocol-specs/addresses-and-keys/image-5.png")} />

The red boxes are uncertainties, which are explained later in this doc.

## Preliminaries

$\mathbb{F}_r$ denotes the AltBN254 scalar field (i.e. the Grumpkin base field).

$\mathbb{F}_q$ denotes the AltBN254 base field (i.e. the Grumpkin scalar field).

Let $\mathbb{G}_{\text{Grumpkin}}$ be the Grumpkin elliptic curve group ($E(\mathbb{F}_r)$).

Let $G \in \mathbb{G}_{\text{Grumpkin}}$ be a generator point for the public key cryptography outlined below. TODO: decide on how to generate this point.

Elliptic curve operators $+$ and $\cdot$ are used to denote addition and scalar multiplication, respectively.

$\text{poseidon2}: \mathbb{F}_r^t \rightarrow \mathbb{F}$ is the Poseidon2 hash function (and $t$ can take values as per the [Poseidon2 spec](https://eprint.iacr.org/2023/323.pdf)).

Note that $q > r$. Below, we'll often define secret keys as an element of $\mathbb{F}_r$, as this is most efficient within a snark circuit. We'll then use such secret keys in scalar multiplications with Grumpkin points ($E(\mathbb{F}_r)$ whose affine points are of the form $\mathbb{F}_r \times \mathbb{F}_r$). Strictly speaking, such scalars in Grumpkin scalar multiplication should be in $\mathbb{F}_q$.  
A potential consequence of using elements of $\mathbb{F}_r$ as secret keys could be that the resulting public keys are not uniformly-distributed in the Grumpkin group, so we should check this. The distribution of such public keys will have a statistical distance of $\frac{2(q - r)}{q}$ from uniform. It turns out that $\frac{1}{2^{126}} < \frac{2(q - r)}{q} < \frac{1}{2^{125}}$, so the statistical distance from uniform is broadly negligible, especially considering that the AltBN254 curve has fewer than 125-bits of security.

## Key Derivation

### Derive Master Secret Key from Secret Key

$$
\begin{aligned}
&\text{derive\_master\_secret\_key\_from\_secret\_key}: \text{string} \times \mathbb{F}_r \to \mathbb{F}_r \\
&\text{derive\_master\_secret\_key\_from\_secret\_key}(\text{domain\_separator\_string}, \text{secret\_key}) \\
&:= \text{poseidon2}(\text{be\_string\_to\_field(domain\_separator\_string)}, \text{secret\_key})
\end{aligned}
$$

> Note: Here, $\text{poseidon2}$ is assumed to be a pseudo-random function.

### Derive Hardened App-siloed Secret Key

$$
\begin{aligned}
&\text{derive\_hardened\_app\_siloed\_secret\_key}: \text{string} \times \mathbb{F}_r \times \mathbb{F}_r \to \mathbb{F}_r \\
&\text{derive\_hardened\_app\_siloed\_secret\_key}(\text{domain\_separator\_string}, \text{app\_address}, \text{parent\_secret\_key}) \\
&:= \text{poseidon2}(\text{be\_string\_to\_field(domain\_separator\_string)}, \text{app\_address}, \text{parent\_secret\_key})
\end{aligned}
$$

> Note: Here, $\text{poseidon2}$ is assumed to be a pseudo-random function.

> Note: this deviates significantly from the 'conventional' [BIP-32 style method for deriving a "hardened child secret key"](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#private-parent-key--private-child-key), to reduce complexity and as an optimization. Such a deviation will need to be validated as secure.
> In particular:
>
> - the notion of a "chain code" has been removed;
> - the notion of an "index" has been replaced by an app_address;
> - HMAC-SHA512 has been replaced with Poseidon2. Note: we don't need a 512-bit output, since we've removed the notion of a "chain code", and so we don't need to split the output of the Poseidon2 function into two outputs.

### Derive Public Key (from Secret Key)

$$
\begin{aligned}
&\text{derive\_public\_key}: \mathbb{F}_r \to \mathbb{G}_{\text{Grumpkin}} \\
&\text{derive\_public\_key}(\text{secret\_key}) := \text{secret\_key} \cdot G
\end{aligned}
$$

## Seed

A seed secret from which all of a user's other keys may be derived. The $\seed$ can live on an offline device, such as a hardware wallet.

$$\seed \stackrel{\$}{\leftarrow} \mathbb{F}_r$$

## Master Secret Key

This $\sk$ must never enter a circuit. A user or wallet may wish to derive this $\sk$ from a cold wallet [$\seed$](#seed).

$$\sk \stackrel{\$}{\leftarrow} \mathbb{F}_r$$

> Note: Often $\sk = hash(\seed)$ for some hardware-wallet-supported hash function, would be recommended. Although, care would need to be taken if the hardware wallet doesn't support hashing directly to $\mathbb{F}_r$, since a truncated hash output could be non-uniformly distributed in $\mathbb{F}_r$.  
> For example, if the hardware wallet only supports sha256, then it would not be acceptable to compute $\sk$ as $\text{sha256}(\seed) \mod r$, since the resulting output (of reducing a 256-bit number modulo $r$) would be biased towards smaller values in $\mathbb{F_r}$. More uniformity might be achieved by instead computing $\sk$ as $( \text{sha256}(\seed, 1) || \text{sha256}(\seed, 2) ) \mod r$, for example, as a modulo reduction of a 512-bit number is closer to being uniformly distributed in $\mathbb{F_r}$.  
> This note is informal, and expert advice should be sought before adopting this approach.

## Nullifier Keys

[App-siloed Nullifier Keys](#app-siloed-nullifier-key) can be used by app developers when deriving their apps' nullifiers. By inserting some secret nullifier key into a nullifier's preimage, it makes the resulting nullifier look random, meaning observers cannot determine which note has been nullified.

> Note that not all nullifiers will require a secret key in their computation, e.g. plume nullifiers, or state variable initialization nullifiers. But the keys outlined in this section should prove useful to many app developers.

### Master Nullifier Secret Key

$$
\begin{aligned}
& \nskm \in \mathbb{F}_r \\
& \nskm = \text{derive\_master\_secret\_key\_from\_secret\_key}(\text{``az\_nsk\_m''}, \seed)
\end{aligned}
$$

> See [`derive_master_secret_key_from_secret_key`](#derive-master-secret-key-from-secret-key).

> $\nskm$ MUST NOT enter an app circuit.  
> $\nskm$ MAY enter the kernel circuit.

### Master Nullifier Public Key

The Master Nullifier Public Key is only included so that other people can derive the user's address from some public information (i.e. from $\Npkm$), in such a way that $\nskm$ is tied to the user's address.

$$
\begin{aligned}
& \Npkm \in \mathbb{G}_{\text{Grumpkin}} \\
& \Npkm = \text{derive\_public\_key}(\nskm)
\end{aligned}
$$

> See [`derive_public_key`](#derive-public-key-from-secret-key).

### App-siloed Nullifier Secret Key

The App-siloed Nullifier Secret Key is a **hardened** child key, and so is only derivable by the owner of the master nullifier secret key. It is hardened so as to enable the $\nskapp$ to be passed into an app circuit, without the threat of $\nskm$ being reverse-derivable by a malicious app. Only when an app-siloed public key needs to be derivable by the general public is a normal (non-hardened) key derivation scheme used.

$$
\begin{aligned}
& \nskapp \in \mathbb{F}_r \\
& \nskapp = \text{derive\_hardened\_app\_siloed\_secret\_key}(\text{``az\_nsk\_app''}, \text{app\_address}, \nskm)
\end{aligned}
$$

> See [`derive_hardened_app_siloed_secret_key`](#derive-hardened-app-siloed-secret-key).

### App-siloed Nullifier Key

If an app developer thinks some of their users might wish to have the option to enable some _trusted_ 3rd party to see when a particular user's notes are nullified, then this nullifier key might be of use. This $\Nkapp$ can be used in a nullifier's preimage, rather than $\nskapp$ in such cases, to enable said 3rd party to brute-force identify nullifications.

> Note: this key can be optionally shared with a trusted 3rd party, and they would not be able to derive the user's secret keys.  
> Note: knowledge of this key enables someone to identify when an emitted nullifier belongs to the user, and to identify which note hashes have been nullified.  
> Note: knowledge of this key would not enable a 3rd party to view the contents of any notes; knowledge of the $\ivsk$ / $\ovskapp$ would be needed for that.  
> Note: this is intentionally not named as a "public" key, since it must not be shared with the wider public.

$$
\begin{aligned}
& \Nkapp \in \mathbb{F}_r \\
& \Nkapp = \text{poseidon2}(\text{``az\_nk\_app''}, \nskapp)
\end{aligned}
$$

:::warning TODO
We could also have derived $\Nkapp$ as $\nskapp \cdot G$, but this would have resulted in a Grumpkin point, which is more cumbersome to insert into the preimage of a nullifier. We might still change our minds to adopt this scalar-multiplication approach, since it might enable us to prove knowledge of $\nskm$ to the app circuit without having to add key derivation logic to the kernel circuit.
:::

## Outgoing Viewing Keys

[App-siloed Outgoing Viewing Secret Keys](#app-siloed-outgoing-viewing-secret-key) can be used to derive ephemeral symmetric encryption keys, which can then be used to encrypt/decrypt data which _the user has created for their own future consumption_. I.e. these keys are for decrypting "outgoing" data from the pov of a sender. This is useful if the user's DB is wiped, and they need to sync from scratch (starting with only $\seed$).

### Master Outgoing Viewing Secret Key

$$
\begin{aligned}
& \ovskm \in \mathbb{F}_r \\
& \ovskm = \text{derive\_master\_secret\_key\_from\_seed}(\text{``az\_ovsk\_m''}, \seed)
\end{aligned}
$$

> See [`derive_master_secret_key_from_seed`](#derive-master-secret-key-from-seed).

> $\ovskm$ MUST NOT enter an app circuit.  
> $\ovskm$ MAY enter the kernel circuit.

### Master Outgoing Viewing Public Key

The Master Outgoing Viewing Public Key is only included so that other people can derive the user's address from some public information (i.e. from $\Ovpkm$), in such a way that $\ovskm$ is tied to the user's address.

$$
\begin{aligned}
& \Ovpkm \in \mathbb{G}_{\text{Grumpkin}} \\
& \Ovpkm = \text{derive\_public\_key}(\ovskm)
\end{aligned}
$$

> See [`derive_public_key`](#derive-public-key-from-secret-key).

### App-siloed Outgoing Viewing Secret Key

The App-siloed Outgoing Viewing Secret Key is a **hardened** child key, and so is only derivable by the owner of the Master Outgoing Viewing Secret Key. It is hardened so as to enable the $\ovskapp$ to be passed into an app circuit, without the threat of $\ovskm$ being reverse-derivable by a malicious app. Only when an app-siloed public key needs to be derivable by the general public is a normal (non-hardened) key derivation scheme used.

$$
\begin{aligned}
& \ovskapp \in \mathbb{F}_r \\
& \ovskapp = \text{derive\_hardened\_app\_siloed\_secret\_key}(\text{``az\_ovsk\_app''}, \text{app\_address}, \ovskm)
\end{aligned}
$$

> See [`derive_hardened_app_siloed_secret_key`](#derive-hardened-app-siloed-secret-key).

## Incoming Viewing Keys

If a sender wants to send some recipient a private message or note, they can derive an ephemeral symmetric encryption key from the recipient's Master Incoming Viewing Public Key. I.e. these keys are for decrypting "incoming" data from the pov of a recipient.

### Master Incoming Viewing Secret Key

$$
\begin{aligned}
& \ivskm \in \mathbb{F}_r \\
& \ivskm = \text{derive\_master\_secret\_key\_from\_seed}(\text{``az\_ivsk\_m''}, \seed)
\end{aligned}
$$

> See [`derive_master_secret_key_from_seed`](#derive-master-secret-key-from-seed).

> $\ivskm$ MUST NOT enter an app circuit.

### Master Incoming Viewing Public Key

The Master Incoming Viewing Public Key can be used by a sender to encrypt messages and notes to the owner of this key.

$$
\begin{aligned}
& \Ivpkm \in \mathbb{G}_{\text{Grumpkin}} \\
& \Ivpkm = \text{derive\_public\_key}(\ivskm)
\end{aligned}
$$

> See [`derive_public_key`](#derive-public-key-from-secret-key).

### App-siloed Incoming Viewing Secret Key

An App-siloed Incoming Viewing Secret Key is not prescribed in this spec, because depending on how an app developer wishes to make use of such a key, it could have implications on the security of the Master Incoming Viewing Secret Key.

> TODO: more discussion needed here, to explain everything we've thought about.

## Tagging Keys

The "tagging" key pair can be used to flag "this ciphertext is for you", without requiring decryption.

### Master Tagging Secret Key

$$
\begin{aligned}
& \tskm \in \mathbb{F}_r \\
& \tskm = \text{derive\_master\_secret\_key\_from\_seed}(\text{``az\_tvsk\_m''}, \seed)
\end{aligned}
$$

> See [`derive_master_secret_key_from_seed`](#derive-master-secret-key-from-seed).

> $\ivskm$ MUST NOT enter an app circuit.

### Master Tagging Public Key

$$
\begin{aligned}
& \Tpkm \in \mathbb{G}_{\text{Grumpkin}} \\
& \Tpkm = \text{derive\_public\_key}(\tskm)
\end{aligned}
$$

> See [`derive_public_key`](#derive-public-key-from-secret-key).

## Acknowledgements

Much of this is inspired by the [ZCash Sapling and Orchard specs](https://zips.z.cash/protocol/protocol.pdf).
