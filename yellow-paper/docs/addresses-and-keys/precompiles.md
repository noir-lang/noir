---
title: Precompiles
---

<!-- Missing exact descriptions of the interfaces & logic of each precompile. -->

Precompiled contracts, which borrow their name from Ethereum's, are contracts not deployed by users but defined at the protocol level. These contract [instances](../contract-deployment/instances.md) and their [classes](../contract-deployment/classes.md) are assigned well-known low-number addresses and identifiers, and their implementation is subject to change via protocol upgrades. Precompiled contracts in Aztec are implemented as a set of circuits, one for each function they expose, like user-defined private contracts. Precompiles may make use of the local PXE oracle.

Note that, unlike user-defined contracts, the address of a precompiled [contract instance](../contract-deployment/instances.md) and the [identifier of its class](../contract-deployment/classes.md#class-identifier) both have no known preimage.

The rationale for precompiled contracts is to provide a set of vetted primitives for [note encryption](../private-message-delivery/private-msg-delivery.md#encryption-and-decryption) and [tagging](../private-message-delivery/private-msg-delivery.md#note-tagging) that applications can use safely. These primitives are guaranteed to be always-satisfiable when called with valid arguments. This allows account contracts to choose their preferred method of encryption and tagging from any primitive in this set, and application contracts to call into them without the risk of calling into a untrusted code, which could potentially halt the execution flow via an unsatisfiable constraint. Furthermore, by exposing these primitives in a reserved set of well-known addresses, applications can be forward-compatible and incorporate new encryption and tagging methods as accounts opt into them.

## Constants

- `ENCRYPTION_BATCH_SIZES=[4, 8, 16, 32]`: Defines what max [batch sizes](../calls/batched-calls.md) are supported in precompiled encryption methods.
- `ENCRYPTION_PRECOMPILE_ADDRESS_RANGE=0x00..0xFFFF`: Defines the range of addresses reserved for precompiles used for encryption and tagging.
- `MAX_PLAINTEXT_LENGTH`: Defines the maximum length of a plaintext to encrypt.
- `MAX_CIPHERTEXT_LENGTH`: Defines the maximum length of a returned encrypted ciphertext.
- `MAX_TAGGED_CIPHERTEXT_LENGTH`: Defines the maximum length of a returned encrypted ciphertext prefixed with a note tag.

<!-- TODO: move to the dedicated constants.md file -->

## Encryption and tagging precompiles

All precompiles in the address range `ENCRYPTION_PRECOMPILE_ADDRESS_RANGE` are reserved for encryption and tagging. Application contracts are expected to call into these contracts with note plaintext(s), recipient address(es), and public key(s). To facilitate forward compatibility, all unassigned addresses within the range expose the functions below as no-ops, meaning that no actions will be executed when calling into them.

<!-- Q: How is this 'no-op' functionality achieved? -->

All functions in these precompiles accept a `PublicKeys` struct which contains the user-advertised public keys. The structure of each of the public keys included can change from one encryption method to another, with the exception of the `nullifier_key` which is always restricted to a single field element. For forward compatibility, the precompiles interface accepts a hash of the public keys, which can be expanded within each method via an oracle call.

```
struct PublicKeys:
  nullifier_key: Field
  incoming_encryption_key: PublicKey
  outgoing_encryption_key: PublicKey
  incoming_internal_encryption_key: PublicKey
  tagging_key: PublicKey
```

To identify which public key to use in the encryption, precompiles also accept an enum:

```
enum EncryptionType:
  incoming = 1
  outgoing = 2
  incoming_internal = 3
```

Precompiles expose the following private functions:

```
validate_keys(public_keys_hash: Field): bool
```

Returns true if the set of public keys represented by `public_keys` is valid for this encryption and tagging mechanism. The precompile must guarantee that any of its methods must succeed if called with a set of public keys deemed as valid. This method returns `false` for undefined precompiles.

```
encrypt(public_keys_hash: Field, encryption_type: EncryptionType, recipient: AztecAddress, plaintext: Field[MAX_PLAINTEXT_LENGTH]): Field[MAX_CIPHERTEXT_LENGTH]
```

Encrypts the given plaintext using the provided public keys, and returns the encrypted ciphertext.

```
encrypt_and_tag(public_keys_hash: Field, encryption_type: EncryptionType, recipient: AztecAddress, plaintext: Field[MAX_PLAINTEXT_LENGTH]): Field[MAX_TAGGED_CIPHERTEXT_LENGTH]
```

Encrypts and tags the given plaintext using the provided public keys, and returns the encrypted note prefixed with its tag for note discovery.

```
encrypt_and_broadcast(public_keys_hash: Field, encryption_type: EncryptionType, recipient: AztecAddress, plaintext: Field[MAX_PLAINTEXT_LENGTH]): Field[MAX_TAGGED_CIPHERTEXT_LENGTH]
```

Encrypts and tags the given plaintext using the provided public keys, broadcasts them as an event, and returns the encrypted note prefixed with its tag for note discovery. These functions should be invoked via a [delegate call](../calls/delegate-calls.md), so that the broadcasted event is emitted as if it were from the caller contract.

```
encrypt<N>([call_context: CallContext, public_keys_hash: Field, encryption_type: EncryptionType, recipient: AztecAddress, plaintext: Field[MAX_PLAINTEXT_LENGTH] ][N]): Field[MAX_CIPHERTEXT_LENGTH][N]
encrypt_and_tag<N>([call_context: CallContext, public_keys_hash: Field, encryption_type: EncryptionType, recipient: AztecAddress, plaintext: Field[MAX_PLAINTEXT_LENGTH] ][N]): Field[MAX_TAGGED_CIPHERTEXT_LENGTH][N]
encrypt_and_broadcast<N>([call_context: CallContext, public_keys_hash: Field, encryption_type: EncryptionType, recipient: AztecAddress, plaintext: Field[MAX_PLAINTEXT_LENGTH] ][N]): Field[MAX_TAGGED_CIPHERTEXT_LENGTH][N]
```

Batched versions of the methods above, which accept an array of `N` tuples of public keys, recipient, and plaintext to encrypt in batch. Precompiles expose instances of this method for multiple values of `N` as defined by `ENCRYPTION_BATCH_SIZES`. Values in the batch with zeroes are skipped. These functions are intended to be used in [batched calls](../calls/batched-calls.md).

```
decrypt(public_keys_hash: Field, encryption_type: EncryptionType, owner: AztecAddress, ciphertext: Field[MAX_CIPHERTEXT_LENGTH]): Field[MAX_PLAINTEXT_LENGTH]
```

Decrypts the given ciphertext, encrypted for the provided owner. Instead of receiving the decryption key, this method triggers an oracle call to fetch the private decryption key directly from the local PXE and validates it against the supplied public key, in order to avoid leaking a user secret to untrusted application code. This method is intended for provable decryption use cases.

## Encryption strategies

List of encryption strategies implemented by precompiles:

### AES128

Uses AES128 for encryption, by generating an AES128 symmetric key and an IV from a shared secret derived from the recipient's public key and an ephemeral keypair. Requires that the recipient's keys are points in the Grumpkin curve. The output of the encryption is the concatenation of the encrypted ciphertext and the ephemeral public key.

Pseudocode for the encryption process:

```
encrypt(plaintext, recipient_public_key):
  ephemeral_private_key, ephemeral_public_key = grumpkin_random_keypair()
  shared_secret = recipient_public_key * ephemeral_private_key
  [aes_key, aes_iv] = sha256(shared_secret ++ [0x01])
  return ephemeral_public_key ++ aes_encrypt(aes_key, aes_iv, plaintext)
```

Pseudocode for the decryption process:

```
decrypt(ciphertext, recipient_private_key):
  ephemeral_public_key = ciphertext[0:64]
  shared_secret = ephemeral_public_key * recipient_private_key
  [aes_key, aes_iv] = sha256(shared_secret ++ [0x01])
  return aes_decrypt(aes_key, aes_iv, ciphertext[64:])
```

<!-- TODO: Why append the [1] at the end of the shared secret? Also, do we want to keep using sha, or should we use a snark-friendlier hash here? -->

## Note tagging strategies

List of note tagging strategies implemented by precompiles:

### Trial decryption

Trial decryption relies on the recipient to brute-force trial-decrypting every note emitted by the chain. Every note is attempted to be decrypted with the associated decryption scheme. If decryption is successful, then the note is added to the local database. This requires no note tags to be emitted along with a note.

In AES encryption, the plaintext is prefixed with the first 8 bytes of the IV. Decryption is deemed successful if the first 8 bytes of the decrypted plaintext matches the first 8 bytes of the IV derived from the shared secret.

This is the cheapest approach in terms of calldata cost, and the simplest to implement, but puts a significant burden on the user. Should not be used except for accounts tied to users running full nodes.

### Delegated trial decryption

Delegated trial decryption relies on a tag added to each note, generated using the recipient's tagging public key. The holder of the corresponding tagging private key can trial-decrypt each tag, and if decryption is successful, proceed to decrypt the contents of the note using the associated decryption scheme.

This allows a user to share their tagging private key with a trusted service provider, who then proceeds to trial decrypt all possible note tags on their behalf. This scheme is simple for the user, but requires trust on a third party.

<!-- TODO: How should the tag be generated here? Tags should be unique, and tags encrypted with the same pubkey should not be linkable to each other without the tagging private key. Can we use a similar method than the IV check in trial-decryption? -->

### Tag hopping

Tag hopping relies on establishing a one-time shared secret through a handshake between each sender-recipient pair, advertise the handshake through a trial-decrypted brute-forced channel, and then generate tags by combining the shared secret and an incremental counter. Recipients need to trial-decrypt events emitted by a canonical `Handshake` contract to detect new channels established with them, and then scan for the next tag for each open channel. Note that the handshake contract leaks whenever a new shared secret has been established, but the participants of the handshake are kept hidden.

This method requires the recipient to be continuously trial-decrypting the handshake channel, and then scanning for a number of tags equivalent to the number of handshakes they had received. While this can get to too large amounts for particularly active addresses, it is still far more efficient than trial decryption.

When Alice wants to send a message to Bob for the first time:

1. Alice creates a note, and calls into Bob's encryption and tagging precompile.
2. The precompile makes an oracle call to `getSharedSecret(Alice, Bob)`.
3. Alice's PXE looks up the shared secret which doesn't exist since this is their first interaction.
4. Alice's PXE generates a random shared secret, and stores it associated Bob along with `counter=1`.
5. The precompile makes a call to the `Handshake` contract that emits the shared secret, encrypted for Bob and optionally Alice.
6. The precompile computes `new_tag = hash(alice, bob, secret, counter)`, emits it as a nullifier, and prepends it to the note ciphertext before broadcasting it.

For all subsequent messages:

1. Alice creates a note, and calls into Bob's encryption and tagging precompile.
2. The precompile makes an oracle call to `getSharedSecret(Alice, Bob)`.
3. Alice's PXE looks up the shared secret and returns it, along with the current value for `counter`, and locally increments `counter`.
4. The precompile computes `previous_tag = hash(alice, bob, secret, counter)`, and performs a merkle membership proof for it in the nullifier tree. This ensures that tags are incremental and cannot be skipped.
5. The precompile computes `new_tag = hash(alice, bob, secret, counter + 1)`, emits it as a nullifier, and prepends it to the note ciphertext before broadcasting it.

## Defined precompiles

List of precompiles defined by the protocol:

<!-- prettier-ignore -->
| Address | Encryption | Note Tagging | Comments |
| ------- | ---------- | ------------ | -------- |
| 0x01    | Noop       | Noop         | Used by accounts to explicitly signal that they cannot receive encrypted payloads. Validation method returns `true` only for an empty list of public keys. All other methods return empty. |
| 0x02    | AES128     | Trial decryption | |
| 0x03    | AES128     | Delegated trial decryption | |
| 0x04    | AES128     | Tag hopping  | |                                                                                                                                                                                           |
