#include "compute_circuit_data.hpp"
#include "account.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include <plonk/reference_string/reference_string.hpp>
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

circuit_data load_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                               std::string const& account_key_path)
{
    waffle::proving_key_data pk_data;
    waffle::verification_key_data vk_data;

    std::cerr << "Loading account proving key from: " << account_key_path << std::endl;
    auto pk_stream = std::ifstream(account_key_path + "/proving_key");
    read_mmap(pk_stream, account_key_path, pk_data);

    auto vk_stream = std::ifstream(account_key_path + "/verification_key");
    read(vk_stream, vk_data);

    auto proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), srs->get_prover_crs(pk_data.n));
    auto verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), srs->get_verifier_crs());

    return { proving_key, verification_key, pk_data.n };
}

void write_circuit_data(circuit_data const& data, std::string const& account_key_path)
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

circuit_data compute_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs)
{
    std::cerr << "Generating account circuit keys..." << std::endl;

    account_tx tx;
    tx.account_path.resize(32);

    Composer composer = Composer(srs);
    account_circuit(composer, tx);

    std::cerr << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    std::cerr << "Done." << std::endl;

    return { proving_key, verification_key, composer.get_num_gates() };
}

circuit_data compute_or_load_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                                          std::string const& key_path)
{
    auto account_key_path = key_path + "/account";

    if (exists(account_key_path)) {
        return load_circuit_data(srs, account_key_path);
    } else {
        mkdir(key_path.c_str(), 0700);
        auto data = compute_circuit_data(srs);
        write_circuit_data(data, account_key_path);
        return data;
    }
}

} // namespace account
} // namespace proofs
} // namespace rollup
