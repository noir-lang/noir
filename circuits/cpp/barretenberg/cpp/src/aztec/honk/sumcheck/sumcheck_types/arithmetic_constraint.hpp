#include <stddef.h>
#include <array>
#include <tuple>
#include "./constraint.hpp"
#include "./multivariates.hpp"
#include "./barycentric_data.hpp"
#include "./univariate.hpp"
#include "./challenge_container.hpp"
#include "../transcript.hpp"
#include <span>
#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

namespace honk {
namespace sumcheck {

template <typename Fr> class ArithmeticConstraint : public Constraint<Fr> {
  public:
    static constexpr size_t CONSTRAINT_LENGTH = 4; // degree of this constraint + 1
    using Constraint<Fr>::HONK_CONSTRAINT_LENGTH;
    using Constraint<Fr>::NUM_HONK_MULTIVARIATES;
    // Will be used after adding all edge contributions to extend
    // FUTURE OPTIMIZATION: successively extend as needed?
    BarycentricData<Fr, CONSTRAINT_LENGTH, HONK_CONSTRAINT_LENGTH> barycentric =
        BarycentricData<Fr, CONSTRAINT_LENGTH, HONK_CONSTRAINT_LENGTH>();

    using UnivariateClass = Univariate<Fr, CONSTRAINT_LENGTH>;

    // using UnivariateView = UnivariateView<Fr, CONSTRAINT_LENGTH>;

    ArithmeticConstraint() = default;

    // TODO: optimize! Karatsuba in general, at least for some degrees?
    //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both
    void add_edge_contribution(auto edge_extensions, Univariate<Fr, CONSTRAINT_LENGTH>& evals)
    {
        auto w_l = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::W_L]);
        auto w_r = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::W_R]);
        auto w_o = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::W_O]);
        auto q_m = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::Q_M]);
        auto q_l = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::Q_L]);
        auto q_r = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::Q_R]);
        auto q_o = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::Q_O]);
        auto q_c = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::Q_C]);

        evals += w_l * (q_m * w_r + q_l);
        evals += q_r * w_r;
        evals += q_o * w_o;
        evals += q_c;
    };

    void add_full_constraint_value_contribution(std::array<Fr, NUM_HONK_MULTIVARIATES> purported_evaluations,
                                                Fr& full_honk_constraint_value)
    {

        auto w_l = purported_evaluations[MULTIVARIATE::W_L];
        auto w_r = purported_evaluations[MULTIVARIATE::W_R];
        auto w_o = purported_evaluations[MULTIVARIATE::W_O];
        auto q_m = purported_evaluations[MULTIVARIATE::Q_M];
        auto q_l = purported_evaluations[MULTIVARIATE::Q_L];
        auto q_r = purported_evaluations[MULTIVARIATE::Q_R];
        auto q_o = purported_evaluations[MULTIVARIATE::Q_O];
        auto q_c = purported_evaluations[MULTIVARIATE::Q_C];
        full_honk_constraint_value += w_l * (q_m * w_r + q_l);
        full_honk_constraint_value += q_r * w_r;
        full_honk_constraint_value += q_o * w_o;
        full_honk_constraint_value += q_c;
    };
};
} // namespace sumcheck
} // namespace honk
