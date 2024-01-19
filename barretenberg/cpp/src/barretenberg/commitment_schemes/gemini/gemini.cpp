
#include "gemini.hpp"
#include "barretenberg/common/thread.hpp"

#include <bit>
#include <memory>
#include <vector>

/**
 * @brief Protocol for opening several multi-linear polynomials at the same point.
 *
 *
 * m = number of variables
 * n = 2ᵐ
 * u = (u₀,...,uₘ₋₁)
 * f₀, …, fₖ₋₁ = multilinear polynomials,
 * g₀, …, gₕ₋₁ = shifted multilinear polynomial,
 *  Each gⱼ is the left-shift of some f↺ᵢ, and gⱼ points to the same memory location as fᵢ.
 * v₀, …, vₖ₋₁, v↺₀, …, v↺ₕ₋₁ = multilinear evalutions s.t. fⱼ(u) = vⱼ, and gⱼ(u) = f↺ⱼ(u) = v↺ⱼ
 *
 * We use a challenge ρ to create a random linear combination of all fⱼ,
 * and actually define A₀ = F + G↺, where
 *   F  = ∑ⱼ ρʲ fⱼ
 *   G  = ∑ⱼ ρᵏ⁺ʲ gⱼ,
 *   G↺ = is the shift of G
 * where fⱼ is normal, and gⱼ is shifted.
 * The evaluations are also batched, and
 *   v  = ∑ ρʲ⋅vⱼ + ∑ ρᵏ⁺ʲ⋅v↺ⱼ = F(u) + G↺(u)
 *
 * The prover then creates the folded polynomials A₀, ..., Aₘ₋₁,
 * and opens them at different points, as univariates.
 *
 * We open A₀ as univariate at r and -r.
 * Since A₀ = F + G↺, but the verifier only has commitments to the gⱼs,
 * we need to partially evaluate A₀ at both evaluation points.
 * As univariate, we have
 *  A₀(X) = F(X) + G↺(X) = F(X) + G(X)/X
 * So we define
 *  - A₀₊(X) = F(X) + G(X)/r
 *  - A₀₋(X) = F(X) − G(X)/r
 * So that A₀₊(r) = A₀(r) and A₀₋(-r) = A₀(-r).
 * The verifier is able to computed the simulated commitments to A₀₊(X) and A₀₋(X)
 * since they are linear-combinations of the commitments [fⱼ] and [gⱼ].
 */
namespace bb::honk::pcs::gemini {

/**
 * @brief Computes d-1 fold polynomials Fold_i, i = 1, ..., d-1
 *
 * @param mle_opening_point multilinear opening point 'u'
 * @param batched_unshifted F(X) = ∑ⱼ ρʲ   fⱼ(X)
 * @param batched_to_be_shifted G(X) = ∑ⱼ ρᵏ⁺ʲ gⱼ(X)
 * @return std::vector<Polynomial>
 */
template <typename Curve>
std::vector<typename bb::Polynomial<typename Curve::ScalarField>> GeminiProver_<Curve>::compute_gemini_polynomials(
    std::span<const Fr> mle_opening_point, Polynomial&& batched_unshifted, Polynomial&& batched_to_be_shifted)
{
    const size_t num_variables = mle_opening_point.size(); // m

    const size_t num_threads = get_num_cpus_pow2();
    constexpr size_t efficient_operations_per_thread = 64; // A guess of the number of operation for which there
                                                           // would be a point in sending them to a separate thread

    // Allocate space for m+1 Fold polynomials
    //
    // The first two are populated here with the batched unshifted and to-be-shifted polynomial respectively.
    // They will eventually contain the full batched polynomial A₀ partially evaluated at the challenges r,-r.
    // This function populates the other m-1 polynomials with the foldings of A₀.
    std::vector<Polynomial> gemini_polynomials;
    gemini_polynomials.reserve(num_variables + 1);

    // F(X) = ∑ⱼ ρʲ fⱼ(X) and G(X) = ∑ⱼ ρᵏ⁺ʲ gⱼ(X)
    Polynomial& batched_F = gemini_polynomials.emplace_back(std::move(batched_unshifted));
    Polynomial& batched_G = gemini_polynomials.emplace_back(std::move(batched_to_be_shifted));
    constexpr size_t offset_to_folded = 2; // Offset because of F an G
    // A₀(X) = F(X) + G↺(X) = F(X) + G(X)/X.
    Polynomial A_0 = batched_F;
    A_0 += batched_G.shifted();

    // Allocate everything before parallel computation
    for (size_t l = 0; l < num_variables - 1; ++l) {
        // size of the previous polynomial/2
        const size_t n_l = 1 << (num_variables - l - 1);

        // A_l_fold = Aₗ₊₁(X) = (1-uₗ)⋅even(Aₗ)(X) + uₗ⋅odd(Aₗ)(X)
        gemini_polynomials.emplace_back(Polynomial(n_l));
    }

    // A_l = Aₗ(X) is the polynomial being folded
    // in the first iteration, we take the batched polynomial
    // in the next iteration, it is the previously folded one
    auto A_l = A_0.data();
    for (size_t l = 0; l < num_variables - 1; ++l) {
        // size of the previous polynomial/2
        const size_t n_l = 1 << (num_variables - l - 1);

        // Use as many threads as it is useful so that 1 thread doesn't process 1 element, but make sure that there is
        // at least 1
        size_t num_used_threads = std::min(n_l / efficient_operations_per_thread, num_threads);
        num_used_threads = num_used_threads ? num_used_threads : 1;
        size_t chunk_size = n_l / num_used_threads;
        size_t last_chunk_size = (n_l % chunk_size) ? (n_l % num_used_threads) : chunk_size;

        // Openning point is the same for all
        const Fr u_l = mle_opening_point[l];

        // A_l_fold = Aₗ₊₁(X) = (1-uₗ)⋅even(Aₗ)(X) + uₗ⋅odd(Aₗ)(X)
        auto A_l_fold = gemini_polynomials[l + offset_to_folded].data();

        parallel_for(num_used_threads, [&](size_t i) {
            size_t current_chunk_size = (i == (num_used_threads - 1)) ? last_chunk_size : chunk_size;
            for (std::ptrdiff_t j = (std::ptrdiff_t)(i * chunk_size);
                 j < (std::ptrdiff_t)((i * chunk_size) + current_chunk_size);
                 j++) {
                // fold(Aₗ)[j] = (1-uₗ)⋅even(Aₗ)[j] + uₗ⋅odd(Aₗ)[j]
                //            = (1-uₗ)⋅Aₗ[2j]      + uₗ⋅Aₗ[2j+1]
                //            = Aₗ₊₁[j]
                A_l_fold[j] = A_l[j << 1] + u_l * (A_l[(j << 1) + 1] - A_l[j << 1]);
            }
        });
        // set Aₗ₊₁ = Aₗ for the next iteration
        A_l = A_l_fold;
    }

    return gemini_polynomials;
};

/**
 * @brief Computes/aggragates d+1 Fold polynomials and their opening pairs (challenge, evaluation)
 *
 * @details This function assumes that, upon input, last d-1 entries in gemini_polynomials are Fold_i.
 * The first two entries are assumed to be, respectively, the batched unshifted and batched to-be-shifted
 * polynomials F(X) = ∑ⱼ ρʲfⱼ(X) and G(X) = ∑ⱼ ρᵏ⁺ʲ gⱼ(X). This function completes the computation
 * of the first two Fold polynomials as F + G/r and F - G/r. It then evaluates each of the d+1
 * fold polynomials at, respectively, the points r, rₗ = r^{2ˡ} for l = 0, 1, ..., d-1.
 *
 * @param mle_opening_point u = (u₀,...,uₘ₋₁) is the MLE opening point
 * @param gemini_polynomials vector of polynomials whose first two elements are F(X) = ∑ⱼ ρʲfⱼ(X)
 * and G(X) = ∑ⱼ ρᵏ⁺ʲ gⱼ(X), and the next d-1 elements are Fold_i, i = 1, ..., d-1.
 * @param r_challenge univariate opening challenge
 */
template <typename Curve>
ProverOutput<Curve> GeminiProver_<Curve>::compute_fold_polynomial_evaluations(
    std::span<const Fr> mle_opening_point, std::vector<Polynomial>&& gemini_polynomials, const Fr& r_challenge)
{
    const size_t num_variables = mle_opening_point.size(); // m

    Polynomial& batched_F = gemini_polynomials[0]; // F(X) = ∑ⱼ ρʲ   fⱼ(X)
    Polynomial& batched_G = gemini_polynomials[1]; // G(X) = ∑ⱼ ρᵏ⁺ʲ gⱼ(X)

    // Compute univariate opening queries rₗ = r^{2ˡ} for l = 0, 1, ..., m-1
    std::vector<Fr> r_squares = squares_of_r(r_challenge, num_variables);

    // Compute G/r
    Fr r_inv = r_challenge.invert();
    batched_G *= r_inv;

    // Construct A₀₊ = F + G/r and A₀₋ = F - G/r in place in gemini_polynomials
    Polynomial tmp = batched_F;
    Polynomial& A_0_pos = gemini_polynomials[0];

    // A₀₊(X) = F(X) + G(X)/r, s.t. A₀₊(r) = A₀(r)
    A_0_pos += batched_G;

    // Perform a swap so that tmp = G(X)/r and A_0_neg = F(X)
    std::swap(tmp, batched_G);
    Polynomial& A_0_neg = gemini_polynomials[1];

    // A₀₋(X) = F(X) - G(X)/r, s.t. A₀₋(-r) = A₀(-r)
    A_0_neg -= tmp;

    std::vector<OpeningPair<Curve>> fold_poly_opening_pairs;
    fold_poly_opening_pairs.reserve(num_variables + 1);

    // Compute first opening pair {r, A₀(r)}
    fold_poly_opening_pairs.emplace_back(
        OpeningPair<Curve>{ r_challenge, gemini_polynomials[0].evaluate(r_challenge) });

    // Compute the remaining m opening pairs {−r^{2ˡ}, Aₗ(−r^{2ˡ})}, l = 0, ..., m-1.
    for (size_t l = 0; l < num_variables; ++l) {
        fold_poly_opening_pairs.emplace_back(
            OpeningPair<Curve>{ -r_squares[l], gemini_polynomials[l + 1].evaluate(-r_squares[l]) });
    }

    return { fold_poly_opening_pairs, std::move(gemini_polynomials) };
};

template class GeminiProver_<curve::BN254>;
template class GeminiProver_<curve::Grumpkin>;
}; // namespace bb::honk::pcs::gemini
