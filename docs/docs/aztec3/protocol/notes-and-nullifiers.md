---
title: Notes and Nullifiers
---

<!-- TODO -->

## Private State

- Talk about UTXOs

### Fragmented State (`Set`)

- Talk about how a state's 'value' can be fragmented across many notes.
- Discuss how states can be updated in such cases.
- Discuss how custom notes and nullifiers are possible
- Give examples that aren't just 'summing values'.
- Give examples of how custom notes gives builders flexibility, and enables compliant token designs.
- Give example Noir syntax
  - Discuss the interface that will be expected of notes, by our Noir Contract stdlib state variable types
  - Discuss how decrypting, recomputing the note_hash, and recomputing the nullifier works.


### Solid state (`Singleton`)

- Talk about a state which always inhabits one and only one note.
- Talk about initialisation nullifiers.


### Custom Nullifiers

- Talk about how we can have nullifier keys, which are siloed to one contract.
- Talk about other examples of how Nullifiers are used
  - zk-nullifiers
  - signalling, like semaphore
  - see nice blog post which suggests other kinds of nullifier

### Public Notes

- Talk about how _public_ state can also be stored in notes in the private data tree.
- Talk about the design challenges we've encountered (given that such notes don't need to be trial-decrypted becuase they're public!), and how we're solving those.

### Notes containing complex types

- Talk about how we can 'squish' complex types into Notes.


