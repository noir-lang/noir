---
sidebar_position: 3
---

# Note Discovery

## Requirements

When users interact with contracts they will generate and publish encrypted notes for other network participants. In order for a user to consume those notes, they need to identify, retrieve and decrypt them. The total number of encrypted notes published by the network will be substantial, making it infeasible for some users to simply retrieve every note and attempt a naive brute-force decryption. For this reason, those users will want to utilize a note discovery protocol to privately identify and provide a much smaller subset of notes for the user to decrypt.

A number of techniques currently exist to perform this task with various compromises of levels of privacy and the required amounts of computational effort and/or network bandwidth. This is a field into which a lot of research if being conducted so our approach is not to dictate a specific technique but to put in place the necessary abstractions such that users can select their preferred protocol and new techniques can be integrated in the future.

## Tag Abstraction

When applications produce notes they will need to call a protocol defined function within the account contract of the recipient and request that a tag be generated. From the protocol's perspective, this tag will simply be a stream of bytes relevant only to the recipient's note discovery protocol. It will be up to the account contract to constrain that the correct tag has been generated and from there the protocol circuits along with the rollup contract will ensure that the tag is correctly published along with the note.