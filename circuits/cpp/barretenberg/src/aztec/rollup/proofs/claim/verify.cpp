#include "./verify.hpp"
#include "claim_circuit.hpp"

namespace rollup {
namespace proofs {
namespace claim {

namespace {
verify_result<Composer> build_circuit(Composer& composer, claim_tx& tx, circuit_data const&)
{
    verify_result<Composer> result;
    claim_circuit(composer, tx);
    return result;
}
} // namespace

verify_result<Composer> verify_logic(claim_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_logic_internal(composer, tx, cd, "claim", build_circuit);
}

verify_result<Composer> verify(claim_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer, tx, cd, "claim", true, build_circuit);
}

} // namespace claim
} // namespace proofs
} // namespace rollup
