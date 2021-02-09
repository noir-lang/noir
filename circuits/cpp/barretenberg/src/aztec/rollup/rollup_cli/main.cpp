#include "../proofs/account/compute_circuit_data.hpp"
#include "../proofs/join_split/compute_circuit_data.hpp"
#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/rollup/verify.hpp"
#include "../proofs/root_rollup/compute_circuit_data.hpp"
#include "../proofs/root_rollup/root_rollup_tx.hpp"
#include "../proofs/root_rollup/verify.hpp"
#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <stdlib/types/turbo.hpp>

using namespace ::rollup::proofs;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace serialize;
namespace tx_rollup = ::rollup::proofs::rollup;

bool create_tx_rollup(tx_rollup::circuit_data const& circuit_data)
{
    auto rollup_size = circuit_data.rollup_size;
    tx_rollup::rollup_tx rollup;

    std::cerr << "Reading tx rollup..." << std::endl;
    read(std::cin, rollup);

    std::cerr << "Received tx rollup with " << rollup.num_txs << " txs." << std::endl;

    if (rollup.num_txs > rollup_size) {
        std::cerr << "Receieved rollup size too large: " << rollup.txs.size() << std::endl;
        return false;
    }

    Timer timer;
    circuit_data.proving_key->reset();

    std::cerr << "Creating tx rollup proof..." << std::endl;
    auto result = verify_rollup(rollup, circuit_data);

    std::cerr << "Time taken: " << timer.toString() << std::endl;
    std::cerr << "Verified: " << result.verified << std::endl;

    write(std::cout, result.proof_data);
    write(std::cout, (uint8_t)result.verified);
    std::cout << std::flush;

    return result.verified;
}

bool create_root_rollup(root_rollup::circuit_data const& circuit_data)
{
    root_rollup::root_rollup_tx root_rollup;

    std::cerr << "Reading root rollup..." << std::endl;
    read(std::cin, root_rollup);

    std::cerr << "Received root rollup with " << root_rollup.rollups.size() << " rollups." << std::endl;

    if (root_rollup.rollups.size() > circuit_data.num_inner_rollups) {
        std::cerr << "Receieved rollup size too large: " << root_rollup.rollups.size() << std::endl;
        return false;
    }

    auto gibberish_data_roots_path = fr_hash_path(28, std::make_pair(fr::random_element(), fr::random_element()));

    Timer timer;
    circuit_data.proving_key->reset();

    std::cerr << "Creating root rollup proof..." << std::endl;
    auto result = verify(root_rollup, circuit_data);

    std::cerr << "Time taken: " << timer.toString() << std::endl;
    std::cerr << "Verified: " << result.verified << std::endl;

    write(std::cout, result.proof_data);
    write(std::cout, (uint8_t)result.verified);
    std::cout << std::flush;

    return result.verified;
}

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    size_t tx_rollup_size = (args.size() > 1) ? (size_t)atoi(args[1].c_str()) : 1;
    size_t root_rollup_size = (args.size() > 2) ? (size_t)atoi(args[2].c_str()) : 1;
    const std::string srs_path = (args.size() > 3) ? args[3] : "../srs_db/ignition";
    const std::string data_path = (args.size() > 4) ? args[4] : "./data";
    bool reduce_mem = (args.size() > 5) ? (bool)atoi(args[5].c_str()) : false;
    auto persist = data_path != "-";
    bool persist_rollup_keys = persist && ((args.size() > 6) ? (bool)atoi(args[6].c_str()) : true);

    std::cerr << "Loading crs..." << std::endl;
    auto crs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(srs_path);

    auto account_circuit_data =
        persist ? account::compute_or_load_circuit_data(crs, data_path) : account::compute_circuit_data(crs);
    auto join_split_circuit_data =
        persist ? join_split::compute_or_load_circuit_data(crs, data_path) : join_split::compute_circuit_data(crs);

    tx_rollup::circuit_data tx_rollup_circuit_data;
    root_rollup::circuit_data root_rollup_circuit_data;
    if (reduce_mem) {
        // Hacky, we have to immediately release the key because if we need to generate both keys it uses too much mem.
        // This way we create both keys on startup, but free the memory, then reload them on demand...
        tx_rollup_circuit_data = tx_rollup::get_circuit_data(tx_rollup_size,
                                                             join_split_circuit_data,
                                                             account_circuit_data,
                                                             crs,
                                                             data_path,
                                                             true,
                                                             persist_rollup_keys,
                                                             persist_rollup_keys);
        tx_rollup_circuit_data.proving_key.reset();

        root_rollup_circuit_data = root_rollup::get_circuit_data(
            root_rollup_size, tx_rollup_circuit_data, crs, data_path, true, persist_rollup_keys, persist_rollup_keys);
        root_rollup_circuit_data.proving_key.reset();

        // We expect a tx rollup first, so reload it...
        tx_rollup_circuit_data = tx_rollup::get_circuit_data(
            tx_rollup_size, join_split_circuit_data, account_circuit_data, crs, data_path, true, true, true);
    } else {
        // Assume we are not making massive circuits.
        tx_rollup_circuit_data = tx_rollup::get_circuit_data(tx_rollup_size,
                                                             join_split_circuit_data,
                                                             account_circuit_data,
                                                             crs,
                                                             data_path,
                                                             true,
                                                             persist_rollup_keys,
                                                             persist_rollup_keys);
        root_rollup_circuit_data = root_rollup::get_circuit_data(
            root_rollup_size, tx_rollup_circuit_data, crs, data_path, true, persist_rollup_keys, persist_rollup_keys);
    }

    std::cerr << "Reading rollups from standard input..." << std::endl;
    serialize::write(std::cout, true);

    while (true) {
        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        uint32_t proof_id;
        read(std::cin, proof_id);

        switch (proof_id) {
        case 0: {
            if (reduce_mem && !tx_rollup_circuit_data.proving_key) {
                root_rollup_circuit_data.proving_key.reset();
                tx_rollup_circuit_data = tx_rollup::get_circuit_data(
                    tx_rollup_size, join_split_circuit_data, account_circuit_data, crs, data_path, true, false, true);
            }
            create_tx_rollup(tx_rollup_circuit_data);
            break;
        }
        case 1: {
            if (reduce_mem && !root_rollup_circuit_data.proving_key) {
                tx_rollup_circuit_data.proving_key.reset();
                root_rollup_circuit_data = root_rollup::get_circuit_data(
                    root_rollup_size, tx_rollup_circuit_data, crs, data_path, true, false, true);
            }
            create_root_rollup(root_rollup_circuit_data);
            break;
        }
        }
    }

    return 0;
}
