#pragma once

#include <span>
namespace bb::honk {

/**
 * @brief Compute the correction term for the permutation argument.
 *
 * @tparam Field
 * @param public_inputs x₀, ..., xₘ₋₁ public inputs to the circuit
 * @param beta random linear-combination term to combine both (wʲ, IDʲ) and (wʲ, σʲ)
 * @param gamma Schwartz-Zippel random evaluation to ensure ∏ᵢ (γ + Sᵢ) = ∏ᵢ (γ + Tᵢ)
 * @param domain_size Total number of rows required for the circuit (power of 2)
 * @param offset Extent to which PI are offset from the 0th index in the wire polynomials, for example, due to inclusion
 * of a leading zero row or Goblin style ECC op gates at the top of the execution trace.
 * @return Field Public input Δ
 */
template <typename Flavor>
typename Flavor::FF compute_public_input_delta(std::span<const typename Flavor::FF> public_inputs,
                                               const typename Flavor::FF& beta,
                                               const typename Flavor::FF& gamma,
                                               const auto domain_size,
                                               size_t offset = 0)
{
    using Field = typename Flavor::FF;
    Field numerator = Field(1);
    Field denominator = Field(1);

    // Let m be the number of public inputs x₀,…, xₘ₋₁.
    // Recall that we broke the permutation σ⁰ by changing the mapping
    //  (i) -> (n+i)   to   (i) -> (-(i+1))   i.e. σ⁰ᵢ = −(i+1)
    //
    // Therefore, the term in the numerator with ID¹ᵢ = n+i does not cancel out with any term in the denominator.
    // Similarly, the denominator contains an extra σ⁰ᵢ = −(i+1) term that does not appear in the numerator.
    // We expect the values of W⁰ᵢ and W¹ᵢ to be equal to xᵢ.
    // The expected accumulated product would therefore be equal to

    //   ∏ᵢ (γ + W¹ᵢ + β⋅ID¹ᵢ)        ∏ᵢ (γ + xᵢ + β⋅(n+i) )
    //  -----------------------  =  ------------------------
    //   ∏ᵢ (γ + W⁰ᵢ + β⋅σ⁰ᵢ )        ∏ᵢ (γ + xᵢ - β⋅(i+1) )

    // At the start of the loop for each xᵢ where i = 0, 1, …, m-1,
    // we have
    //      numerator_acc   = γ + β⋅(n+i) = γ + β⋅n + β⋅i
    //      denominator_acc = γ - β⋅(1+i) = γ - β   - β⋅i
    // at the end of the loop, add and subtract β to each term respectively to
    // set the expected value for the start of iteration i+1.
    // Note: The public inputs may be offset from the 0th index of the wires, for example due to the inclusion of an
    // initial zero row or Goblin-stlye ECC op gates. Accordingly, the indices i in the above formulas are given by i =
    // [0, m-1] + offset, i.e. i = offset, 1 + offset, …, m - 1 + offset.
    Field numerator_acc = gamma + (beta * Field(domain_size + offset));
    Field denominator_acc = gamma - beta * Field(1 + offset);

    for (const auto& x_i : public_inputs) {
        numerator *= (numerator_acc + x_i);     // γ + xᵢ + β(n+i)
        denominator *= (denominator_acc + x_i); // γ + xᵢ - β(1+i)

        numerator_acc += beta;
        denominator_acc -= beta;
    }
    return numerator / denominator;
}

/**
 * @brief Compute lookup grand product delta
 *
 * @details Similar to how incorporation of public inputs into the permutation grand product results in
 * z_permutation(X_n) = \Delta_{PI}, the structure of the lookup grand product polynomial results in
 * z_lookup(X_n) = (γ(1 + β))^n = \Delta_{lookup}. This is a side effect of the way in which we
 * incorporate the original plookup construction (for which z_lookup(X_n) = 1) into plonk/honk.
 * See https://hackmd.io/@aztec-network/ByjS5GplK? for a more detailed explanation.
 *
 * @tparam Field
 * @param beta
 * @param gamma
 * @param domain_size dyadic circuit size
 * @return Field
 */
template <typename Field>
Field compute_lookup_grand_product_delta(const Field& beta, const Field& gamma, const auto domain_size)
{
    Field gamma_by_one_plus_beta = gamma * (Field(1) + beta); // γ(1 + β)
    return gamma_by_one_plus_beta.pow(domain_size);           // (γ(1 + β))^n
}

} // namespace bb::honk