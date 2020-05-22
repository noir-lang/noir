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

join_split_circuit_data compute_join_split_circuit_data(std::string const& srs_path, std::string const& key_path)
{
    auto proving_key_path = key_path + "/join_split_proving_key";
    auto verification_key_path = key_path + "/join_split_verification_key";

    if (file_exists(proving_key_path) && file_exists(verification_key_path)) {
        waffle::proving_key_data pk_data;
        waffle::verification_key_data vk_data;

        Timer timer;
        std::cerr << "Loading join-split proving key from: " << proving_key_path << std::endl;
        auto pk_stream = std::ifstream(proving_key_path);
        read(pk_stream, pk_data);

        std::cerr << "Loading join-split verification key from: " << verification_key_path << std::endl;
        auto vk_stream = std::ifstream(verification_key_path);
        read(vk_stream, vk_data);
        std::cerr << "Done: " << timer.toString() << "s" << std::endl;

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
    } else {
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

        std::cerr << "Writing keys..." << std::endl;
        make_dir(key_path);
        auto pk_stream = std::ofstream(proving_key_path);
        auto vk_stream = std::ofstream(verification_key_path);
        write(pk_stream, *proving_key);
        write(vk_stream, *verification_key);
        pk_stream.close();
        vk_stream.close();
        std::cerr << "Done." << std::endl;

        return { proving_key, verification_key, composer.get_num_gates(), proof.proof_data.size() };
    }
}

} // namespace rollup_proofs
} // namespace rollup
