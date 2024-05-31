---
title: Private Kernel Circuit
---

This circuit is executed by the user, on their own device. This is to ensure private inputs to the circuit remain private!

Read the latest information about the Aztec Private Kernel Circuit in the [protocol specs section](../../../../protocol-specs/circuits/private-kernel-initial).

:::note

**This is the only core protocol circuit which actually needs to be "zk" (zero-knowledge)!!!** That's because this is the only core protocol circuit which handles private data, and hence the only circuit for which proofs must not leak any information about witnesses! (The private data being handled includes: details of the Aztec.nr Contract function which has been executed; the address of the user who executed the function; the intelligible inputs and outputs of that function).

Most so-called "zk-Rollups" do not make use of this "zero-knowledge" property. Their snarks are "snarks"; with no need for zero-knowledge, because they don't seek privacy; they only seek the 'succinct' computation-compression properties of snarks. Aztec's "zk-Rollup" actually makes use of "zero-knowledge" snarks. That's why we sometimes call it a "zk-zk-Rollup", or "_actual_ zk-Rollup".

:::
