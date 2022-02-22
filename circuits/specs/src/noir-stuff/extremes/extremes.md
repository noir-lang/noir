# Extremes

This is all investigative.

This section sets out (in non-Noir pseudocode) what Noir functions might look like in two extremes:
(TODO: feel free to update these to Noir syntax)
-  Extreme 1: Noir knows all about the architecture of Aztec 3, and so can abstract all of the 'inner workings' of Aztec 3 away from the developer.
-  Extreme 2: Noir doesn't know any of the inner workings of Aztec 3, and so the developer would have to manually write circuits which exactly conform to Aztec 3's architecture.

We'll probably end up somewhere in the middle.

## Examples covered:

- [incrementation of private state _not_ owned by caller](./incr-private-not-owned.md)
  - Note: if a private state is _not_ owned by the caller, then this is the only mutation that can safely be done on that state. This is because the caller cannot know the current value of the state they're modifying. So they can only safely add to it. (Unless we relax some design constraints, perhaps).
- [incrementation of private state owned by the caller](./incr-private-owned.md)
- [decrementation of private state owned by the caller](./decr-private-owned.md)
- [editing a public state](./edit-public.md)
- [reading a public state](./read-public.md)
- [calling a private function of a different contract](./call-private.md)
- [calling a public function of a different contract](./call-public.md)
- [calling a public function from a private function of the same contract](./call-public-same.md)
- [calling an L1 function](./call-l1.md)
- [emitting an 'event'](./emit-event.md)
- [executing a function as a callback, following an L1 result](./callback.md)
- [deploying a contract from a private/public function](./deploy-contract.md)
- [new contract constructors]()
- ['globally' available variables](./global-vars.md)