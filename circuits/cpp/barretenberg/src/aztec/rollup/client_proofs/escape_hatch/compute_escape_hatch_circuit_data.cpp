#include "compute_escape_hatch_circuit_data.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>
#include <rollup/client_proofs/escape_hatch/escape_hatch.hpp>
#include <rollup/client_proofs/escape_hatch/sign_notes.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace rollup::client_proofs::escape_hatch;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;
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
    grumpkin::fr priv_key = grumpkin::fr::random_element();
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    tx_note gibberish_note = { pub_key, 0, fr::random_element() };
    gibberish_note.secret.data[3] = gibberish_note.secret.data[3] & 0x03FFFFFFFFFFFFFFULL;
    gibberish_note.secret = gibberish_note.secret.to_montgomery_form();
    auto gibberish_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));
    auto gibberish_nullifier_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));

    escape_hatch_tx tx;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.old_data_root = fr::random_element();
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { gibberish_note, gibberish_note };
    tx.account_index = 0;
    tx.account_path = gibberish_path;
    tx.signing_pub_key = pub_key;
    tx.new_null_roots = { fr::random_element(), fr::random_element() };
    tx.new_nullifier_paths = { gibberish_nullifier_path, gibberish_nullifier_path };
    tx.current_nullifier_paths = { gibberish_nullifier_path, gibberish_nullifier_path };
    tx.account_nullifier_path = gibberish_nullifier_path;

    tx.new_data_root = fr::random_element();
    tx.old_data_roots_root = fr::random_element();
    tx.new_data_roots_root = fr::random_element();

    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1] }, { priv_key, pub_key });

    tx.public_owner = fr::random_element();

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

    return { proving_key, verification_key, pk_data.n, proof };
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

    std::ofstream os(escape_hatch_key_path + "/noop_proof");
    os.write((char*)data.padding_proof.data(), (std::streamsize)data.padding_proof.size());

    std::cerr << "Done." << std::endl;
}

escape_hatch_circuit_data compute_escape_hatch_circuit_data(std::string const& srs_path)
{
    std::cerr << "Generating escape_hatch circuit keys..." << std::endl;

    escape_hatch_tx tx(dummy_tx());
    Composer composer = Composer(srs_path);
    escape_hatch_circuit(composer, tx);
    auto proving_key = composer.compute_proving_key();
    auto verification_key = composer.compute_verification_key();

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    return { proving_key, verification_key, composer.get_num_gates(), proof.proof_data };
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

} // namespace rollup_proofs
} // namespace rollup
