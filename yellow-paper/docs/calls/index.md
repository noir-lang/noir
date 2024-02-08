---
title: Calls
---

<!-- Mike review: General comment for calls: it would be nice to see some very simple diagrams showing the flow for how the different calls are pushed, popped, and processed, if possible.
Also, struct definitions should be given or linked-to (elsewhere in the yp) - it's quite difficult to visualise a struct from a prose description. -->

<!--
In some section (either 'calls', 'state', or maybe another section), we should explain which kinds of functions can read/write/call other kinds of state/functions. Maybe wait until this discussion resolves itself: https://docs.google.com/spreadsheets/d/12Fk0oTvj-yHbdnAkMnu0ymsDqCOEXLdmAxdVB5T_Y3Q/edit#gid=0
-->

# Calls

Functions in the Aztec Network can call other functions. There are several types of call:

- [Synchronous calls](./sync-calls.md): when a private function calls another private function; or when a public function calls another public function.
- [Enqueued calls](./enqueued-calls.md): when a private function calls a public function.
- [Batched calls](./batched-calls.md): when multiple calls to the same function are enqueued and processed as a single call on a concatenation of the arguments.

The protocol also supports alternative call methods, such as [static](./static-calls.md), [delegate](./delegate-calls.md), and [unconstrained](./unconstrained-calls.md) calls.

In addition to function calls, the protocol allows for communication via message-passing back-and-forth between L1 and L2, as well as from public to private functions.

import DocCardList from '@theme/DocCardList';

<DocCardList />
