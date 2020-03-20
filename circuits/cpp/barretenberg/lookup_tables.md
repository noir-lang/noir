# Data structures  

For a given PLONK program, we have several unknowns:  

1. What is the size of t?
2. What is the size of s?  

If we work on fixed data structures, the lookup table logic is simplified, as the polynomial identities being 
evaluated are over a fixed number of polynomials.  

Upgrading the protocol to have a variable number of `s` and `t` polynomials could be a future project - it will be a lot easier to do this once we have working code that uses fixed data structures.

It seems reasonable to assume that, for any given PLONK program with a depth of `n`, that the number of elements in `s` will be < 2n.  
If this condition is not met, we can add dummy gates into the circuit this condition is satisfied.  

### Representing t  

The degree of the quotient polynomial is minimized, if for any given row index `i`, we *either* combine `f` terms into the lookup grand product, *or* we combine `t` terms. But not at the same time.  

This means that `t` is sparse - it only contains nonzero elements when a given row `i` is not involved in a lookup.  

It seems reasonable to assume that we can represent every element of `t` in this sparse form, using 1 polynomial commitment of total degree `n`. If this is not the case,  
we can add dummy gates into the circuit until this condition is met.

### Representing elements of t  

A lookup table entry is a tuple of several distinct elements:  

1: At least one key  
2: At least one value  

Our XOR and AND tables are composed of two keys and one value  
Our Pedersen table is composed of one key and two values  
Our range table is composed of just one key  

We can make all of our table entries conform to a single structure, using a tuple of three values (t1, t2, t3) and a random element `alpha`:

`t = t1 + \alpha * t2 + \alpha^2 * t3`  

Each lookup table will have a distinct selector polynomial that maps program memory cells into `s`, which can be used to ensure that all lookups  
conform to this structure. For lookups that do not require all 3 elements, unused elements are set to `0`.

### Lookup table data structures  

1. `s1(X)`: the first sorted list polynomial  
2. `s2(X)`: the second sorted list polynomial  
3. `t1(X)`: the first lookup polynomial
4. `t2(X)`: the second lookup polynomial  
5. `t3(X)`: the third lookup polynomial  

## Representing lookup table data structures in the proving key and verifier key  

Barretenberg has `proving_key` and `verification_key` classes. `proving_key` uses a `std::string` key to map to a `polynomial` object. This is how we store polynomials - our `widget` classes can then 
extract polynomials from the map to perform the polynomial arithmetic required to construct the quotient polynomial.  

Similarly, `verification_key` maps `std::string` keys to `barretenberg::g1::affine_element` objects - commitments to the polynomials.  

We can add the following keys into the proving key / verification key:  

1. `z_lookup` : the lookup table permutation polynomial
2. `list_1` : the first sorted list polynomial
3. `list_2` : the second sorted list polynomial
4. `table_1` : the first lookup polynomial
5. `table_2` : the second lookup polynomial
6. `table_3` : the third lookup polynomial
7. `table_betas` : precomputed polynomial used to split the lookup table into sub-tables
8. `table_selector` : precomputed polynomial used to indicate when a given row contains a lookup table entry  
9. `range_selector` : precomputed polynomial that selects whether a row is involved in a range lookup  
10. `and_selector` : precomputed polynomial that selects whether a row is involved in a logical AND  
11. `xor_selector` : precomputed polynomial that selects whether a row is inovlved in a logical XOR  
12. `pedersen_selector` : precomputed polynomial that selects whether a row is involved in a pedersen lookup  

The latter four selector polynomials can be implemented in stages.  

### Modifying the prover algorithms  

We already have a `turbo_logic_widget`, `turbo_range_widget` and `turbo_fixed_base_widget` (for Pedersen hashes) widget classes implemented. So it seems intuitive that 
we adapt these to use lookups, instead of the current situation which uses custom gates.  

This creates a relatively distinct separation of concerns. We split up the lookup table grand product polynomial identity into the following two components:  

1. The part that evaluates a product of `f` elements  
2. The part that evaluates a product of `t` and `s` elements  

The former part is computed in a `Widget` class that is linked to a specific lookup table.  

The latter part is computed in a new `Widget` class: `lookup_widget`  

### New methods needed to evaluate lookups  

Currently, widgets do not have the ability to add commitments into the proof - we don't expose a method that can be called that does this. We need to change this.  

We can add two new methods that are called for every widget class: `compute_commitments` and `compute_randomized_commitments`.  

The majority of our widgets will do nothing when these method is called (i.e. have an empty function definition).  

The `lookup_widget`, however, will commit to `s_1` and `s_2`  in `compute_commitments`, and commit to `z_lookup` in `compute_randomized_commitments` 

The other existing widget methods can be used to perform all of the remaining required prover computations. `compute_quotient_contribution` will be used to compute the grand product of `z_lookup`.  

Similarly, the verification components of our widgets can be used to reconstruct the quotient polynomial evaluation (`compute_quotient_evaluation_contribution`) and to update a list of scalar multiplications the verifier must perform (`append_scalar_multiplication_inputs`)

# Computing witness values  

We need to construct several new polynomials as part of the lookup table sub-protocol. We can create a new `Composer` class that does this. e.g. `UltraComposer` - we modify the existing `TurboComposer`, so that when `compute_proving_key` is called, all of the lookup table precomputed polynomials are calculated. 

TODO: flesh out new composer structure
