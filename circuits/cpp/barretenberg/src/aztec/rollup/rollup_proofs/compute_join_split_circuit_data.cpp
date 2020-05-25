#include "compute_join_split_circuit_data.hpp"
#include "../client_proofs/join_split/join_split.hpp"
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>

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

join_split_circuit_data load_join_split_circuit_data(std::string const& srs_path,
                                                     std::string const& join_split_key_path)
{
    waffle::proving_key_data pk_data;
    waffle::verification_key_data vk_data;

    std::cerr << "Loading join-split proving key from: " << join_split_key_path << std::endl;
    auto pk_stream = std::ifstream(join_split_key_path + "/proving_key");
    read_mmap(pk_stream, join_split_key_path, pk_data);

    auto vk_stream = std::ifstream(join_split_key_path + "/verification_key");
    read(vk_stream, vk_data);

    auto crs = std::make_unique<waffle::FileReferenceStringFactory>(srs_path);
    auto proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.n));
    auto verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs->get_verifier_crs());

    // Have to create a proof to get it's byte length :(
    join_split_tx tx;
    tx.input_path[0].resize(32);
    tx.input_path[1].resize(32);
    Composer composer = Composer(proving_key, verification_key, pk_data.n);
    join_split_circuit(composer, tx);
    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return { proving_key, verification_key, pk_data.n, proof.proof_data.size() };
}

void write_join_split_circuit_data(join_split_circuit_data const& data, std::string const& join_split_key_path)
{
    std::cerr << "Writing keys..." << std::endl;
    mkdir(join_split_key_path.c_str(), 0700);
    auto pk_stream = std::ofstream(join_split_key_path + "/proving_key");
    auto vk_stream = std::ofstream(join_split_key_path + "/verification_key");
    write_mmap(pk_stream, join_split_key_path, *data.proving_key);
    write(vk_stream, *data.verification_key);
    pk_stream.close();
    vk_stream.close();
    std::cerr << "Done." << std::endl;
}

join_split_circuit_data compute_join_split_circuit_data(std::string const& srs_path)
{
    std::cerr << "Generating join-split circuit keys..." << std::endl;

    // Junk data required just to create keys.
    join_split_tx tx;
    tx.input_path[0].resize(32);
    tx.input_path[1].resize(32);

    Composer composer = Composer(srs_path);
    join_split_circuit(composer, tx);

    std::cerr << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    // TODO: Avoid by computing proof size independently?
    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();
    std::cerr << "Done." << std::endl;

    return { proving_key, verification_key, composer.get_num_gates(), proof.proof_data.size() };
}

join_split_circuit_data compute_or_load_join_split_circuit_data(std::string const& srs_path,
                                                                std::string const& key_path)
{
    auto join_split_key_path = key_path + "/join_split_keys";

    if (exists(join_split_key_path)) {
        return load_join_split_circuit_data(srs_path, join_split_key_path);
    } else {
        mkdir(key_path.c_str(), 0700);
        auto data = compute_join_split_circuit_data(srs_path);
        write_join_split_circuit_data(data, join_split_key_path);
        return data;
    }
}

} // namespace rollup_proofs
} // namespace rollup
