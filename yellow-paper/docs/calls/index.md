---
title: Calls
---

# Calls

Functions in the Aztec Network can call other functions. These calls are [synchronous](./sync-calls.md) when they they occur within private functions or within public functions, but are [enqueued](./enqueued-calls.md) when done from a private to a public function, and optionally [batched](./batched-calls.md). The protocol also supports alternate call methods, such as [static](./static-calls.md), [delegate](./delegate-calls.md), and [unconstrained](./unconstrained-calls.md) calls.

In addition to function calls, the protocol allows for communication via message-passing back-and-forth between L1 and L2, as well as from public to private functions.

import DocCardList from '@theme/DocCardList';

<DocCardList />
