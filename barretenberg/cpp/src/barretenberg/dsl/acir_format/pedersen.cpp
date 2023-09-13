#include "pedersen.hpp"

namespace acir_format {

using namespace proof_system::plonk;

void create_pedersen_constraint(Builder& builder, const PedersenConstraint& input)
{
    std::vector<field_ct> scalars;

    for (const auto& scalar : input.scalars) {
        // convert input indices to field_ct
        field_ct scalar_as_field = field_ct::from_witness_index(&builder, scalar);
        scalars.push_back(scalar_as_field);
    }

    // TODO: Does Noir need additive homomorphic Pedersen hash? If so, using plookup version won't help.
    auto point = stdlib::pedersen_plookup_commitment<Builder>::commit(scalars, input.hash_index);

    builder.assert_equal(point.x.witness_index, input.result_x);
    builder.assert_equal(point.y.witness_index, input.result_y);
}

} // namespace acir_format
