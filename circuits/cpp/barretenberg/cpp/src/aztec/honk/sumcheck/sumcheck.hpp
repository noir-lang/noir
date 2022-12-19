#include "./sumcheck_round.hpp"
#include <stddef.h>
#include <array>
#include <tuple>

namespace honk {
namespace sumcheck {
template <class SumcheckTypes, template <class> class... ConstraintPack> class Sumcheck {
    using Fr = typename SumcheckTypes::Fr;
    using Univariate = typename SumcheckTypes::Univariate;
    using EdgeGroup = typename SumcheckTypes::EdgeGroup;
    using HonkPolys = typename SumcheckTypes::HonkPolys;
    using ChallengeContainer = typename SumcheckTypes::ChallengeContainer;
    using Transcript = Transcript<Fr>;

  public:
    static constexpr size_t NUM_CONSTRAINTS = sizeof...(ConstraintPack);

    HonkPolys polynomials;
    SumcheckRound round;
    size_t multivariate_d; // aka num_vars
    size_t multivariate_n;
    Transcript<Fr> transcript;
    ChallengeContainer challenges; // construct with round size, claimed sum

    Sumcheck(HonkPolys polynomials)
        : polynomials(polynomials)
    {
        multivariate_d = polynomials.multivariate_d;
        multivariate_n = 1 << multivariate_d;
        // TODO: construct round, etc.
    };
};
} // namespace sumcheck
} // namespace honk
