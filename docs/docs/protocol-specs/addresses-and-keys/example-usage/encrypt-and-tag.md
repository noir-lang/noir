## Encrypt and tag an incoming message

Bob wants to send Alice a private message, e.g. the contents of a note, which we'll refer to as the $\plaintext$. Bob and Alice are using a "tag hopping" scheme to help with note discovery. Let's assume they've already handshaked to establish a shared secret $\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}$, from which a sequence of tags $\tagg_{m,i}^{Bob \rightarrow Alice}$ can be derived.

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | Given by Alice | (Diversifier) | Remember, in most cases, $\d=1$ is sufficient.
$\Gd$ | $\d \cdot G$ | (Diversified) generator | Remember, when $\d = 1$, $\Gd = G$.
$\eskheader$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkdheader$ | $\eskheader \cdot \Gd$ | (Diversified) Ephemeral public key |
$\sharedsecret_{m,header}$ | $\esk_{header} \cdot \Ivpkm$ | Shared secret, for ciphertext header encryption | TODO: can we use the same ephemeral keypair for both the ciphertext header and the ciphertext?<br />TODO: diversify the $\Ivpkm$? |
$\hmencheader$ | h("?", $\sharedsecret_{m,header}$) | Ciphertext header encryption key |
$\ciphertextheader$ | $enc^{\Ivpkm}_{\hmencheader}$(app\_address) | Ciphertext header |  |
|||||
$\esk$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkd$ | $\esk \cdot \Gd$ | (Diversified) Ephemeral public key |
$\sharedsecret_{app,enc}$ | $\esk \cdot \Ivpkdstealth$ | Shared secret, for ciphertext encryption |
$\happenc$ | h("?", $\sharedsecret_{app,enc}$) | Incoming data encryption key |
$\ciphertext$ | $enc^{\Ivpkdstealth}_{\happenc}(\plaintext)$ | Ciphertext |
$\payload$ | [$\tagg_{m, i}^{Bob \rightarrow Alice}$, $\ciphertextheader$, $\ciphertext$, $\Epkdheader$, $\Epkd$] | Payload |

<!-- TODO: This requires app-specific incoming viewing keys, which we don't have. How do we adapt this derivation? -->

Alice can learn about her new $\payload$ as follows. First, she would identify the transaction has intended for her, either by observing $\tagg_{m, i}^{Bob \rightarrow Alice}$ on-chain herself (and then downloading the rest of the payload which accompanies the tag), or by making a privacy-preserving request to a server, to retrieve the payload which accompanies the tag. Assuming the $\payload$ has been identified as Alice's, and retrieved by Alice, we proceed.

> Given that the tag in this illustration was derived from Alice's master key, the tag itself doesn't convey which app_address to use, to derive the correct app-siloed incoming viewing secret key that would enable decryption of the ciphertext. So first Alice needs to decrypt the $\ciphertextheader$ using her master key:

<!-- prettier-ignore -->
| Thing | Derivation | Name |
|---|---|---|
$\sharedsecret_{m,header}$ | $\ivskm \cdot \Epkdheader$ | Shared secret, for encrypting the ciphertext header |
$\hmencheader$ | h("?", $\sharedsecret_{m,header}$) | Incoming encryption key |
app_address | $decrypt_{\hmencheader}^{\ivskm}(\ciphertextheader)$ | App address |
||||
$\ivskstealth$ | See derivations above. Use the decrypted app_address. | Incoming viewing secret key |
$\sharedsecret_{app, enc}$ | $\ivskstealth \cdot \Epkd$ | Shared secret, for ciphertext encryption |
$\happenc$ | h("?", $\sharedsecret_{app, enc}$) | Ciphertext encryption key |
$\plaintext$ | $decrypt_{\happenc}^{\ivskstealth}(\ciphertext)$ | Plaintext |

## Encrypt and tag an outgoing message

Bob wants to send himself a private message (e.g. a record of the outgoing notes that he's created for other people) which we'll refer to as the $\plaintext$. Let's assume Bob has derived a sequence of tags $\tagg_{m,i}^{Bob \rightarrow Alice}$ for himself (see earlier).

> Note: this illustration uses _master_ keys for tags, rather than app-specific keys for tags. App-specific keys for tags could be used instead, in which case a 'ciphertext header' wouldn't be needed for the 'app_address', since the address could be inferred from the tag.

> Note: rather than copying the 'shared secret' approach of Bob sending to Alice, we can cut a corner (because Bob is the sender and recipient, and so knows his own secrets).

> Note: if Bob has sent a private message to Alice, and he also wants to send himself a corresponding message:
>
> - he can likely re-use the ephemeral keypairs for himself.
> - he can include $\esk$ in the plaintext that he sends to himself, as a way of reducing the size of his $\ciphertext$ (since the $\esk$ will enable him to access all the information in the ciphertext that was sent to Alice).

<!-- TODO: can we use a shared public key to encrypt the $\ciphertextheader$, to reduce duplicated broadcasting of encryptions of the app_address? E.g. derive a shared secret, hash it, and use that as a shared public key? -->

> Note: the violet symbols should actually be orange here.

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | Given by Alice | (Diversifier) | Remember, in most cases, $\d=1$ is sufficient.
$\Gd$ | $\d \cdot G$ | (Diversified) generator | Remember, when $\d = 1$, $\Gd = G$.
$\eskheader$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkdheader$ | $\eskheader \cdot \Gd$ | (Diversified) Ephemeral public key |
$\hmencheader$ | h("?", $\ovskm$, $\Epkdheader$) | Header encryption key | This uses a master secret key $\ovskm$, which MUST NOT be given to an app nor to an app circuit. However, it can still be given to a trusted precompile, which can handle this derivation securely. | 
$\ciphertextheader$ | $enc_{\hmencheader}$(app\_address) | Ciphertext header encryption key |  |
|||||
$\esk$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key |
$\Epkd$ | $\esk \cdot \Gd$ | (Diversified) Ephemeral public key |
$\happenc$ | h("?", $\ovskapp$, $\Epkd$) | Outgoing data encryption key | Since $\ovskapp$ is a _hardened_ app-siloed secret key, it may be safely given to the dapp or passed into the app's circuit. |
$\ciphertext$ | $enc_{\happenc}(\plaintext)$ | Ciphertext |
$\payload$ | [$\tagg_{m, i}^{Bob \rightarrow Bob}$, $\ciphertextheader$, $\ciphertext$, $\Epkdheader$, $\Epkd$] | Payload |

Alice can learn about her new $\payload$ as follows. First, she would identify the transaction has intended for her, either by observing $\tagg_{m, i}^{Bob \rightarrow Alice}$ on-chain herself (and then downloading the rest of the payload which accompanies the tag), or by making a privacy-preserving request to a server, to retrieve the payload which accompanies the tag. Assuming the $\payload$ has been identified as Alice's, and retrieved by Alice, we proceed.

> Given that the tag in this illustration was derived from Alice's master key, the tag itself doesn't convey which app_address to use, to derive the correct app-siloed incoming viewing secret key that would enable decryption of the ciphertext. So first Alice needs to decrypt the $\ciphertextheader$ using her master key:

<!-- prettier-ignore -->
| Thing | Derivation | Name |
|---|---|---|
$\hmencheader$ | h("?", $\ovskm$, $\Epkdheader$) |  |
app_address | $decrypt_{\hmencheader}(\ciphertextheader)$ |
||||
$\ovskapp$ | See derivations above. Use the decrypted app_address. | |
$\happenc$ | h("?", $\ovskm$, $\Epkd$) |
$\plaintext$ | $decrypt_{\happenc}(\ciphertext)$ |

<!-- TODO: how does a user validate that they have successfully decrypted a ciphertext? Is this baked into ChaChaPoly1035, for example? -->

### Doing this inside an app circuit

Here's how an app circuit could constrain the app-siloed outgoing viewing secret key ($\ovskapp$) to be correct:

The app circuit exposes, as public inputs, an "outgoing viewing key validation request":

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
`outgoing_viewing_key_validation_request` | app_address: app_address,<br />hardened_child_sk: $\nskapp$,<br />claimed_parent_pk: $\Npkm$ |

The kernel circuit can then validate the request (having been given $\ovskm$ as a private input to the kernel circuit):

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\ovskapp$ | $h(\ovskm, \text{app\_address})$ |
$\Ovpkm$ | $\ovskm \cdot G$ | Outgoing viewing public key |
| | | | Copy-constrain $\ovskm$ with $\ovskm$. |

If the kernel circuit succeeds in these calculations, then the $\ovskapp$ has been validated as the correct app-siled secret key for $\Ovpkm$.

## Encrypt and tag an internal incoming message

Internal incoming messages are handled analogously to outgoing messages, since in both cases the sender is the same as the recipient, who has access to the secret keys when encrypting and tagging the message.
