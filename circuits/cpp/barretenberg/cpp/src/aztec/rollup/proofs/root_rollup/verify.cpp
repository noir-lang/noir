#include "./verify.hpp"
#include "create_root_rollup_tx.hpp"
#include "./root_rollup_circuit.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types;

namespace {
verify_result build_circuit(Composer& composer, root_rollup_tx& tx, circuit_data const& circuit_data)
{
    verify_result result;

    if (!circuit_data.inner_rollup_circuit_data.verification_key) {
        info("Inner verification key not provided.");
        return result;
    }

    if (circuit_data.inner_rollup_circuit_data.padding_proof.size() == 0) {
        info("Inner padding proof not provided.");
        return result;
    }

    // Pad the rollup if necessary.
    pad_root_rollup_tx(tx, circuit_data);

    auto circuit_result = root_rollup_circuit(composer,
                                              tx,
                                              circuit_data.inner_rollup_circuit_data.rollup_size,
                                              circuit_data.rollup_size,
                                              circuit_data.inner_rollup_circuit_data.verification_key);

    result.recursion_output = circuit_result.recursion_output;
    result.broadcast_data = circuit_result.broadcast_data;

    return result;
}
} // namespace

verify_result verify_logic(root_rollup_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_logic_internal(composer, tx, cd, "root rollup", build_circuit);
}

verify_result verify(root_rollup_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer, tx, cd, "root rollup", true, build_circuit);
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
