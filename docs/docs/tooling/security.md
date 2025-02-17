---
title: Security checks
description: Security checks currently provided by the compiler
keywords: [Nargo, Security, Brillig, Unconstrained]
sidebar_position: 2
---

# Security checks

Two compilation security passes exist currently to ensure soundness of compiled code. Problems they catch are reported as "bugs" (as opposed to errors) in the compiler output.

### Independent subgraph detection

This pass examines the instruction flow graph to see if the final function would involve values that don't come from any provided inputs and don't result in the outputs. That would mean there are no constraints ensuring the required continuity.

This check is enabled by default and can be disabled by passing the `--skip-underconstrained-check` option to `nargo`.

### Brillig manual constraint coverage

The results of a Brillig function call must be constrained to ensure security, adhering to these rules: every resulting value (including every array element of a resulting array) has to be involved in a later constraint (i.e. assert, range check) against either one of the arguments of the call, or a constant. In this context, involvement means that a descendant value (e.g. a result of a chain of operations over the value) of a result has to be checked against a descendant value of an argument.

This pass checks if the constraint coverage of Brillig calls is sufficient in these terms.

The check is at the moment disabled by default due to performance concerns and can be enabled by passing the `--enable-brillig-constraints-check` option to `nargo`.

#### Lookback option

Certain false positives of this check can be avoided by providing the `--enable-brillig-constraints-check-lookback` option to `nargo`, which can be slower at compile-time but additionally ensures that descendants of call argument values coming from operations *preceding* the call itself would be followed. For example, consider this case:

```
unconstrained fn unconstrained_add(v0: Field, v1: Field) -> Field {
    v0 + v1
}

fn main f0 (v0: Field, v1: Field) {
    let foo = v0 + v1;
    let bar = unconstrained_add(v0, v1);
    assert(foo == bar);
    return bar
}
```

Normally, the addition operation over `v0` and `v1` happening before the call itself would prevent the call from being (correctly) considered properly constrained. With this option enabled, the false positive goes away at the cost of the check becoming somewhat less performant on large unrolled functions. 
