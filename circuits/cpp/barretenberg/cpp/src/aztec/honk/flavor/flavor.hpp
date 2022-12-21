#pragma once
#include <common/log.hpp>

class StandardArithmetization {
  public:
    enum POLYNOMIAL {
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

    static constexpr size_t NUM_POLYNOMIALS = POLYNOMIAL::COUNT;
};

namespace honk::sumcheck { // TODO(Cody): get namespaces right here
class StandardHonk {
  public:
    using Arithmetization = StandardArithmetization;
    using MULTIVARIATE = Arithmetization::POLYNOMIAL;
    static constexpr size_t MAX_RELATION_LENGTH = 5; // TODO(Cody): increment after fixing add_edge_contribution; kill
                                                     // after moving barycentric class out of relations

    // TODO(Cody): should extract this from the parameter pack. Maybe that should be done here?
};
} // namespace honk::sumcheck