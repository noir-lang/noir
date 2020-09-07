#include "compute_account_circuit_data.hpp"
#include "account.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>

namespace rollup {
namespace proofs {
namespace account {

using namespace rollup::proofs::account;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs;
using namespace plonk::stdlib::merkle_tree;

namespace {
bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}
} // namespace

account_circuit_data load_account_circuit_data(std::string const& srs_path, std::string const& account_key_path)
{
    waffle::proving_key_data pk_data;
    waffle::verification_key_data vk_data;

    std::cerr << "Loading join-split proving key from: " << account_key_path << std::endl;
    auto pk_stream = std::ifstream(account_key_path + "/proving_key");
    read_mmap(pk_stream, account_key_path, pk_data);

    auto vk_stream = std::ifstream(account_key_path + "/verification_key");
    read(vk_stream, vk_data);

    auto crs = std::make_unique<waffle::FileReferenceStringFactory>(srs_path);
    auto proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.n));
    auto verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs->get_verifier_crs());

    return { proving_key, verification_key, pk_data.n };
}

void write_account_circuit_data(account_circuit_data const& data, std::string const& account_key_path)
{
    std::cerr << "Writing keys..." << std::endl;
    mkdir(account_key_path.c_str(), 0700);
    std::ofstream pk_stream(account_key_path + "/proving_key");
    std::ofstream vk_stream(account_key_path + "/verification_key");
    write_mmap(pk_stream, account_key_path, *data.proving_key);
    write(vk_stream, *data.verification_key);
    pk_stream.close();
    vk_stream.close();

    std::cerr << "Done." << std::endl;
}

account_circuit_data compute_account_circuit_data(std::string const& srs_path)
{
    std::cerr << "Generating account circuit keys..." << std::endl;

    account_tx tx;
    tx.account_path.resize(32);

    Composer composer = Composer(srs_path);
    account_circuit(composer, tx);

    std::cerr << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();
    std::cerr << "Done." << std::endl;

    return { proving_key, verification_key, composer.get_num_gates() };
}

account_circuit_data compute_or_load_account_circuit_data(std::string const& srs_path, std::string const& key_path)
{
    auto account_key_path = key_path + "/account";

    if (exists(account_key_path)) {
        return load_account_circuit_data(srs_path, account_key_path);
    } else {
        mkdir(key_path.c_str(), 0700);
        auto data = compute_account_circuit_data(srs_path);
        write_account_circuit_data(data, account_key_path);
        return data;
    }
}

} // namespace account
} // namespace proofs
} // namespace rollup
