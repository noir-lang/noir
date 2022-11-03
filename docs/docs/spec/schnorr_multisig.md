# Schnorr Multi Signatures

We implement the SpeedyMuSig protocol by E. Crites, C. Komlo and M. Maller described in [ePrint 2021/1375](https://eprint.iacr.org/2021/1375) in Fig. 7. 
Given a group of $n$ signers with public/private key pairs $\\{  (\mathsf{pk}_i,\mathsf{sk}_i) \\}_i$, the protocol allows the group to generate a Schnorr signature for an aggregate public key $\mathsf{pk}$ derived from the group's public keys.
We use it to generate Schnorr signatures over aggregated spending keys, which are used inside of circuits to validate the public inputs. 

The Schnorr signatures produced are defined over the Grumpkin curve, using BLAKE2s as random-oracle to derive Fiat-Shamir challenges.

## Protocol description 

SpeedyMuSig is a two round multi-party computation. 

### Setup 

Each user $P_i$ with signing key-pair $(\mathsf{pk}_i,\mathsf{sk}_i)$ generates a Proof of Possession $\pi_i$ which is a variant of a Schnorr proof of knowledge of the discrete logarithm of a group element. 
Its purpose is to attest that $P_i$ actually knows the secret key $\mathsf{sk}_i$ associated to the public key $\mathsf{pk}_i \in \mathbb{G}$  such that $\mathsf{pk}_i = \mathsf{sk}_i \cdot G$.

The users share their public keys and associated proofs $(\mathsf{pk}_i, \pi_i)$, and verify the proofs.
If all checks succeed, the aggregated public key can be computed as $\mathsf{pk} = \sum_{i} \mathsf{pk}_i$.

The proof of possession implementation follows the specification in the paper and can be instantiated over any curve and hash function used as random oracle. We deviate slightly from the paper since we implement the signature variant where we send $(\bar{c},\bar{z})$ _(defined as `(e,s)` in the code)_ instead of $(\bar{R},\bar{z})$. These representations are equivalent. 

We disallow the usage of the same hash function for both the signature scheme and the proof of knowledge (and later the nonce generation). 
This is done to ensure that all three hash functions $H_{reg}, H_{non}, H_{sig}$ are properly domain separated. 
To maintain compatibility with the existing circuits, we use BLAKE2s for $H_{sig}$ without domain separation. 
This means that we cannot use the same function for the other two hash functions, even if we include prefixes for the latter ones.
We therefore use the Keccak function, with proper domain separation strings, for $H_{reg}$ and $H_{non}$. 
In both functions, we also pass in the group generator $G$ as first argument. We do not apply this to the $H_{sig}$ function as we maintain backwards compatibility with the existing circuit. 

Proofs of possession are deterministic, with the nonce $k$ derived in a similar was as in our Schnorr implementation using HMAC.

There is a slight bias in the generation of the Fiat-Shamir challenge $e$ due to the modular reduction by $|\mathbb{F}|$.


### Round 1 

Each party $P_i$ generates a random secret nonce pair $(r_i,s_i) \in \mathbb{F}^2$ which is saves in its private state. 
The parties broadcast the corresponding commitments $(R_i,S_i)$ where $R_i = r_i \cdot G$ and $S_i = s_i \cdot G$.

This round can be performed in advance and multiple times in order to generate several sets of commitments, 
so that subsequent signing can be done faster. 

### Round 2

After receiving a list of commitments $\\{(R_i,S_i)\\}_i$ from all parties, each party computes their additive share $z_i$ of the final signature's $s$ component. 

The nonce challenge $a$ is computed slightly differently as in the paper, in order to prevent accidental collisions. 
Indeed, since the length of the message is variable, we add both a fixed prefix and suffix so that the message cannot be interpreted as the nonce from another signing session.
We have $a = H_{non}\Big(\mathsf{pk} \ \vert\vert\ \mathtt{"m\_start"}  \ \vert\vert\ m \ \vert\vert\ \mathtt{"m\_end"} \ \vert\vert\ (R_1,S_1)\ \vert\vert\ \cdots \ \vert\vert\ (R_n,S_n)\Big)$ 
(we interpret $a$ as a field element by reducing modulo $|\mathbb{F}|$, noting the slight bias from the uniform distribution).

The Schnorr challenge $R$ is then computed as $R = \sum_{i} (R_i + a \cdot S_i) = \Big(\sum_{i} R_i \Big) + a \cdot \Big( \sum_{i} S_i \Big)$, 
which is used to derive the final signature's Fiat-Shamir challenge $e$ (using the different hash function).

Each user then computes $z_i = r_i + a\cdot s_i - \mathsf{sk}_i \cdot e$. 
At this point, these shares can simply be sent to a single trusted "signature coordinator", along with the public outputs from the first two rounds, 
who can compute the final signature $(s,e)$.

### Signature Aggregation

Once all parties have produced their round 2 output $z_i$, they can either broadcast this value to all other parties so that everyone can produce the final signature,
or they can send it to a central coordinator who will produce the final signature.


Given the set of round 1 nonce commitments $\\{ (R_i,S_i)\\}_i$, and the signature shares $\\{z_i\\}_i$, the final signature is computed as follows:
- validate all public keys with their proofs of possession.
- validate round 1 and round 2 messages
- recompute aggregate public key
- recompute values $a, R$ and derive the signature's Fiat-Shamir challenge $e$ 
- compute the $s$ component of the signature as $s = \sum_i z_i$. 

The final signature is $(s, e)$ as defined by the single party Schnorr signature algorithm implementation.

## Known issues

### Bias in Fiat-Shamir challenges 

When the protocol is instantiated over an elliptic curve whose subgroup's order is far from $2^{256}-1$ (the range of the output of our hash functions), 
the field element we obtain from the Fiat-Shamir transform will not exactly follow the uniform distribution over $\mathbb{F}$.
Applying the modular reduction over the 256-bit hash output (which we assume is uniform) induces a small bias in favor of "smaller" field elements. 
Therefore, the field element $e$ does not actually follow a uniform distribution over $\mathbb{F}$. 

Since fixing this issue would require circuit changes, we chose to accept this slight bias.

## Usage notes

### Non-deterministic signatures 

Signatures produced with this protocol cannot be made deterministic as there is no easy way of ensuring that all participants are generating their shares of the nonce deterministically.
While deterministic signatures are handy in the single party case where the signer may not have access to strong randomness (in the browser for example), 
multisignature nonces essentially combine each signer's randomness using the unpredictable output of a hash function. 

### Message ordering 

When the parties are broadcasting the round 1 nonce commitements $(R_i,S_i)$, 
the signature will only succeed if all parties agree on the same ordering.
Otherwise, parties would end up with different $a$ values.