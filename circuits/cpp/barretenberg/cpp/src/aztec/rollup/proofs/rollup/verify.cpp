#include "./verify.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace plonk::stdlib::types;

namespace {
verify_result<Composer> build_circuit(Composer& composer, rollup_tx& tx, circuit_data const& cd)
{
    verify_result<Composer> result;

    if (!cd.join_split_circuit_data.verification_key) {
        info("Join split verification key not provided.");
        return result;
    }

    if (cd.join_split_circuit_data.padding_proof.size() == 0) {
        info("Join split padding proof not provided.");
        return result;
    }

    pad_rollup_tx(tx, cd.num_txs, cd.join_split_circuit_data.padding_proof);

    result.recursion_output = rollup_circuit(composer, tx, cd.verification_keys, cd.num_txs);
    return result;
}
} // namespace

verify_result<Composer> verify_logic(rollup_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_logic_internal(composer, tx, cd, "tx rollup", build_circuit);
}

verify_result<Composer> verify(rollup_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer, tx, cd, "tx rollup", true, build_circuit);
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
