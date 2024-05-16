---
title: Portals
---

A portal is a point of contact between L1 and a contract on Aztec. For applications such as token bridges, this is the point where the tokens are held on L1 while used in L2.

As outlined in [Communication](../../../../../protocol-specs/l1-smart-contracts/index.md), an Aztec L2 contract does not have to be linked to a portal contract, but can specify an intended portal in storage. Note, that a portal doesn't actually need to be a contract, it could be any address on L1.
