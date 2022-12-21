#include <array>
#include <tuple>

#include "../../flavor/flavor.hpp"
#include "relation.hpp"
#include "../transcript.hpp"
#include "../polynomials/multivariates.hpp"
#include "../polynomials/barycentric_data.hpp"
#include "../polynomials/univariate.hpp"

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

namespace honk::sumcheck {

template <typename FF> class ArithmeticRelation : public Relation<FF> {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 4;
    using MULTIVARIATE = StandardHonk::MULTIVARIATE; // could just get from StandardArithmetization

    // FUTURE OPTIMIZATION: successively extend as needed?

    // This relation takes no randomness, so it will not receive a ChallengeContainer.
    ArithmeticRelation() = default;
    explicit ArithmeticRelation(auto){}; // NOLINT(readability-named-parameter)

    // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
    //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both
    void add_edge_contribution(auto extended_edges, Univariate<FF, RELATION_LENGTH>& evals)
    {
        auto w_l = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
        auto w_r = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_R]);
        auto w_o = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_O]);
        auto q_m = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_M]);
        auto q_l = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_L]);
        auto q_r = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_R]);
        auto q_o = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_O]);
        auto q_c = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_C]);

        evals += w_l * (q_m * w_r + q_l);
        evals += q_r * w_r;
        evals += q_o * w_o;
        evals += q_c;
    };

    void add_full_relation_value_contribution(auto& purported_evaluations, FF& full_honk_relation_value)
    {

        auto w_l = purported_evaluations[MULTIVARIATE::W_L];
        auto w_r = purported_evaluations[MULTIVARIATE::W_R];
        auto w_o = purported_evaluations[MULTIVARIATE::W_O];
        auto q_m = purported_evaluations[MULTIVARIATE::Q_M];
        auto q_l = purported_evaluations[MULTIVARIATE::Q_L];
        auto q_r = purported_evaluations[MULTIVARIATE::Q_R];
        auto q_o = purported_evaluations[MULTIVARIATE::Q_O];
        auto q_c = purported_evaluations[MULTIVARIATE::Q_C];

        full_honk_relation_value += w_l * (q_m * w_r + q_l);
        full_honk_relation_value += q_r * w_r;
        full_honk_relation_value += q_o * w_o;
        full_honk_relation_value += q_c;
    };
};
} // namespace honk::sumcheck
