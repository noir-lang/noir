#include "compute_escape_hatch_circuit_data.hpp"
#include "../join_split/compute_join_split_circuit_data.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include "escape_hatch_circuit.hpp"
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace escape_hatch {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

namespace {
bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}
} // namespace

escape_hatch_tx dummy_tx()
{
    auto root_gibberish_path =
        fr_hash_path(ROOT_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));
    auto data_gibberish_path =
        fr_hash_path(DATA_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));
    auto null_gibberish_path =
        fr_hash_path(NULL_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));

    escape_hatch_tx tx;
    tx.js_tx = join_split::noop_tx();

    tx.new_data_root = fr::random_element();
    tx.old_data_path = data_gibberish_path;
    tx.new_data_path = data_gibberish_path;

    tx.old_null_root = fr::random_element();
    tx.new_null_roots = { fr::random_element(), fr::random_element() };
    tx.old_null_paths.resize(2);
    tx.old_null_paths[0] = null_gibberish_path;
    tx.old_null_paths[1] = null_gibberish_path;
    tx.new_null_paths.resize(2);
    tx.new_null_paths[0] = null_gibberish_path;
    tx.new_null_paths[1] = null_gibberish_path;
    tx.account_null_path = null_gibberish_path;

    tx.old_data_roots_root = fr::random_element();
    tx.new_data_roots_root = fr::random_element();
    tx.old_data_roots_path = root_gibberish_path;
    tx.new_data_roots_path = root_gibberish_path;

    return tx;
}

escape_hatch_circuit_data load_escape_hatch_circuit_data(std::string const& srs_path,
                                                         std::string const& escape_hatch_key_path)
{
    waffle::proving_key_data pk_data;
    waffle::verification_key_data vk_data;

    std::cerr << "Loading escape_hatch proving key from: " << escape_hatch_key_path << std::endl;
    auto pk_stream = std::ifstream(escape_hatch_key_path + "/proving_key");
    read_mmap(pk_stream, escape_hatch_key_path, pk_data);

    auto vk_stream = std::ifstream(escape_hatch_key_path + "/verification_key");
    read(vk_stream, vk_data);

    auto crs = std::make_unique<waffle::FileReferenceStringFactory>(srs_path);
    auto proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs->get_prover_crs(pk_data.n));
    auto verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs->get_verifier_crs());

    std::ifstream is(escape_hatch_key_path + "/noop_proof");
    std::vector<uint8_t> proof((std::istreambuf_iterator<char>(is)), std::istreambuf_iterator<char>());

    return { proving_key, verification_key, pk_data.n };
}

void write_escape_hatch_circuit_data(escape_hatch_circuit_data const& data, std::string const& escape_hatch_key_path)
{
    std::cerr << "Writing keys and padding proof..." << std::endl;
    mkdir(escape_hatch_key_path.c_str(), 0700);
    std::ofstream pk_stream(escape_hatch_key_path + "/proving_key");
    std::ofstream vk_stream(escape_hatch_key_path + "/verification_key");
    write_mmap(pk_stream, escape_hatch_key_path, *data.proving_key);
    write(vk_stream, *data.verification_key);
    pk_stream.close();
    vk_stream.close();

    std::cerr << "Done." << std::endl;
}

escape_hatch_circuit_data compute_escape_hatch_circuit_data(std::string const& srs_path)
{
    std::cerr << "Generating escape_hatch circuit keys..." << std::endl;

    escape_hatch_tx tx(dummy_tx());
    Composer composer = Composer(srs_path);
    escape_hatch_circuit(composer, tx);

    std::cerr << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    std::cerr << "Done." << std::endl;

    return { proving_key, verification_key, composer.get_num_gates() };
}

escape_hatch_circuit_data compute_or_load_escape_hatch_circuit_data(std::string const& srs_path,
                                                                    std::string const& key_path)
{
    auto escape_hatch_key_path = key_path + "/escape_hatch";

    if (exists(escape_hatch_key_path)) {
        return load_escape_hatch_circuit_data(srs_path, escape_hatch_key_path);
    } else {
        mkdir(key_path.c_str(), 0700);
        auto data = compute_escape_hatch_circuit_data(srs_path);
        write_escape_hatch_circuit_data(data, escape_hatch_key_path);
        return data;
    }
}

} // namespace escape_hatch
} // namespace proofs
} // namespace rollup
