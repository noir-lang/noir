---
sidebar_position: 3
---

# Encryption and Decryption

Applications should be able to provably encrypt data for a target user, as part of private message delivery. As stated on the Keys section, we define three types of encrypted data, based on the sender and the recipient, from the perspective of a user:

- Incoming data: data created by someone else, encrypted for and sent to the user.
- Outgoing data: data created by the user to be sent to someone else, encrypted for the user.
- Internal incoming data: data created by the user, encrypted for and sent to the user.

Encryption mechanisms support these three types of encryption, which may rely on different keys advertised by the user.

## Precompiles and Note Discovery

Even though encryption is a well-solved problem, unlike note discovery, the protocol bundles both operations together for simplicity and efficiency. Most use cases call for encryption and note tagging to be executed together, so note tagging precompile contracts are expected to handle encryption as well. This allows users to choose their preferred encryption method, trading between encryption cost and bits of security.

## Key Abstraction

To support different kinds of encryption mechanisms, the protocol does not make any assumptions on the type of public keys advertised by each user. Validation of their public keys is handled by the precompile contract selected by the user.

## Provable Decryption

While provable encryption is required to guarantee correct private message delivery, provable decryption is required for disclosing activity within an application. This allows auditability and compliance use cases, as well as being able to prove that a user did not execute certain actions. To support this, encryption precompiles also allow for provable decryption.