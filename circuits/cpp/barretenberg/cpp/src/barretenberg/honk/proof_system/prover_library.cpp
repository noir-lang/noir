#include "prover_library.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include <span>
#include <string>

namespace proof_system::honk::prover_library {

/**
 * @brief Compute the permutation grand product polynomial Z_perm(X)
 * *
 * @details (This description assumes Flavor::NUM_WIRES 3).
 * Z_perm may be defined in terms of its values  on X_i = 0,1,...,n-1 as Z_perm[0] = 1 and for i = 1:n-1
 *                  (w_1(j) + β⋅id_1(j) + γ) ⋅ (w_2(j) + β⋅id_2(j) + γ) ⋅ (w_3(j) + β⋅id_3(j) + γ)
 * Z_perm[i] = ∏ --------------------------------------------------------------------------------
 *                  (w_1(j) + β⋅σ_1(j) + γ) ⋅ (w_2(j) + β⋅σ_2(j) + γ) ⋅ (w_3(j) + β⋅σ_3(j) + γ)
 *
 * where ∏ := ∏_{j=0:i-1} and id_i(X) = id(X) + n*(i-1). These evaluations are constructed over the
 * course of four steps. For expositional simplicity, write Z_perm[i] as
 *
 *                A_1(j) ⋅ A_2(j) ⋅ A_3(j)
 * Z_perm[i] = ∏ --------------------------
 *                B_1(j) ⋅ B_2(j) ⋅ B_3(j)
 *
 * Step 1) Compute the 2*Flavor::NUM_WIRES length-n polynomials A_i and B_i
 * Step 2) Compute the 2*Flavor::NUM_WIRES length-n polynomials ∏ A_i(j) and ∏ B_i(j)
 * Step 3) Compute the two length-n polynomials defined by
 *          numer[i] = ∏ A_1(j)⋅A_2(j)⋅A_3(j), and denom[i] = ∏ B_1(j)⋅B_2(j)⋅B_3(j)
 * Step 4) Compute Z_perm[i+1] = numer[i]/denom[i] (recall: Z_perm[0] = 1)
 *
 * Note: Step (4) utilizes Montgomery batch inversion to replace n-many inversions with
 * one batch inversion (at the expense of more multiplications)
 *
 * @todo TODO(#222)(luke): Parallelize
 */

template <typename Flavor>
typename Flavor::Polynomial compute_permutation_grand_product(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                              typename Flavor::FF beta,
                                                              typename Flavor::FF gamma)
{
    using barretenberg::polynomial_arithmetic::copy_polynomial;
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    auto wire_polynomials = key->get_wires();

    // TODO(luke): instantiate z_perm here then make the 0th accum a span of it? avoid extra memory.

    // Allocate accumulator polynomials that will serve as scratch space
    std::array<Polynomial, Flavor::NUM_WIRES> numerator_accumulator;
    std::array<Polynomial, Flavor::NUM_WIRES> denominator_accumulator;
    for (size_t i = 0; i < Flavor::NUM_WIRES; ++i) {
        numerator_accumulator[i] = Polynomial{ key->circuit_size };
        denominator_accumulator[i] = Polynomial{ key->circuit_size };
    }

    // Populate wire and permutation polynomials
    auto ids = key->get_id_polynomials();
    auto sigmas = key->get_sigma_polynomials();

    // Step (1)
    // TODO(#222)(kesha): Change the order to engage automatic prefetching and get rid of redundant computation
    for (size_t i = 0; i < key->circuit_size; ++i) {
        for (size_t k = 0; k < Flavor::NUM_WIRES; ++k) {
            numerator_accumulator[k][i] = wire_polynomials[k][i] + (ids[k][i] * beta) + gamma; // w_k(i) + β.id_k(i) + γ
            denominator_accumulator[k][i] =
                wire_polynomials[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
        }
    }

    // Step (2)
    for (size_t k = 0; k < Flavor::NUM_WIRES; ++k) {
        for (size_t i = 0; i < key->circuit_size - 1; ++i) {
            numerator_accumulator[k][i + 1] *= numerator_accumulator[k][i];
            denominator_accumulator[k][i + 1] *= denominator_accumulator[k][i];
        }
    }

    // Step (3)
    for (size_t i = 0; i < key->circuit_size; ++i) {
        for (size_t k = 1; k < Flavor::NUM_WIRES; ++k) {
            numerator_accumulator[0][i] *= numerator_accumulator[k][i];
            denominator_accumulator[0][i] *= denominator_accumulator[k][i];
        }
    }

    // Step (4)
    // Use Montgomery batch inversion to compute z_perm[i+1] = numerator_accumulator[0][i] /
    // denominator_accumulator[0][i]. At the end of this computation, the quotient numerator_accumulator[0] /
    // denominator_accumulator[0] is stored in numerator_accumulator[0].
    // Note: Since numerator_accumulator[0][i] corresponds to z_lookup[i+1], we only iterate up to i = (n - 2).
    FF* inversion_coefficients = &denominator_accumulator[1][0]; // arbitrary scratch space
    FF inversion_accumulator = FF::one();
    for (size_t i = 0; i < key->circuit_size - 1; ++i) {
        inversion_coefficients[i] = numerator_accumulator[0][i] * inversion_accumulator;
        inversion_accumulator *= denominator_accumulator[0][i];
    }
    inversion_accumulator = inversion_accumulator.invert(); // perform single inversion per thread
    for (size_t i = key->circuit_size - 2; i != std::numeric_limits<size_t>::max(); --i) {
        numerator_accumulator[0][i] = inversion_accumulator * inversion_coefficients[i];
        inversion_accumulator *= denominator_accumulator[0][i];
    }

    // Construct permutation polynomial 'z_perm' in lagrange form as:
    // z_perm = [0 numerator_accumulator[0][0] numerator_accumulator[0][1] ... numerator_accumulator[0][n-2] 0]
    Polynomial z_perm(key->circuit_size);
    // Initialize 0th coefficient to 0 to ensure z_perm is left-shiftable via division by X in gemini
    z_perm[0] = 0;
    copy_polynomial(numerator_accumulator[0].data(), &z_perm[1], key->circuit_size - 1, key->circuit_size - 1);

    return z_perm;
}

/**
 * @brief Compute the lookup grand product polynomial Z_lookup(X).
 *
 * @details The lookup grand product polynomial Z_lookup is of the form
 *
 *                   ∏(1 + β) ⋅ ∏(q_lookup*f_k + γ) ⋅ ∏(t_k + βt_{k+1} + γ(1 + β))
 * Z_lookup(X_j) = -----------------------------------------------------------------
 *                                   ∏(s_k + βs_{k+1} + γ(1 + β))
 *
 * where ∏ := ∏_{k<j}. This polynomial is constructed in evaluation form over the course
 * of three steps:
 *
 * Step 1) Compute polynomials f, t and s and incorporate them into terms that are ultimately needed
 * to construct the grand product polynomial Z_lookup(X):
 * Note 1: In what follows, 't' is associated with table values (and is not to be confused with the
 * quotient polynomial, also refered to as 't' elsewhere). Polynomial 's' is the sorted  concatenation
 * of the witnesses and the table values.
 * Note 2: Evaluation at Xω is indicated explicitly, e.g. 'p(Xω)'; evaluation at X is simply omitted, e.g. 'p'
 *
 * 1a.   Compute f, then set accumulators[0] = (q_lookup*f + γ), where
 *
 *         f = (w_1 + q_2*w_1(Xω)) + η(w_2 + q_m*w_2(Xω)) + η²(w_3 + q_c*w_3(Xω)) + η³q_index.
 *      Note that q_2, q_m, and q_c are just the selectors from Standard Plonk that have been repurposed
 *      in the context of the plookup gate to represent 'shift' values. For example, setting each of the
 *      q_* in f to 2^8 facilitates operations on 32-bit values via four operations on 8-bit values. See
 *      Ultra documentation for details.
 *
 * 1b.   Compute t, then set accumulators[1] = (t + βt(Xω) + γ(1 + β)), where t = t_1 + ηt_2 + η²t_3 + η³t_4
 *
 * 1c.   Set accumulators[2] = (1 + β)
 *
 * 1d.   Compute s, then set accumulators[3] = (s + βs(Xω) + γ(1 + β)), where s = s_1 + ηs_2 + η²s_3 + η³s_4
 *
 * Step 2) Compute the constituent product components of Z_lookup(X).
 * Let ∏ := Prod_{k<j}. Let f_k, t_k and s_k now represent the k'th component of the polynomials f,t and s
 * defined above. We compute the following four product polynomials needed to construct the grand product
 * Z_lookup(X).
 * 1.   accumulators[0][j] = ∏ (q_lookup*f_k + γ)
 * 2.   accumulators[1][j] = ∏ (t_k + βt_{k+1} + γ(1 + β))
 * 3.   accumulators[2][j] = ∏ (1 + β)
 * 4.   accumulators[3][j] = ∏ (s_k + βs_{k+1} + γ(1 + β))
 *
 * Step 3) Combine the accumulator product elements to construct Z_lookup(X).
 *
 *                      ∏ (1 + β) ⋅ ∏ (q_lookup*f_k + γ) ⋅ ∏ (t_k + βt_{k+1} + γ(1 + β))
 *  Z_lookup(g^j) = --------------------------------------------------------------------------
 *                                      ∏ (s_k + βs_{k+1} + γ(1 + β))
 *
 * Note: Montgomery batch inversion is used to efficiently compute the coefficients of Z_lookup
 * rather than peforming n individual inversions. I.e. we first compute the double product P_n:
 *
 * P_n := ∏_{j<n} ∏_{k<j} S_k, where S_k = (s_k + βs_{k+1} + γ(1 + β))
 *
 * and then compute the inverse on P_n. Then we work back to front to obtain terms of the form
 * 1/∏_{k<i} S_i that appear in Z_lookup, using the fact that P_i/P_{i+1} = 1/∏_{k<i} S_i. (Note
 * that once we have 1/P_n, we can compute 1/P_{n-1} as (1/P_n) * ∏_{k<n} S_i, and
 * so on).
 *
 * @param key proving key
 * @param wire_polynomials
 * @param sorted_list_accumulator
 * @param eta
 * @param beta
 * @param gamma
 * @return Polynomial
 */
template <typename Flavor>
typename Flavor::Polynomial compute_lookup_grand_product(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                         typename Flavor::FF eta,
                                                         typename Flavor::FF beta,
                                                         typename Flavor::FF gamma)

{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    auto sorted_list_accumulator = key->sorted_accum;

    const FF eta_sqr = eta.sqr();
    const FF eta_cube = eta_sqr * eta;

    const size_t circuit_size = key->circuit_size;

    // Allocate 4 length n 'accumulator' polynomials. accumulators[0] will be used to construct
    // z_lookup in place.
    // Note: The magic number 4 here comes from the structure of the grand product and is not related to the program
    // width.
    std::array<Polynomial, 4> accumulators;
    for (size_t i = 0; i < 4; ++i) {
        accumulators[i] = Polynomial{ key->circuit_size };
    }

    // Obtain column step size values that have been stored in repurposed selctors
    std::span<const FF> column_1_step_size = key->q_r;
    std::span<const FF> column_2_step_size = key->q_m;
    std::span<const FF> column_3_step_size = key->q_c;

    // We utilize three wires even when more are available.
    // TODO(#389): const correctness
    std::array<std::span<FF>, 3> wires = key->get_table_column_wires();

    // Note: the number of table polys is related to program width but '4' is the only value supported
    std::array<std::span<const FF>, 4> tables{
        key->table_1,
        key->table_2,
        key->table_3,
        key->table_4,
    };

    std::span<const FF> lookup_selector = key->q_lookup;
    std::span<const FF> lookup_index_selector = key->q_o;

    const FF beta_plus_one = beta + FF(1);                      // (1 + β)
    const FF gamma_times_beta_plus_one = gamma * beta_plus_one; // γ(1 + β)

    // Step (1)

    FF T0; // intermediate value for various calculations below
    // Note: block_mask is used for efficient modulus, i.e. i % N := i & (N-1), for N = 2^k
    const size_t block_mask = circuit_size - 1;
    // Initialize 't(X)' to be used in an expression of the form t(X) + β*t(Xω)
    FF next_table = tables[0][0] + tables[1][0] * eta + tables[2][0] * eta_sqr + tables[3][0] * eta_cube;

    for (size_t i = 0; i < circuit_size; ++i) {

        // Compute i'th element of f via Horner (see definition of f above)
        T0 = lookup_index_selector[i];
        T0 *= eta;
        T0 += wires[2][(i + 1) & block_mask] * column_3_step_size[i];
        T0 += wires[2][i];
        T0 *= eta;
        T0 += wires[1][(i + 1) & block_mask] * column_2_step_size[i];
        T0 += wires[1][i];
        T0 *= eta;
        T0 += wires[0][(i + 1) & block_mask] * column_1_step_size[i];
        T0 += wires[0][i];
        T0 *= lookup_selector[i];

        // Set i'th element of polynomial q_lookup*f + γ
        accumulators[0][i] = T0;
        accumulators[0][i] += gamma;

        // Compute i'th element of t via Horner
        T0 = tables[3][(i + 1) & block_mask];
        T0 *= eta;
        T0 += tables[2][(i + 1) & block_mask];
        T0 *= eta;
        T0 += tables[1][(i + 1) & block_mask];
        T0 *= eta;
        T0 += tables[0][(i + 1) & block_mask];

        // Set i'th element of polynomial (t + βt(Xω) + γ(1 + β))
        accumulators[1][i] = T0 * beta + next_table;
        next_table = T0;
        accumulators[1][i] += gamma_times_beta_plus_one;

        // Set value of this accumulator to (1 + β)
        accumulators[2][i] = beta_plus_one;

        // Set i'th element of polynomial (s + βs(Xω) + γ(1 + β))
        accumulators[3][i] = sorted_list_accumulator[(i + 1) & block_mask];
        accumulators[3][i] *= beta;
        accumulators[3][i] += sorted_list_accumulator[i];
        accumulators[3][i] += gamma_times_beta_plus_one;
    }

    // Step (2)

    // Note: This is a small multithreading bottleneck, as we have only 4 parallelizable processes.
    for (auto& accum : accumulators) {
        for (size_t i = 0; i < circuit_size - 1; ++i) {
            accum[i + 1] *= accum[i];
        }
    }

    // Step (3)

    // Compute <Z_lookup numerator> * ∏_{j<i}∏_{k<j}S_k
    FF inversion_accumulator = FF::one();
    for (size_t i = 0; i < circuit_size - 1; ++i) {
        accumulators[0][i] *= accumulators[2][i];
        accumulators[0][i] *= accumulators[1][i];
        accumulators[0][i] *= inversion_accumulator;
        inversion_accumulator *= accumulators[3][i];
    }
    inversion_accumulator = inversion_accumulator.invert(); // invert

    // Compute [Z_lookup numerator] * ∏_{j<i}∏_{k<j}S_k / ∏_{j<i+1}∏_{k<j}S_k = <Z_lookup numerator> /
    // ∏_{k<i}S_k
    for (size_t i = circuit_size - 2; i != std::numeric_limits<size_t>::max(); --i) {
        // N.B. accumulators[0][i] = z_lookup[i + 1]
        accumulators[0][i] *= inversion_accumulator;
        inversion_accumulator *= accumulators[3][i];
    }

    Polynomial z_lookup(key->circuit_size);
    // Initialize 0th coefficient to 0 to ensure z_perm is left-shiftable via division by X in gemini
    z_lookup[0] = FF::zero();
    barretenberg::polynomial_arithmetic::copy_polynomial(
        accumulators[0].data(), &z_lookup[1], key->circuit_size - 1, key->circuit_size - 1);

    return z_lookup;
}

/**
 * @brief Construct sorted list accumulator polynomial 's'.
 *
 * @details Compute s = s_1 + η*s_2 + η²*s_3 + η³*s_4 (via Horner) where s_i are the
 * sorted concatenated witness/table polynomials
 *
 * @param key proving key
 * @param sorted_list_polynomials sorted concatenated witness/table polynomials
 * @param eta random challenge
 * @return Polynomial
 */
template <typename Flavor>
typename Flavor::Polynomial compute_sorted_list_accumulator(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                            typename Flavor::FF eta)
{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    const size_t circuit_size = key->circuit_size;

    auto sorted_list_accumulator = Polynomial{ circuit_size };

    auto sorted_polynomials = key->get_sorted_polynomials();

    // Construct s via Horner, i.e. s = s_1 + η(s_2 + η(s_3 + η*s_4))
    for (size_t i = 0; i < circuit_size; ++i) {
        FF T0 = sorted_polynomials[3][i];
        T0 *= eta;
        T0 += sorted_polynomials[2][i];
        T0 *= eta;
        T0 += sorted_polynomials[1][i];
        T0 *= eta;
        T0 += sorted_polynomials[0][i];
        sorted_list_accumulator[i] = T0;
    }

    return sorted_list_accumulator;
}

/**
 * @brief Add plookup memory records to the fourth wire polynomial
 *
 * @details This operation must be performed after the first three wires have been committed to, hence the dependence on
 * the `eta` challenge.
 *
 * @tparam Flavor
 * @param eta challenge produced after commitment to first three wire polynomials
 */
template <typename Flavor>
void add_plookup_memory_records_to_wire_4(std::shared_ptr<typename Flavor::ProvingKey>& key, typename Flavor::FF eta)
{
    // The plookup memory record values are computed at the indicated indices as
    // w4 = w3 * eta^3 + w2 * eta^2 + w1 * eta + read_write_flag;
    // (See plookup_auxiliary_widget.hpp for details)
    auto wires = key->get_wires();

    // Compute read record values
    for (const auto& gate_idx : key->memory_read_records) {
        wires[3][gate_idx] += wires[2][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[1][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[0][gate_idx];
        wires[3][gate_idx] *= eta;
    }

    // Compute write record values
    for (const auto& gate_idx : key->memory_write_records) {
        wires[3][gate_idx] += wires[2][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[1][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[0][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += 1;
    }
}

template honk::flavor::Standard::Polynomial compute_permutation_grand_product<honk::flavor::Standard>(
    std::shared_ptr<honk::flavor::Standard::ProvingKey>&, honk::flavor::Standard::FF, honk::flavor::Standard::FF);

template honk::flavor::Ultra::Polynomial compute_permutation_grand_product<honk::flavor::Ultra>(
    std::shared_ptr<honk::flavor::Ultra::ProvingKey>&, honk::flavor::Ultra::FF, honk::flavor::Ultra::FF);

template typename honk::flavor::Ultra::Polynomial compute_lookup_grand_product<honk::flavor::Ultra>(
    std::shared_ptr<typename honk::flavor::Ultra::ProvingKey>& key,
    typename honk::flavor::Ultra::FF eta,
    typename honk::flavor::Ultra::FF beta,
    typename honk::flavor::Ultra::FF gamma);

template typename honk::flavor::Ultra::Polynomial compute_sorted_list_accumulator<honk::flavor::Ultra>(
    std::shared_ptr<typename honk::flavor::Ultra::ProvingKey>& key, typename honk::flavor::Ultra::FF eta);

template void add_plookup_memory_records_to_wire_4<honk::flavor::Ultra>(
    std::shared_ptr<typename honk::flavor::Ultra::ProvingKey>& key, typename honk::flavor::Ultra::FF eta);

} // namespace proof_system::honk::prover_library
