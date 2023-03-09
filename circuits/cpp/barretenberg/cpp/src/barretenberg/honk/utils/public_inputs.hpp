#pragma once

#include <span>
namespace honk {

/**
 * @brief Compute the correction term for the permutation argument.
 *
 * @tparam Field
 * @param public_inputs x₀, ..., xₘ₋₁ public inputs to the circuit
 * @param beta random linear-combination term to combine both (wʲ, IDʲ) and (wʲ, σʲ)
 * @param gamma Schwartz-Zippel random evaluation to ensure ∏ᵢ (γ + Sᵢ) = ∏ᵢ (γ + Tᵢ)
 * @param domain_size Total number of rows required for the circuit (power of 2)
 * @return Field Public input Δ
 */
template <typename Field>
Field compute_public_input_delta(std::span<const Field> public_inputs,
                                 const Field& beta,
                                 const Field& gamma,
                                 const size_t domain_size)
{
    Field numerator = Field::one();
    Field denominator = Field::one();

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
    Field numerator_acc = gamma + (beta * Field(domain_size));
    Field denominator_acc = gamma - beta;

    for (const auto& x_i : public_inputs) {
        numerator *= (numerator_acc + x_i);     // γ + xᵢ + β(n+i)
        denominator *= (denominator_acc + x_i); // γ + xᵢ - β(1+i)

        numerator_acc += beta;
        denominator_acc -= beta;
    }
    return numerator / denominator;
}

} // namespace honk