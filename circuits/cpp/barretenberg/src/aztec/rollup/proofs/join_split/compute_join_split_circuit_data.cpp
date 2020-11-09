#include "compute_join_split_circuit_data.hpp"
#include "join_split_circuit.hpp"
#include "../notes/native/sign_notes.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace rollup::proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs;
using namespace rollup::proofs::notes;
using namespace plonk::stdlib::merkle_tree;

namespace {
bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}
} // namespace

join_split_tx noop_tx()
{
    grumpkin::fr priv_key = grumpkin::fr::random_element();
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    native::value_note gibberish_note = { pub_key, 0, fr::random_element(), 0 };
    gibberish_note.secret.data[3] = gibberish_note.secret.data[3] & 0x03FFFFFFFFFFFFFFULL;
    gibberish_note.secret = gibberish_note.secret.to_montgomery_form();
    auto gibberish_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));

    join_split_tx tx;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.asset_id = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.old_data_root = fr::random_element();
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { gibberish_note, gibberish_note };
    tx.output_note = { gibberish_note, gibberish_note };
    tx.account_index = 0;
    tx.account_path = gibberish_path;
    tx.signing_pub_key = pub_key;
    tx.account_private_key = priv_key;

    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    tx.signature = native::sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                      tx.output_owner,
                                      { priv_key, pub_key });
    return tx;
}

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

    std::ifstream is(join_split_key_path + "/noop_proof");
    std::vector<uint8_t> proof((std::istreambuf_iterator<char>(is)), std::istreambuf_iterator<char>());

    return { proving_key, verification_key, pk_data.n, proof };
}

void write_join_split_circuit_data(join_split_circuit_data const& data, std::string const& join_split_key_path)
{
    std::cerr << "Writing keys and padding proof..." << std::endl;
    mkdir(join_split_key_path.c_str(), 0700);
    std::ofstream pk_stream(join_split_key_path + "/proving_key");
    std::ofstream vk_stream(join_split_key_path + "/verification_key");
    write_mmap(pk_stream, join_split_key_path, *data.proving_key);
    write(vk_stream, *data.verification_key);
    pk_stream.close();
    vk_stream.close();

    std::ofstream os(join_split_key_path + "/noop_proof");
    os.write((char*)data.padding_proof.data(), (std::streamsize)data.padding_proof.size());

    std::cerr << "Done." << std::endl;
}

join_split_circuit_data compute_join_split_circuit_data(std::string const& srs_path)
{
    std::cerr << "Generating join-split circuit keys..." << std::endl;

    join_split_tx tx(noop_tx());
    Composer composer = Composer(srs_path);
    join_split_circuit(composer, tx);

    std::cerr << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();
    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();
    std::cerr << "Done." << std::endl;

    return { proving_key, verification_key, composer.get_num_gates(), proof.proof_data };
}

join_split_circuit_data compute_or_load_join_split_circuit_data(std::string const& srs_path,
                                                                std::string const& key_path)
{
    auto join_split_key_path = key_path + "/join_split";

    if (exists(join_split_key_path)) {
        return load_join_split_circuit_data(srs_path, join_split_key_path);
    } else {
        mkdir(key_path.c_str(), 0700);
        auto data = compute_join_split_circuit_data(srs_path);
        write_join_split_circuit_data(data, join_split_key_path);
        return data;
    }
}

} // namespace join_split
} // namespace proofs
} // namespace rollup