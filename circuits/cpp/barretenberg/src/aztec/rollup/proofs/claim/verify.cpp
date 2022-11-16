#include "./verify.hpp"
#include "claim_circuit.hpp"

namespace rollup {
namespace proofs {
namespace claim {
static std::shared_ptr<waffle::verification_key> verification_key;
static size_t number_of_gates;

namespace {
verify_result<Composer> build_circuit(Composer& composer, claim_tx& tx, circuit_data const&)
{
    verify_result<Composer> result;
    claim_circuit(composer, tx);
    number_of_gates = composer.get_num_gates();
    return result;
}
} // namespace

verify_result<Composer> verify_logic(claim_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    verification_key = composer.circuit_verification_key;
    return verify_logic_internal(composer, tx, cd, "claim", build_circuit);
}

verify_result<Composer> verify(claim_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer, tx, cd, "claim", true, build_circuit);
}

std::shared_ptr<waffle::verification_key> get_verification_key()
{
    return verification_key;
}

size_t get_number_of_gates()
{
    return number_of_gates;
}

} // namespace claim
} // namespace proofs
} // namespace rollup
