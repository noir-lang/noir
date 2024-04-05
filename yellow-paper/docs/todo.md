TODO:

- See cryptography/todo.md
- Describe what we _desire_ the protocol to be; not what it is today.
- Define/describe:
  - an oracle
- More detail and precisely-named definitions relating to note hashes (inner, unique, siloed, ...) and nullifiers.
- Clear descriptions of the precompiles for handshaking & note tagging. Including the encoding of logs (header, body, ephemeral keys, etc.)
- Poseidon hash for all trees?
- Maybe describe all hashes on one page?
- For all hashes: Describe exact ordering of preimage values. Describe the encoding of each preimage value. State the hash to use. Include a domain separator. Give the hash a unique name. Is there anything weird like truncating an output to fit in a field?
  If possible, describe the property(or properties) we want from the hash (collision resistance?, 2nd preimage resistance? pseudo-random function? hash to curve?).
- Consistently use the same name for each concept throughout. Link to it the first time it’s mentioned on a page. When introducing a ‘thing’, try to put it in a section with a subtitle which matches that thing’s name.
- Structs (or tables) (clearly named and typed, in a subsection) for everything whose layout we should know!
- Who is going to write the specs for:
  - Data bus?
  - RAM/ROM?
  - Circuit arithmetisation
  - Custom Gates
  - The proving system (honk, goblin, protogalaxy, etc.)
- Spec logs better!
  - Use data bus?
  - Allow several (4 or 8 or something) fields to be emitted by a function, before needing to sha256 the data?
  - Have a 'reset' circuit to sha256 logs?
  - Will there be space in the data bus for logs?
  - Resolve 'impersonation' discussions (see the forum)

Contents:

- Custom types
- Constants
  - Subdivided into categories:
    - Circuit constants
    - Tree constants
    - Seq Selection constants
    - P2P constants
    - Block constants
- Serialization & Deserialization
  - aztec.nr
- Encodings
- Hashing
- Merkleization
- Key derivation algorithms

Layout: highlight out-of-protocol information in a box.

Abstraction & Standardisation:

- Account abstraction
- Constructor abstraction
- Nonce abstraction
- Fee abstraction
- Tx Hash abstraction

Every struct, constant, other definition, needs a corresponding subheading with that exact name, somewhere in the docs? Might get ugly...
