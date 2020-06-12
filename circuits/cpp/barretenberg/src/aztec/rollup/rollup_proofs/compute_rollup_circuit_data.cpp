#include "compute_rollup_circuit_data.hpp"
#include <rollup/client_proofs/join_split/join_split.hpp>
#include "rollup_circuit.hpp"
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

namespace {
bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}
} // namespace

rollup_circuit_data load_rollup_circuit_data(size_t rollup_size,
                                             join_split_circuit_data const& inner,
                                             std::string const& srs_path,
                                             std::string const& rollup_key_path)
{
    waffle::proving_key_data pk_data;
    waffle::verification_key_data vk_data;

    std::cerr << "Loading keys from: " << rollup_key_path << std::endl;
    auto pk_stream = std::ifstream(rollup_key_path + "/proving_key");
    read_mmap(pk_stream, rollup_key_path, pk_data);

    auto vk_stream = std::ifstream(rollup_key_path + "/verification_key");
    read(vk_stream, vk_data);

    auto crs = std::make_unique<waffle::FileReferenceStringFactory>(srs_path);
    auto proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.n));
    auto verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs->get_verifier_crs());

    return {
        proving_key, verification_key, rollup_size, pk_data.n, inner.padding_proof.size(), inner.verification_key
    };
}

void write_rollup_circuit_data(rollup_circuit_data const& data, std::string const& rollup_key_path)
{
    std::cerr << "Writing keys..." << std::endl;
    mkdir(rollup_key_path.c_str(), 0700);
    Timer write_timer;
    std::ofstream pk_stream(rollup_key_path + "/proving_key");
    std::ofstream vk_stream(rollup_key_path + "/verification_key");
    write_mmap(pk_stream, rollup_key_path, *data.proving_key);
    write(vk_stream, *data.verification_key);
    pk_stream.close();
    vk_stream.close();
    std::cerr << "Done: " << write_timer.toString() << "s" << std::endl;
}

rollup_circuit_data compute_rollup_circuit_data(size_t rollup_size,
                                                join_split_circuit_data const& inner,
                                                bool create_keys,
                                                std::string const& srs_path)
{
    if (!create_keys) {
        std::shared_ptr<waffle::proving_key> proving_key;
        std::shared_ptr<waffle::verification_key> verification_key;
        return { proving_key, verification_key, rollup_size, 0, inner.padding_proof.size(), inner.verification_key };
    }

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
        std::vector(rollup_size, std::vector<uint8_t>(inner.padding_proof.size(), 1)),
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
        fr::random_element(),
        gibberish_roots_path,
        gibberish_roots_path,
        std::vector(rollup_size * 2, gibberish_roots_path),
        std::vector(rollup_size * 2, uint32_t(0)),
    };

    rollup_circuit(composer, rollup, inner.verification_key, rollup_size, false);
    std::cerr << "Rollup circuit gates: " << composer.get_num_gates() << std::endl;

    std::cerr << "Creating keys..." << std::endl;
    Timer timer;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    size_t num_gates = composer.get_num_gates();
    std::cerr << "Done: " << timer.toString() << "s" << std::endl;

    return {
        proving_key, verification_key, rollup_size, num_gates, inner.padding_proof.size(), inner.verification_key
    };
}

rollup_circuit_data compute_or_load_rollup_circuit_data(size_t rollup_size,
                                                        join_split_circuit_data const& inner,
                                                        std::string const& srs_path,
                                                        std::string const& key_path)
{
    auto rollup_key_path = key_path + "/rollup_" + std::to_string(rollup_size);

    if (exists(rollup_key_path)) {
        return load_rollup_circuit_data(rollup_size, inner, srs_path, rollup_key_path);
    } else {
        mkdir(key_path.c_str(), 0700);
        auto data = compute_rollup_circuit_data(rollup_size, inner, true, srs_path);
        write_rollup_circuit_data(data, rollup_key_path);
        return data;
    }
}

} // namespace rollup_proofs
} // namespace rollup
