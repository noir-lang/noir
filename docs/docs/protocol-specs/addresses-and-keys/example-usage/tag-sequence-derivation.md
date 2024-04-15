# Handshaking for tag-hopping

Deriving a sequence of tags for tag-hopping.

## Deriving a sequence of tags between Alice and Bob across all apps

For Bob to derive a shared secret for Alice:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\esk_{hs}$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key, for handshaking | $hs$ = handshake.
$\Epk_{hs}$ | $\esk_{hs} \cdot G$ | Ephemeral public key, for handshaking |
$\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}$ | $\esk_{hs} \cdot \Ivpkm$ | Shared secret, for tagging | Here, we're illustrating the derivation of a shared secret (for tagging) using _master_ keys.

Having derived a Shared Secret, Bob can now share it with Alice as follows:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Taghs$ | $\esk_{hs} \cdot \Tpkm$ | Handshake message identification tag | Note: the tagging public key $\Tpkm$ exists as an optimization, seeking to make brute-force message identification as fast as possible. In many cases, handshakes can be performed offchain via traditional web2 means, but in the case of on-chain handshakes, we have no preferred alternative over simply brute-force attempting to reconcile every 'Handshake message identification tag'. Note: this optimization reduces the recipient's work by 1 cpu-friendly hash per message (at the cost of 255-bits to broadcast a compressed encoding of $\Taghs$). We'll need to decide whether this is the right speed/communication trade-off. | 
$\payload$ | [$\Taghs$, $\Epk_{hs}$] | Payload | This can be broadcast via L1.<br />Curve points can be compressed in the payload. |

Alice can identify she is the indended the handshake recipient as follows:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\Taghs$ | $\tskm \cdot \Epk_{hs}$ | Handshake message identification tag | Alice can extract $\Taghs$ and $\Epk_{hs}$ from the $\payload$ and perform this scalar multiplication on _every_ handshake message. If the computed $\Taghs$ value matches that of the $\payload$, then the message is indented for Alice.<br />Clearly, handshake transactions will need to be identifiable as such (to save Alice time), e.g. by revealing the contract address of some canonical handshaking contract alongside the $\payload$.<br />Recall: this step is merely an optimization, to enable Alice to do a single scalar multiplication before moving on (in cases where she is not the intended recipient). |

If Alice successfully identifies that she is the indended the handshake recipient, she can proceed with deriving the shared secret (for tagging) as follows:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\sharedsecret_{m,tagging}^{Bob \rightarrow Alice}$ | $\ivskm \cdot \Epk_{hs}$ | Shared secret, for tagging |  |

A sequence of tags can then be derived by both Alice and Bob as:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\tagg_{m,i}^{Bob \rightarrow Alice}$ | $\text{pos2}(\text{``az\_tag\_ss\_m''}, \sharedsecret_{m,tagging}^{Bob \rightarrow Alice}, i)$ | The i-th tag in the sequence. |  |

This tag can be used as the basis for note retreival schemes. Each time Bob sends Alice a $\ciphertext$, he can attach the next unused $\tagg_{m,i}^{Bob \rightarrow Alice}$ in the sequence. Alice - who is also able to derive the next $\tagg_{m,i}^{Bob \rightarrow Alice}$ in the sequence - can make privacy-preserving calls to a server, requesting the $\ciphertext$ associated with a particular $\tagg_{m,i}^{Bob \rightarrow Alice}$.

> The colour key isn't quite clear for $\tagg_{m,i}^{Bob \rightarrow Alice}$. It will be a publicly-broadcast piece of information, but no one should learn that it relates to Bob nor Alice (except perhaps some trusted 3rd party whom Alice has entrusted with her $\ivskm$).

<!-- TODO: Prevent spam (where a malicious user could observe the emitted tag $\tagg_{m,i}^{Bob \rightarrow Alice}$, and re-emit it many times via some other app-contract). Perhaps this could be achieved by emitting the tag as a nullifier (although this would cause state bloat). -->

## Deriving a sequence of tags from Bob to himself across all apps

The benefit of Bob deriving a sequence of tags for himself, is that he can re-sync his _outgoing_ transaction data more quickly, if he ever needs to in future.

This can be done by either:

- Copying the approach used to derive a sequence of tags between Bob and Alice (but this time do it between Bob and Bob, and use Bob's outgoing keys).
- Generating a very basic sequence of tags $\tagg_{app, i}^{Bob \rightarrow Bob} = \text{pos2}(\text{``az\_tag\_ovsk\_app''}, \ovskapp, i)$ (at the app level) and $\tagg_{m, i}^{Bob \rightarrow Bob} = \text{pos2}(\text{``az\_tag\_ovsk\_m''}, \ovskm, i)$ (at the master level).
  - Note: In the case of deriving app-specific sequences of tags, Bob might wish to also encrypt the app*address as a ciphertext header (and attach a master tag $\tagg*{m, i}^{Bob \rightarrow Bob}$), to remind himself of the apps that he should derive tags _for_.
