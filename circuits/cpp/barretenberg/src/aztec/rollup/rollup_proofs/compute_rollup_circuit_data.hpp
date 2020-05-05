#include <plonk/proof_system/verification_key/verification_key.hpp>
#include "../client_proofs/join_split/join_split.hpp"
#include "compute_inner_circuit_data.hpp"
#include "rollup_circuit.hpp"

namespace rollup {
namespace rollup_proofs {

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

struct rollup_circuit_data {
    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates;
    std::shared_ptr<waffle::verification_key> inner_verification_key;
};

rollup_circuit_data compute_rollup_circuit_data(size_t batch_size)
{
    auto inner = compute_inner_circuit_data();

    std::cerr << "Generating rollup circuit keys..." << std::endl;

    Composer composer = Composer("../srs_db/ignition");

    // Junk data required just to create keys.
    std::vector<waffle::plonk_proof> proofs(batch_size, { std::vector<uint8_t>(inner.proof_size) });

    rollup_circuit(composer, batch_size, proofs, inner.verification_key);

    std::cerr << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    auto num_gates = composer.get_num_gates();
    std::cerr << "Done." << std::endl;

    return { proving_key, verification_key, num_gates, inner.verification_key };
}

} // namespace rollup_proofs
} // namespace rollup