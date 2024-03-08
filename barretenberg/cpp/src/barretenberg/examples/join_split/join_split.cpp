#include "join_split.hpp"
#include "barretenberg/examples/join_split/types.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "join_split_circuit.hpp"

namespace bb::join_split_example::proofs::join_split {

using namespace bb::plonk;
using namespace bb::crypto::merkle_tree;

Builder new_join_split_circuit(join_split_tx const& tx)
{
    Builder builder;
    join_split_circuit(builder, tx);

    if (builder.failed()) {
        std::string error = format("builder logic failed: ", builder.err());
        throw_or_abort(error);
    }

    info("public inputs: ", builder.public_inputs.size());

    info("num gates before finalization: ", builder.get_num_gates());

    return builder;
}

} // namespace bb::join_split_example::proofs::join_split
