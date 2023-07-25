# Noir Contracts

## What is a Noir Contract?

**Noir** is a domain specific language for creating and verifying proofs. It's design choices are influenced heavily by Rust.

We've extended the Noir language to understand the notion of an **'Aztec smart contract'**.

- A **smart contract** is just a collection of persistent state variables, and a collection of functions which may edit those state variables.
- An **Aztec smart contract** is a smart contract with **private** state variables and **private** functions.
- A **Noir Contract** is just an Aztec smart contract, written in Noir syntax.


> Throughout these docs, we'll refer to "regular Noir" as being the version of Noir without the Noir Contract syntax.