## Deriving diversified public keys

A diversified public key can be derived from Alice's keys, to enhance Alice's transaction privacy. If Alice's counterparties' databases are compromised, it enables Alice to retain privacy from such leakages. Diversified public keys are used for generating diversified addresses.

Basically, Alice must personally derive and provide Bob and Charlie with random-looking addresses (for Alice). Because Alice is the one deriving these Diversified Addresses (they can _only_ be derived by Alice), if Bob and Charlie chose to later collude, they would not be able to convince each-other that they'd interacted with Alice.

This is not to be confused with 'Stealth Addresses', which 'flip' who derives: Bob and Charlie would each derive a random-looking Stealth Address for Alice. Alice would then discover her new Stealth Addresses through decryption.

> All of the key information below is Alice's

Alice derives a 'diversified' incoming viewing public key, and sends it to Bob:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ |diversifier |
$\Gd$ | $\d \cdot G$ | diversified generator |
$\Ivpkmd$ | $\ivskm \cdot \Gd$ | Diversified incoming viewing public key |

> Notice: when $\d = 1$, $\Ivpkmd = \Ivpkm$. Often, it will be unncessary to diversify the below data, but we keep $\d$ around for the most generality.

## Deriving stealth public keys

> All of the key information below is Alice's

Stealth Public Keys are used for generating Stealth Addresses. For Bob to derive a Stealth Address for Alice, Bob derives:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\d$ | Given by Alice | (Diversifier) | Remember, in most cases, $\d=1$ is sufficient.
$\Gd$ | $\d \cdot G$ | (Diversified) generator | Remember, when $\d = 1$, $\Gd = G$.
$\esk_{stealth}$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret, for deriving the stealth key shared secret |
$\Epkd,_{stealth}$ | $\esk_{stealth} \cdot \Gd$ | (Diversified) Ephemeral public key, for deriving the stealth key shared secret |
$\sharedsecret_{m, stealth}$ | $\esk_{stealth} \cdot \Ivpkmd$ | Stealth key shared secret |
$\hstealth$ | $\text{pos2}(\text{``az\_stealth\_key''}, \sharedsecret_{m, stealth})$ | stealth key |
$\Ivpkmdstealth$ | $\hstealth \cdot \Gd + \Ivpkmd$ | (Diversified) Stealth viewing public key |

Having derived a Stealth Address for Alice, Bob can now share it with Alice as follows:

<!-- prettier-ignore -->
| Thing | Derivation | Name | Comments |
|---|---|---|---|
$\tagg_{m, i}^{Bob \rightarrow Alice}$ | See earlier in this doc. | | Derive the next tag in the $Bob\rightarrow Alice$ sequence.<br />Note: we illustrate with a _master_ tag sequence, but an app-specific tag sequence could also be used (in which case an encryption of the app_address in a ciphertext header wouldn't be required; it could just be inferred from the tag used). |
$\esk_{header}$ | $\stackrel{rand}{\leftarrow} \mathbb{F}$ | ephemeral secret key, for deriving the ciphertext header shared secret |
$\Epkd,_{header}$ | $\esk_{header} \cdot \Gd$ | (Diversified) Ephemeral public key, for deriving the ciphertext header shared secret |
$\sharedsecret_{m,header}$ | $\esk_{header} \cdot \Ivpkm$ | Ciphertext header shared secret | TODO: we might need to use a different ephemeral keypair from the one used to derive the stealth address. |
$\hmencheader$ | $\text{pos2}(\text{``az\_enc\_key''}, \sharedsecret_{m,header})$ | ciphertext header encryption key
$\ciphertextheader$ | $\text{encrypt}^{\Ivpkm}_{\hmencheader}$(app\_address) | | TODO: diversify this? |
$\payload$ | [ $\tagg_{m, i}^{Bob \rightarrow Alice}$, $\Epkd,_{header}$, $\ciphertextheader$, $\Epkd,_{stealth}$ ] |

Alice can learn about her new Stealth Address as follows. First, she would identify the transaction has intended for her, either by observing $\tagg_{m, i}^{Bob \rightarrow Alice}$ on-chain herself (and then downloading the rest of the payload which accompanies the tag), or by making a privacy-preserving request to a server, to retrieve the payload which accompanies the tag. Assuming the $\payload$ has been identified as Alice's, we proceed:

<!-- prettier-ignore -->
| Thing | Derivation | Name |
|---|---|---|
$\sharedsecret_{m,header}$ | $\ivskm \cdot \Epkd,_{header}$ | Ciphertext header shared secret |
$\hmencheader$ | $\text{pos2}(\text{``az\_enc\_key''}, \sharedsecret_{m,header})$ | ciphertext header encryption key |
app_address | $\text{decrypt}_{\hmencheader}^{\ivskm}(\ciphertextheader)$ |
$\ivskm$ | See derivations above. Use the decrypted app_address in the derivation. | app-specific incoming viewing secret key |
$\sharedsecret_{m, stealth}$ | $\ivskm \cdot \Epkd,_{stealth}$ | Stealth key shared secret |
$\hstealth$ | $\text{pos2}(\text{``az\_stealth\_key''}, \sharedsecret_{m, stealth})$ | stealth key |
$\ivskmstealth$ | $\hstealth + \ivskm$ |
$\Ivpkmdstealth$ | $\ivskmstealth \cdot \Gd$ | (Diversified) Stealth viewing public key |
