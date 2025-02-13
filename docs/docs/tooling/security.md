---
title: Security checks
description: Security checks currently provided by the compiler
keywords: [Nargo, Security, Brillig, Unconstrained]
sidebar_position: 2
---

# Security checks

Two SSA security passes exist currently to ensure soundness of compiled code. Problems they catch are reported as bugs (as opposed to errors) in the compiler output. 

### Independent subgraph detection

This pass examines the SSA to see if the final function would involve values that don't come from any provided inputs and don't result in the outputs. That would mean there are no constraints ensuring the required continuity.

This check is enabled by default and can be disabled by passing the `--skip-underconstrained-check` option to `nargo`.

### Brillig manual constraint coverage

The results of a Brillig function call must be constrained to ensure security, adhering to these rules: every resulting value (including every array element of a resulting array) has to be involved in a following constraint (assert, range check) against either one of the arguments of the call, or a constant. In this context, involvement means that a descendant value (e.g. a result of a chain of operations over the value) of a result has to be checked against a descendant value of an argument.

This pass checks if the constraint coverage of Brillig calls is sufficient in these terms.

The check is at the moment disabled by default due to performance concerns and can be enabled by passing the `--enable-brillig-constraints-check` option to `nargo`.

#### Lookback option

Certain false positives of this check can be avoided by providing the `--enable-brillig-constraints-check-lookback` option to `nargo`, which can be slower but additionally ensures that descendants of call argument values coming from operations *preceding* the call itself would be followed. For example, consider this SSA case:

```
acir(inline) fn main f0 {
  b0(v0: Field, v1: Field):
    v3 = add v0, v1
    v5 = call f1(v0, v1) -> Field
    constrain v3 == v5
    return v3
}

brillig(inline) fn foo f1 {
  b0(v0: Field, v1: Field):
    v2 = add v0, v1
    return v2
}
```

Normally, the `add` operation over `v0` and `v1` happening before the call itself would prevent the call from being (correctly) considered properly constrained. With lookback option, this false positive goes away at the cost of the check becoming somewhat less performant on large unrolled functions. 
