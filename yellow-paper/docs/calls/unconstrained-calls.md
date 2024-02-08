# Unconstrained calls

<!--
TODO: Validate use cases for unconstrained calls. Maybe these are not actually needed?
I wasn't actually aware of these. What's the benefit of doing this vs simply simulating the private function? I suppose the benefit is that an 'external' private function can be called as an unconstrained function by some _constrained_ function? Why not create a duplicate version of the private function, which is unconstrained, and call that instead?
-->
<!-- What about executing public functions as unconstrained? -->

Private function calls can be executed as _unconstrained_. Unconstrained function calls execute the code at the target and return the result, but their execution is not constrained. It is responsibility of the caller to constrain the result, if needed. Unconstrained calls are a generalization of oracle function calls, where the call is not to a PXE function but to another contract. Side effects from unconstrained calls are ignored. Note that all calls executed from an unconstrained call frame will be unconstrained as well.

Unconstrained calls are executed via a `unconstrainedCallPrivateFunction` oracle call, which accepts the same arguments as a regular `callPrivateFunction`, and return the result from the function call. Unconstrained calls are not pushed into the `private_call_stack` and do not incur in an additional kernel iteration.

THe rationale for unconstrained calls is to allows apps to consume results from functions that do not need to be provable. An example use case for unconstrained calls is unconstrained encryption and note tagging, which can be used in applications where constraining such encryption computations isn't necessary, e.g. if the sender is incentivized to ensure the recipient receives the correct data.

Another motivation for unconstrained calls is for retrieving or computing data where the end result can be more efficiently constrained by the caller.
