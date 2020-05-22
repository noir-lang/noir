#include "compute_rollup_circuit_data.hpp"
#include "../client_proofs/join_split/join_split.hpp"
#include "rollup_circuit.hpp"
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

namespace {
bool file_exists(std::string const& filename)
{
    std::ifstream infile(filename);
    return infile.good();
}

void make_dir(std::string const& path)
{
    struct stat st;
    if (stat(path.c_str(), &st) == -1) {
        mkdir(path.c_str(), 0700);
    }
}
} // namespace

rollup_circuit_data compute_rollup_circuit_data(size_t rollup_size,
                                                join_split_circuit_data const& inner,
                                                bool create_keys,
                                                std::string const& srs_path,
                                                std::string const& key_path)
{
    if (!create_keys) {
        std::shared_ptr<waffle::proving_key> proving_key;
        std::shared_ptr<waffle::verification_key> verification_key;
        return { proving_key, verification_key, rollup_size, 0, inner.proof_size, inner.verification_key };
    }

    auto proving_key_path = key_path + "/rolllup_proving_key_" + std::to_string(rollup_size);
    auto verification_key_path = key_path + "/rolllup_verification_key_" + std::to_string(rollup_size);

    if (file_exists(proving_key_path) && file_exists(verification_key_path)) {
        waffle::proving_key_data pk_data;
        waffle::verification_key_data vk_data;

        Timer timer;
        std::cerr << "Loading rollup proving key from: " << proving_key_path << std::endl;
        auto pk_stream = std::ifstream(proving_key_path);
        // TODO: Use is_base_of?
        read(static_cast<std::istream&>(pk_stream), pk_data);

        std::cerr << "Loading rollup verification key from: " << verification_key_path << std::endl;
        auto vk_stream = std::ifstream(verification_key_path);
        read(vk_stream, vk_data);
        std::cerr << "Done: " << timer.toString() << "s" << std::endl;

        auto crs = std::make_unique<waffle::FileReferenceStringFactory>(srs_path);
        auto proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.n));
        auto verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs->get_verifier_crs());

        return { proving_key, verification_key, rollup_size, pk_data.n, inner.proof_size, inner.verification_key };
    } else {
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

        std::cerr << "Creating keys..." << std::endl;
        auto proving_key = composer.compute_proving_key();
        auto verification_key = composer.compute_verification_key();
        size_t num_gates = composer.get_num_gates();
        std::cerr << "Done." << std::endl;

        std::cerr << "Writing keys..." << std::endl;
        make_dir(key_path);
        auto pk_stream = std::ofstream(proving_key_path);
        auto vk_stream = std::ofstream(verification_key_path);
        write(pk_stream, *proving_key);
        write(vk_stream, *verification_key);
        pk_stream.close();
        vk_stream.close();
        std::cerr << "Done." << std::endl;

        return { proving_key, verification_key, rollup_size, num_gates, inner.proof_size, inner.verification_key };
    }
}

} // namespace rollup_proofs
} // namespace rollup
