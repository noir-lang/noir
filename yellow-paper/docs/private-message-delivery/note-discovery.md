---
sidebar_position: 3
---

# Note Discovery

## Requirements

When users interact with contracts they will generate and publish encrypted notes for other network participants. In order for a user to consume notes that belong to them, they need to identify, retrieve and decrypt them. A simple, privacy-preserving approach to this would be to download all of the notes and attempt decryption. However, the total number of encrypted notes published by the network will be substantial, making it infeasible for some users to do this. Those users will want to utilize a note discovery protocol to privately identify their notes.

A number of techniques currently exist to help with this and it is a field into which a lot of research is being conducted. Therefore, our approach is not to dictate or enshrine a specific note discovery mechanism but to put in place the necessary abstractions such that users can freely choose. Additionally, through this approach we allow for integration of new or improved protocols in the future.

## Tag Abstraction

When applications produce notes they will need to call a protocol defined function within the account contract of the recipient and request that a tag be generated. From the protocol's perspective, this tag will simply be a stream of bytes relevant only to the recipient's note discovery protocol. It will be up to the account contract to constrain that the correct tag has been generated and from there the protocol circuits along with the rollup contract will ensure that the tag is correctly published along with the note.