#pragma once                       // just adding these willy-nilly
#include <ecc/curves/bn254/fr.hpp> // just added to get info()

namespace honk {
// starting with arithmetic and permutation constraints
// Note: including Z_PERM_SHIFT here is probably not the right thing long term but
// doing it for now for convenience
enum MULTIVARIATE {
    W_L,
    W_R,
    W_O,
    Z_PERM,
    Z_PERM_SHIFT,
    Q_M,
    Q_L,
    Q_R,
    Q_O,
    Q_C,
    SIGMA_1,
    SIGMA_2,
    SIGMA_3,
    ID_1,
    ID_2,
    ID_3,
    LAGRANGE_1,
    COUNT
};
static constexpr size_t NUM_MULTIVARIATES = MULTIVARIATE::COUNT;

namespace sumcheck {
/**
 *
 * @brief A container for all of the Honk! polynomials (e.g., wire and selector polynomials).
 * These polynomials all low-degree extensions over H^d with H = {0, 1} (where d =
 * ceil(log(number of gates))), hence they are multilinear polynomials in d variables. As such, it is efficient to store
 * these polynomials in terms of univariate degree-1 polynomials. We call such a polynomial an Edge, short for

 ... TODO(cody)rewrite!

 * Suppose now the Honk polynomials (multilinear in d variables) are called P_1, ..., P_N. At initialization,
 * we think of these as lying in a two-dimensional array, where each column records the value of one P_i on H^d. In this
 * array, each row contains N edge polynomials (an EdgeGroup). Hence the array has shape (n/2, N). After the first
 * round, the array will be updated ('folded'), so that the first n/4 rows will represent the evaluations P_i(X1, ...,
 * X_{d-1}, u_d) as a low-degree extension on H^{d-1}.
 *
 * @tparam Fr
 *
 * NOTE: With ~40 columns, prob only want to allocate 256 EdgeGroup's at once to keep stack under 1MB?
 * TODO: might want to just do C-style multidimensional array? for guaranteed adjacency?
 * NOTE: got rid of `populate` method--just assuming the EdgeGroup's are constructed at the time that the
 *       Multivariatess instance is constructed
 */
template <class Fr, size_t num_polys, size_t num_vars> class Multivariates {
  public:
    const static size_t multivariate_d = num_vars;
    const static size_t multivariate_n = 1 << num_vars;
    static constexpr size_t num = num_polys;

    std::array<std::array<Fr, (multivariate_n >> 1)>, num_polys> folded_polynomials;
    std::array<Fr*, num_polys> full_polynomials;

    /* For groups, we imagine all of the defining polynomial data in
       a matrix like this:
                   | P_1 | P_2 | P_3 | P_4 | ... | P_N | N = NUM_HONK_POLYS
                   |-----------------------------------|
         group 0 --|  *  |  *  |  *  |  *  | ... |  *  | vertex 0
                 \-|  *  |  *  |  *  |  *  | ... |  *  | vertex 1
         group 1 --|  *  |  *  |  *  |  *  | ... |  *  | vertex 2
                 \-|  *  |  *  |  *  |  *  | ... |  *  | vertex 3
                   |  *  |  *  |  *  |  *  | ... |  *  |
       group m-1 --|  *  |  *  |  *  |  *  | ... |  *  | vertex n-2
                 \-|  *  |  *  |  *  |  *  | ... |  *  | vertex n-1
        m = n/2

        In practice, we record this value in an n/2 x N matrix groups
        where each row is though of as a group of edge polynomials.

     */
    Multivariates() = default;

    Multivariates(std::array<Fr*, num_polys> full_polynomials)
        : full_polynomials(full_polynomials){};

    /**
     * @brief Evaluate at the round challenge and prepare class for next round.
     * Illustration of layout in example of first round when d==3 (showing just one Honk polynomial,
     * i.e., what happens in just one column of our two-dimensional array):
     *
     * groups    vertex terms              collected vertex terms               groups after folding
     *     g0 -- v0 (1-X1)(1-X2)(1-X3) --- (v0(1-X3) + v1 X3) (1-X1)(1-X2) ---- (v0(1-u3) + v1 u3) (1-X1)(1-X2)
     *        \- v1 (1-X1)(1-X2)  X3   --/                                  --- (v2(1-u3) + v3 u3) (1-X1)  X2
     *     g1 -- v2 (1-X1)  X2  (1-X3) --- (v1(1-X3) + v2 X3) (1-X1)  X2  -/ -- (v4(1-u3) + v5 u3)   X1  (1-X2)
     *        \- v3 (1-X1)  X2    X3   --/                                  / - (v6(1-u3) + v7 u3)   X1    X2
     *     g2 -- v4   X1  (1-X2)(1-X3) --- (v3(1-X3) + v4 X3)   X1  (1-X2)-/ /
     *        \- v5   X1  (1-X2)  X3   --/                                  /
     *     g3 -- v6   X1    X2  (1-X3) --- (v5(1-X3) + v6 X3)   X1    X2  -/
     *        \- v7   X1    X2    X3   --/
     *
     *  TODO: Is it better to avoid copying in the to get the third column? could maybe do by
     * just tracking a gap parameter in the for loop, e.g. EDGE_GAP = (1 << i).
     * @param challenge
     */

    void fold(size_t round_size, const Fr& challenge)
    {
        for (size_t j = 0; j < num_polys; ++j) {
            for (size_t i = 0; i < round_size; i += 2) {
                // old: a0, a1
                // new: (1 - r).a0 + r.a1
                // => r.(a1 - a0) + a0
                folded_polynomials[j][i >> 1] =
                    folded_polynomials[j][i] + challenge * (folded_polynomials[j][i + 1] - folded_polynomials[j][i]);
            }
        }
    }

    void fold_first_round(size_t round_size, const Fr& challenge)
    {
        for (size_t j = 0; j < num_polys; ++j) {
            for (size_t i = 0; i < round_size; i += 2) {
                // old: a0, a1
                // new: (1 - r).a0 + r.a1
                // => r.(a1 - a0) + a0
                folded_polynomials[j][i >> 1] =
                    full_polynomials[j][i] + challenge * (full_polynomials[j][i + 1] - full_polynomials[j][i]);
            }
        }
    }

    // TODO(cody): Double check edge case here and above. For now, test_fold_1
    // seems to show that the round_size==2 case is handled correctly
};
} // namespace sumcheck
} // namespace honk
