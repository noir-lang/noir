# Code Freeze Fall 2021: unsigned integers

###### tags: `project-notes`

A standard library `uint` is a circit manifestation of a fixed width unsigned integer. The type is parameterized by a composer and one of the built-in types `uintN_t` for `N = 8, 16, 32, 64`. The value here is referred to as the `width` of the type.

Shorthand: write `uint_ct` for a generic `uint<Composer, Native>`, and refer to an instance of such a class a simply a `uint_ct`.

# Role:
One wants such a type, for example, to implement traditional "numeric" hash functions, as we do for BLAKE2s and SHA256.

# Data types

The state of a `uint_ct` is described by the following `protected` members:
  - `Composer* context`
  - `mutable uint256_t additive_constant`: A component of the value.
  - `mutable WitnessStatus witness_status`: An indicator of the extent to which the instance has been normalized (see below).
  - `mutable std::vector<uint32_t> accumulators`: Accumulators encoding the base-4 expansion of the witness value, as produced by `TurboComposer::decomposer_into_base4_accumulators`. This vector is populated when the `uint_ct` is normalized. We record the values for later use in some operators (e.g., shifting).
  - `mutable uint32_t witness_index`: The index of a witness giving part of the value.

# Key concepts

## Value and constancy

Similar to the value of an instance of `field_t<Composer>`, the value (a `uint256_t`) of a `uint_ct` consists of a "constant part" and possibly a witness value. To be precise, the function `get_value` returns
    
    (uint256_t(context->get_variable(witness_index)) + additive_constant) & MASK,

where `MASK` enforces that the result is reduced modulo `width`. There is also an "unbounded" version that does not mask off any overflowing values.

The value of a `uint_ct` $a$ consists of a witness $a_w$ and a constant part $a_c$. We will use this notation throughout. If the index of the witness $a_w$ is the special `IS_CONSTANT` value, then $a$ is said to be constant. 
## Normalization

A naive implementation of the class `uint_ct` would take `field_t` and enrich it with structure to ensure that the value  is always appropriately range constrained. Our implementation is more efficient in several ways. 

We track an `additive_constant` to reduce the number of divisions (by $2^{width}$) that must be recorded by the prover; for instance, if a uint $a$ is to be repeatedly altered by adding circuit constants $c_1, ... , c_m$, the circuit writer is happy to save the prover some effort by computing $c = (c_1 + ... + c_m) \% 2^{width}$ and, instead, asking the prover to demonstrate that they have computed the long division of $a + c$ by $2^{width}$.

We also allow for the deferral of range constraints for efficiency reasons.

If $a$ is constant, then it is regarded as "normalized"--the prover does not need to provide any constraints on it showing that its value is of the appropriate width.

If $a$ is not constant, then it is allowed to exist in an 'unnormalized' state. By definition, normalizing $a$ means replacing it by a new `uint_ct` $rem$ with $rem_{c}=0$ and $rem_w$ proven to equal to $a_w + a_c \% 2^{width}$. To prove this equation, one must impose the following two constraints:

1) $a_w + a_c = 2^{width} q + r$ for some integers $q, r$;
2) $r$ lies in the range $[0, 2^{width}-1]$.

We track whether these two constraints have been applied independently. If the first constrain has been applied, then $a$ is said to be weakly normalized. If both have been applied, $a$ is said to be noramlized. This status is tracked through an enum called `WitnessStatus` that can take on three values.

## Example: addition

Our function `operator+` on `uint_ct`s does not return a normalized value. Suppose we apply it to compute $b = a_1 + a_2$ where $a_1, a_2$ are two `uint_ct`s both having zero additive constants. Abusing notation to conflate a `uint_ct` with its value, the constraints imposed by `operator+` are: $a_1 + a_2 = 2^{width} q_1 + b$ and $q_1\in \{0, 1, 2\}.$ That is, $b$ is only weakly normalized. Without appropriately range constraining $b$, it is not known that $b$ is the remainder of division of $a_1 + a_2$ by $2^{width}$.

Suppose we know ahead of time that we actually want to compute $a_1 + a_2 + a_3$ with $a_3$ also having additive zero additive constant. Computing this sum as $c = b + a_3$, the result $c$ is weakly normalized, backed by a constraint $b + a_3 = 2^{width} q_2 + c$. Now suppose that $c$ normalized. Altogether, we

 $$ 2^{width} q_2 + c = b + a_3 = a_1 + a_2 - 2^{width} q_1 + a_3 \quad{} \Rightarrow \quad{} a_1 + a_2 + a_3 = 2^{width} (q_1 + q_2) + c$$

 and $c \in [0, 2^{width}-1]$. This shows that we can defer range constraints and correctly compute `uint_ct` additions. 
 This, of course, has the tradeoff that the circuit writer must take care to manually impose range constraints when they are needed.


# Descriptions of algorithms

Extensive comments were added to code to document complicated formulas and describe our algorithms. Some of the logic has been delegated to the widgets, having been 'absorbed', so to speak, into the protocol definition itself. In particular, `create_balanced_add_gate` imposes an addition constraint and a range constraint, and this is described in the turbo arithmetic widget. Similarly `create_big_add_gate_with_bit_extraction` extract bits information from a scalar represented in terms of two-bit 'quads'. The audit added around these TurboPLONK gates and widgets.

A reader coming to the task of understanding this code with little or no preparation is advised to begin bu reading the function `TurboComposer::decompose_into_base4_accumulators`. This is the TurboPLONK function that imposes a range constraint by building a base-4 expansion of a given witness, recording this information in a vector of witnesses that accumulate to the given input (in the case of a correct proof). The decomposition there is used repeatedly for operations on uints (e.g., bit shifting).


# Code Paths
There is branching in `operator>`, where the conditions for `>` and `<=` are unified. This affects all of the other comparisons, which are implemented in terms of `>`.

Otherwise, the code avoids branching as much as possible. Some circuit construction algorithms divide into cases, (e.g., whether a bit shift is by an even or an odd amount), but the predicates in those cases are known at compile time, not just at proving time.