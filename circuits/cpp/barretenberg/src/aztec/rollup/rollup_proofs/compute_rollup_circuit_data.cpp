#include "compute_rollup_circuit_data.hpp"
#include "../client_proofs/join_split/join_split.hpp"
#include "rollup_circuit.hpp"

namespace rollup {
namespace rollup_proofs {

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

rollup_circuit_data compute_rollup_circuit_data(size_t rollup_size,
                                                join_split_circuit_data const& inner,
                                                bool create_keys,
                                                std::string const& srs_path)
{
    std::cerr << "Generating rollup circuit... (size: " << rollup_size << ")" << std::endl;
    Composer composer = Composer(srs_path);

    // Junk data required just to create keys.
    auto gibberish_data_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));
    auto gibberish_null_path = fr_hash_path(128, std::make_pair(fr::random_element(), fr::random_element()));
    auto gibberish_roots_path = fr_hash_path(28, std::make_pair(fr::random_element(), fr::random_element()));

    rollup_tx rollup = {
        0,
        (uint32_t)rollup_size,
        0,
        std::vector(rollup_size, std::vector<uint8_t>(inner.proof_size, 1)),
        fr::random_element(),
        fr::random_element(),
        fr::random_element(),
        gibberish_data_path,
        gibberish_data_path,
        fr::random_element(),
        std::vector(rollup_size * 2, fr::random_element()),
        std::vector(rollup_size * 2, gibberish_null_path),
        std::vector(rollup_size * 2, gibberish_null_path),
        fr::random_element(),
        std::vector(rollup_size * 2, gibberish_roots_path),
        std::vector(rollup_size * 2, uint32_t(0)),
    };

    rollup_circuit(composer, rollup, inner.verification_key, rollup_size, false);
    std::cerr << "Rollup circuit gates: " << composer.get_num_gates() << std::endl;

    std::shared_ptr<waffle::proving_key> proving_key;
    std::shared_ptr<waffle::verification_key> verification_key;
    size_t num_gates = 0;

    if (create_keys) {
        std::cerr << "Creating keys..." << std::endl;
        proving_key = composer.compute_proving_key();
        verification_key = composer.compute_verification_key();
        num_gates = composer.get_num_gates();
        std::cerr << "Done." << std::endl;
    }

    // auto proving_key = std::shared_ptr<waffle::proving_key>();
    // auto verification_key = std::shared_ptr<waffle::verification_key>();
    // size_t num_gates = 0;

    return { proving_key, verification_key, rollup_size, num_gates, inner.proof_size, inner.verification_key };
}

} // namespace rollup_proofs
} // namespace rollup
