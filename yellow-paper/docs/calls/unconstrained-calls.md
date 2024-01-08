---
sidebar_position: 6
---

# Unconstrained calls

<!-- TODO: Validate use cases for unconstrained calls. Maybe these are not actually needed? -->

Private function calls can be executed as _unconstrained_. Unconstrained function calls execute the code at the target and return the result, but their execution is not constrained. It is responsibility of the caller to constrain the result, if needed. Unconstrained calls are a generalization of oracle function calls, where the call is not to a PXE function but to another contract. Side effects from unconstrained calls are ignored. Note that all calls executed from an unconstrained call frame will be unconstrained as well.

Unconstrained calls are executed via a `unconstrainedCallPrivateFunction` oracle call, which accepts the same arguments as a regular `callPrivateFunction`, and return the result from the function call. Unconstrained calls are not pushed into the `private_call_stack` and do not incur in an additional kernel iteration.

Rationale for unconstrained calls is to allows apps to consume results from functions that do not need to be provable. An example use case for unconstrained calls is unconstrained encryption and note tagging, which can be used when the sender is incentivized to ensure the recipient receives the data sent. 

Another motivation for unconstrained calls is for retrieving or computing data where the end result can be more efficiently constrained by the caller.