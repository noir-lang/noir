# Code Freeze Fall 2021: Schnorr signatures

The code is templated in such a way that the primes $q$ and $r$ are defined relative to the group `G1`, which is unfortunate, since $r$ is chosen as a fixed, definite value in our specs. An alternative would be to have the templates in schnorr.tcc refer to `F_nat` and `F_em` (for 'native' and 'emulated') or something like this. The easier and probably better alternative for now is to just rename our primes in the Yellow Paper as $p_{\text{B}}$ and $p_{\text{G}}$.

For Aztec's current uses cases, `G1` is a cyclic subgroup of an elliptic curve defined over a field $\mathbb{F}_q$ (implemented as a class `Fq`), and `Fr` (aka `field_t`) is the a field of size equal to the size of `G1`, so `Fr` is the field acting on `G1` by scalar multiplication.

## Role:

Yellow paper only mentions them here: _The Blake2s hash is utilized for computing nullifiers and for generating pseudorandom challenges, when verifying Schnorr signatures and when recursively verifying Plonk proofs_.

They are used by the account circuit and the join-split circuit.

## Data types

`crypto::schnorr::signature` is a pair $(s, e)$ of two 256-bit integers represented as length-32 `std::array`'s of `uint32_t`'s.

`crypto::schnorr::signature_b` is a pair $(s, r)$ of the same type.

`wnaf_record<C>` is a vector of `bool_t<C>`'s along with a skew.

`signature_bits<C>` is four `field_t`'s, representing a signature $(s, e)$ by splitting component into two.

## Formulas

### Elliptic curve addition

We restrict in this code to working with curves described by Weierstrass equations of the form $y^2 = x^3 + B$ defined over a $\mathbb{F}_r$ with $r$ prime. Consider two non-identity points $P_1 = (x_1, y_1)$, $P_2 = (x_2, y_2)$. If $x_1 = x_2$, then $y_1 = \pm y_2$, so the two points are equal or one is the inverse of the other. If $y_1 = y_2$, then one has $x_1 = \zeta x_2$ with $\zeta^3=1$. In the case of Grumpkin, the equation $X^3-1$ splits over $\mathbb{F}_r$, there are indeed distinct pairs of points satisfying this relation (for an example of how we handle this elsewhere in the code base, see https://github.com/AztecProtocol/aztec2-internal/issues/437).

Suppose $P_1 \neq - P_2$. Then $P_1 + P_2 = (x_3, y_3)$ with

$$ 
x_3 = \lambda^2 - (x_1 + x_2), \quad y_3 = \lambda.(x_1 - x_3) - y_1 
$$

where $\lambda = \dfrac{y_2 - y_1}{x_2 - x_1}$ if $P_1 \neq P_2$ and $\lambda = \dfrac{3x_1^2}{2y_1}$ if $P_1 = P_2$.

## Algorithms

Let $g$ be a generator of $\mathbb{G}_1$.

### HMAC

We the HMAC algorithm as Pseudo-Random Function (PRF) to generate a randomly distributed Schnorr signature nonce $k$ in a deterministic way.
HMAC is the Hash-based Message Authentication Code specification as defined in [RFC4231](https://tools.ietf.org/html/rfc4231).

The HMAC algorithm: Given a message $m$, and a PRF key $\text{priv}$, the $\text{output}$ value is computed as 

$$\text{output} = \text{HMAC}( \text{priv}, \mathtt{m}) = h(s' \oplus \text{opad} || h((s' \oplus \text{ipad} || m)))$$
where:

- $h$ is a hash function modeling a random oracle, whose block size is 64 bytes
- $s'$ is a block-sized key derived from $\text{priv}$. 
    - If $\text{priv}$ is larger than the block size, we first hash it using $h$ and set $s' = h(\text{priv}) \vert\vert (0 \ldots 0)$
    - Otherwise, $s' = \text{priv} \vert\vert (0 \ldots 0)$
    - In both cases, $s'$ is right-padded with 0s up to the 64 byte block size.
- $\text{opad}$ is a 64-byte string, consisting of repeated bytes valued `0x5c`
- $\text{ipad}$ is a 64-byte string, consisting of repeated bytes valued `0x36`
- $||$ denotes concatenation
- $\oplus$ denotes bitwise exclusive or
- $text{output}$ is a 32-byte string

In order to derive a secret nonce $k \in \mathbb{F}_r$, we need to expand $text{output}$ in order to derive a 512 bit integer
$$
k_{512} = h\big(\text{HMAC}( \text{priv}, \mathtt{m}) \ \vert \vert \ 1 \big)  \ \vert \vert \  h\big(\text{HMAC}( \text{priv}, \mathtt{m}) \ \vert \vert\ 0 \big) \in [0,\ldots,2^{512}-1].
$$
Modeling $k_{512}$ as a uniformly sampled integer, taking $k = k_{512} \bmod r$ ensures that the statistical distance between the distribution of $k$ and the uniform distribution over $\mathbb{F}_r$ is negligible.

### Sign

We use signatures with compression as described in Section 19.2.3 of [BS], in the sense that the signature contains the hash, meaning that the signature contains a hash and a field element, rather than a group element and a field element.

The algorithm: Given a message $m$, an account $(\text{priv}, \text{pub})\in \mathbb{F}_r \times \mathbb{G}_1$ produces the signature
$$\text{Sig} = (s, e) \in \{0,1\}^{256} \times \{0,1\}^{256}$$

where:

- $s = \big( \mathbb{F}_r(k) - \text{priv} \cdot \mathbb{F}_r(e) \big) \bmod r  \in  \{0,1\}^{256}$.
- $k =  h\big(\text{HMAC}( \text{priv}, \mathtt{m}) \ \vert \vert \ 0 \big)  \ \vert \vert \  h\big(\text{HMAC}( \text{priv}, \mathtt{m}) \ \vert \vert\ 1 \big) \in  \{0,1\}^{512}$ is the signer's secret nonce.
- $R = (R.x,R.y) = \mathbb{F}_r(k) \cdot g \in \mathbb{G}_1$, is a commitment to the signer's nonce $k$.
- $e= h(p(R.x, \text{pub}.x, \text{pub.y}) \ \vert \vert \ m) \in  \{0,1\}^{256}$ is the Fiat-Shamir response.
- $\text{pub} = (x,y) \in \mathbb{F}_q \times \mathbb{F}_q \approx \mathbb{G}_1$ is the affine representation of the signer's public key
- $\mathbb{F}_r (\cdot) : \{0,1\}^{\star} \rightarrow \mathbb{F}_r$ is a function interpreting a binary string as an integer and applying the modular reduction by $r$.
- $p$ is a collisian-resistant pedersen hash function.
- $h$ is a hash function modeling a random oracle, which is instantiated with BLAKE2s. 

The purpose of $p(r, \text{pub}.x, \text{pub.y})$ is to include the public key in the parameter $e$ whilst ensuring the input to $h$ is no more than 64 bytes.

### Verify

Given $\text{Sig} = (s, e)\in \{0,1\}^{256} \times \{0,1\}^{256}$, purported to be the signature of a messages $m$ by an account $(\text{priv}, \text{pub})\in \mathbb{F}_r \times \mathbb{G}_1$ with respect to a random oracle hash function $h$, compute

- $R = (R.x, R.y) = \mathbb{F}_r(e)\cdot \text{pub} + \mathbb{F}_r(s)\cdot g \in \mathbb{G}_1$;
- $e' = h(p(R.x, \text{pub}.x, \text{pub}.y)\ \vert\vert\ m) \in \{0,1\}^{256}$.

The signature is verified if and only if $e'== e$, where the comparison is done bit-wise.

Imprecise rationale: The verification equation is $e = h((e.pub + s.g).x, m)$ where both sides of the equation are represented as an array of 256 bits. 
VERIFIER has seen that SIGNER can produce a preimage for a given $e$ which is outside of SIGNER's control by chosing a particular value of $s$. 
The difficulty of this assumption is documented, in the case where $\mathbb{G}_1$ is the units group of a finite field, in Schnorr's original paper [Sch] (cf especially pages 10-11).

### Variable base multiplication

scalar presented as `bit_array`

scalar presented as a `wnaf_record`, provided along with a `current_accumulator`

## Code Paths

### `verify_signature`

- There is an aborted state reached if $s\cdot g$ and $e\cdot pub$ have the same x-coordinate.
- Normal signature verification path.

### `variable_base_mul(pub_key, current_accumulator, wnaf)`

- This function is only called inside of `variable_base_mul(pub_key, low_bits, high_bits)`. There is an `init` predicate given by: "`current_accumulator` and `pub_key` have the same x-coordinate". This is intended as a stand-in for the more general check that these two points are equal. This condition distinguishes between two modes in which the function is used in the implementation of the function `variable_base_mul(pub_key, low_bits, high_bits)`: on the second call, the condition `init` is espected to be false, so that the results of the first call, recorded in `current_accumulator`, are incorporated in the output.
- There is branching depending on whether on the parity of the scalar represented by `wnaf`.

### `variable_base_mul(pub_key, low_bits, high_bits)`

- There is an aborted state that is reached when either of the field elements is zero.

### `convert_signature(scontext, signature)`

There is no branching here.

### `convert_message(context, message_string)`

This function has not been investigated since I propose it be removed. It is not used or tested.

### `convert_field_into_wnaf(context, limb)`

- When accumulating a `field_t` element using the proposed wnaf representaiton, there is branching at each bit position depending on the 32nd digit of the current `uint64_t` element `wnaf_entries[i+1]`.

## Security Notes

### Usage of HMAC for deterministic signatures 

There are two main reasons why one may want deterministic signatures. 
In some instances, the entropy provided by the system may be insufficient to guarantee uniform `k`, and using `HMAC` with a proper cryptographic hash function should therefore ensure this property. 
By deriving it from the secret key, it also ensures that `k` remains private to the signer. 
Nowadays, and especially with the types of devices we would be creating signatures, we can assume that the system's randomness source is strong enough for creating signatures.

There are different ways of achieving this property, such as [RFC 6979](https://datatracker.ietf.org/doc/html/rfc6979), or as defined by the [EdDSA](https://ed25519.cr.yp.to/eddsa-20150704.pdf) specification.

Our approach is closer to RFC 6979, though we do not use rejection sampling and instead generate a 512-bit value and apply modular reduction by $r$.
This ensures that the statistical difference between the distribution of `k` and the uniform distribution over $\\{ 0,1, \ldots, r-1\\}$ is negligible.  
Note that any leakage of the value of `k` may be catastrophic, especially in ECDSA. 

Unfortunately, by using the secret key $\text{priv}$ for signing and as input to `HMAC`, the original security proof of the signature scheme no longer applies.
We would need to derive two independent signing and PRF keys from one 256-bit secret seed.


### Signature malleability

Given a valid signature $(s,e) \in  \{0,1\}^{256} \times \{0,1\}^{256}$ , it is possible to generate another valid signature $(s',e) \in  \{0,1\}^{256} \times \{0,1\}^{256}$, where $s'\neq s$ but $\mathbb{F}_r(s') = \mathbb{F}_r(s)$ (take $s'$ to be congruent to $s$ modulo $r$).
In our context, signatures are used within the `account` and `join_split` circuits to link the public inputs to the user's spending key. 
The signatures themselves are private inputs to the circuit and are not revealed. We do not depend on their non-malleability in this context. 
The solution would be to check that $\text{int}(s) < r$. 

### Missing $R.y$ component in Pedersen hash

As mentioned, we use the collision-resistant Pedersen hash to compress $R$ and $\text{pub}$ when computing the Fiat-Shamir challenge $e$. 
We are aware that we do not embed the $y$ coordinate of $R$ and are working on a security proof to ensure this does not render the scheme insecure.


### Biased sampling of Fiat-Shamir challenge $e$ 

When we interpret $e \in \{0,1\}^{256}$ as a field element by reducing the corresponding integer modulo $r$, 
the resulting field element is slightly biased in favor of "smaller" field elements, since $r \not\vert\ \ 2^{256}$. 
Fixing this issue would require a technique similar to the method we use to derive $k$ without bias. 
Unfortunately, this would require many more gates inside the circuit verification algorithm (additional hash compuation and modular reduction of a 512 bit integer).

We are no longer in the random oracle model since the distribution of the challenge is not uniform. 
We are looking into alternative proofs to guarantee correctness. 

### Domain separation

We do not use domain separation when generating the Fiat-Shamir challenge $e$ with BLAKE2s.
Other components using the same hash function as random oracle should be careful that this could not lead to collisions when similar inputs are being processed. 

We also note that we do not hash the group generator into the hash function.

## References

WNAF representation: https://github.com/bitcoin-core/secp256k1/blob/master/src/ecmult_impl.h, circa line 151

NOTE: the original NAF paper Morain, Olivos, "Speeding up the computations...", 1990 has a sign error in displayed equation (7). This is not present in our `variable_base_mul` function.

[BS20] Boneh, D., Shoup, V "A Graduate Course in Applied Cryptography" Version 0.5, January 2020.

[Sch] Schnorr, C. "Efficient Identification and Signatures for Smart Cards", 1990.
