## Desirable properties

There are two main properties we might desire from the various hash functions defined in aztec:

### Collision Resistance

Poseidon2 is the predominant hash function in the Aztec protocol. It is assumed to be collision resistant.

#### Domain Separators

To minimize the potential for collisions between distinct hashing contexts, all hashes are domain-separated by passing a `string` which describes the context of the hash. Each such string in this spec begins with the prefix `az_`, to domain-separate all hashes specifically to the Aztec protocol.

> The strings provided in this spec are mere suggestions at this stage; we might find that the strings should be smaller bit-lengths, to reduce in-circuit constraints.

In the case of using Poseidon2 for hashing (which is the case for most hashing in the Aztec protocol), the string is converted from a big-endian byte representation into a `Field` element, and passed as a first argument into the hash. In the case of using non-algebraic hash functions (such as sha256), the string is converted from a big-endian byte representation into bits, and passed as the first bits into the hash. These details are conveyed more clearly as pseudocode in the relevant sections of the spec.

For some hashes there is further domain-separation. For example, [Merkle tree hashing](../../../aztec/concepts/storage/trees/index.md#layers) of the tree.

### Pseudo-randomness

Sometimes we desire the output of a hash function to be pseudo-random. Throughout the Aztec protocol, it is assumed that Poseidon2 can be used as a pseudo-random function.

Pseudo-randomness is required in cases such as:

- Fiat-Shamir challenge generation.
- Expanding a random seed to generate additional randomness.
  - See the derivation of [master secret keys](../../../aztec/concepts/accounts/keys.md#master-keys).
- Deriving a nullifier, and siloing a nullifier.
  - See [deriving a nullifier](../../../aztec/concepts/accounts/keys.md#deriving-a-nullifier-within-an-app-contract).
