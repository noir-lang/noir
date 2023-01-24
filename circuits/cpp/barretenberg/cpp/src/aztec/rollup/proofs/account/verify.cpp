#include "./verify.hpp"
#include "./account.hpp"
#include "./account_tx.hpp"

namespace rollup {
namespace proofs {
namespace account {

namespace {
verify_result<Composer> build_circuit(Composer& composer, account_tx& tx, circuit_data const&)
{
    verify_result<Composer> result;
    account_circuit(composer, tx);
    return result;
}
} // namespace

verify_result<Composer> verify_logic(account_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_logic_internal(composer, tx, cd, "account", build_circuit);
}

verify_result<Composer> verify(account_tx& tx, circuit_data const& cd)
{
    Composer composer = Composer(cd.proving_key, cd.verification_key, cd.num_gates);
    return verify_internal(composer, tx, cd, "account", true, build_circuit);
}

} // namespace account
} // namespace proofs
} // namespace rollup
