#include <sstream>
#include <iostream>

#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>

#include "../proofs/account/compute_circuit_data.hpp"
#include "../proofs/account/verify.hpp"
#include "../proofs/join_split/compute_circuit_data.hpp"
#include "../proofs/claim/get_circuit_data.hpp"
#include "../proofs/claim/verify.hpp"
#include "../proofs/rollup/index.hpp"
#include "../proofs/root_rollup/index.hpp"
#include "../proofs/root_verifier/index.hpp"
#include <common/timer.hpp>
#include <common/container.hpp>
#include <common/map.hpp>

#include <proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

using namespace ::rollup::proofs;
using namespace plonk::stdlib::merkle_tree;
using namespace serialize;
namespace tx_rollup = ::rollup::proofs::rollup;

namespace {
// Number of transactions in an inner rollup.
size_t txs_per_inner;
// Number of inner rollups in a root rollup.
size_t inners_per_root;
// In mock mode, mock proofs (expected public inputs, but no constraints) are generated.
bool mock_proofs;
// Create big circuits proving keys lazily to improve startup times.
bool lazy_init;
// True if rollup circuit data (proving and verification keys) are to be persisted to disk.
// We likely don't have enough memory to hold all keys in memory, and loading keys from disk is faster.
bool persist;
// Path to save proving keys to if persist is on.
std::string data_path;

std::shared_ptr<waffle::DynamicFileReferenceStringFactory> crs;
join_split::circuit_data js_cd;
account::circuit_data account_cd;
claim::circuit_data claim_cd;
tx_rollup::circuit_data tx_rollup_cd;
root_rollup::circuit_data root_rollup_cd;
root_verifier::circuit_data root_verifier_cd;
} // namespace

// Postcondition: tx_rollup_cd has a proving key and verification key.
void init_tx_rollup(size_t num_txs)
{
    if (tx_rollup_cd.proving_key) {
        // We always have a vk if we have a pk, as we request both in the call to get_circuit_data.
        return;
    }
    if (lazy_init) {
        // In lazy init mode we conserve memory. Throw away the root rollup proving key first.
        info("Purging root rollup proving key.");
        root_rollup_cd.proving_key.reset();
    }
    tx_rollup_cd = tx_rollup::get_circuit_data(
        num_txs, js_cd, account_cd, claim_cd, crs, data_path, true, persist, persist, true, true, mock_proofs);
}

bool create_tx_rollup()
{
    init_tx_rollup(txs_per_inner);

    tx_rollup::rollup_tx rollup;
    std::cerr << "Reading tx rollup..." << std::endl;
    read(std::cin, rollup);
    std::cerr << "Received tx rollup with " << rollup.num_txs << " txs." << std::endl;

    auto result = verify(rollup, tx_rollup_cd);

    write(std::cout, result.proof_data);
    write(std::cout, result.verified);
    std::cout << std::flush;

    return result.verified;
}

// Postcondition: root_rollup_cd has a proving key and verification key.
void init_root_rollup(size_t num_rollups)
{
    if (root_rollup_cd.proving_key) {
        // We always have a vk if we have a pk, as we request both in the call to get_circuit_data.
        return;
    }
    if (!tx_rollup_cd.verification_key) {
        // If we've never created the tx rollup circuit data, we won't have a vk. Build it.
        init_tx_rollup(txs_per_inner);
    }
    if (lazy_init) {
        // In lazy init mode we conserve memory. Throw away the tx rollup proving key first.
        info("Purging tx rollup proving key.");
        tx_rollup_cd.proving_key.reset();
    }
    root_rollup_cd = root_rollup::get_circuit_data(
        num_rollups, tx_rollup_cd, crs, data_path, true, persist, persist, true, true, mock_proofs);
}

bool create_root_rollup()
{
    init_root_rollup(inners_per_root);

    root_rollup::root_rollup_tx root_rollup;
    std::cerr << "Reading root rollup..." << std::endl;
    read(std::cin, root_rollup);
    std::cerr << "Received root rollup with " << root_rollup.rollups.size() << " rollups." << std::endl;

    auto result = verify(root_rollup, root_rollup_cd);

    root_rollup::root_rollup_broadcast_data broadcast_data(result.broadcast_data);
    auto buf = join({ to_buffer(broadcast_data), result.proof_data });

    write(std::cout, buf);
    write(std::cout, result.verified);
    std::cout << std::flush;

    return result.verified;
}

bool create_claim()
{
    claim::claim_tx claim_tx;
    std::cerr << "Reading claim tx..." << std::endl;
    read(std::cin, claim_tx);

    auto result = verify(claim_tx, claim_cd);

    write(std::cout, result.proof_data);
    write(std::cout, result.verified);
    std::cout << std::flush;

    return result.verified;
}

// Postcondition: root_verifier_cd has a proving key and verification key.
void init_root_verifier()
{
    if (root_verifier_cd.proving_key) {
        // We always have a vk if we have a pk, as we request both in the call to get_circuit_data.
        return;
    }
    if (!root_rollup_cd.verification_key) {
        // If we've never created the root rollup circuit data, we won't have a vk. Build it.
        init_root_rollup(txs_per_inner);
    }
    root_verifier_cd = root_verifier::get_circuit_data(root_rollup_cd,
                                                       crs,
                                                       { root_rollup_cd.verification_key },
                                                       data_path,
                                                       true,
                                                       persist,
                                                       persist,
                                                       true,
                                                       true,
                                                       mock_proofs);
}

bool create_root_verifier()
{
    init_root_verifier();

    std::vector<uint8_t> root_rollup_proof_buf;
    std::cerr << "Reading root verifier tx..." << std::endl;
    read(std::cin, root_rollup_proof_buf);

    auto rollup_size = inners_per_root * tx_rollup_cd.rollup_size;
    auto tx = root_verifier::create_root_verifier_tx(root_rollup_proof_buf, rollup_size);

    auto result = verify(tx, root_verifier_cd, root_rollup_cd);

    result.proof_data = join({ tx.broadcast_data, result.proof_data });
    write(std::cout, result.proof_data);
    write(std::cout, (uint8_t)result.verified);
    std::cout << std::flush;

    return result.verified;
}

bool create_account_proof()
{
    account::account_tx account_tx;
    std::cerr << "Reading account tx..." << std::endl;
    read(std::cin, account_tx);

    auto result = verify(account_tx, account_cd);

    write(std::cout, result.proof_data);
    write(std::cout, result.verified);
    std::cout << std::flush;

    return result.verified;
}

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    info("Rollup CLI pid: ", getpid());
    info("Command line: ", join(args, " "));

    const std::string srs_path = (args.size() > 1) ? args[1] : "../srs_db/ignition";
    txs_per_inner = args.size() > 2 ? (std::stoul(args[2])) : 1;
    inners_per_root = args.size() > 3 ? (std::stoul(args[3])) : 1;
    mock_proofs = args.size() > 4 ? args[4] == "true" : false;
    lazy_init = args.size() > 5 ? args[5] == "true" : false;
    persist = args.size() > 6 ? args[6] == "true" : true;
    data_path = (args.size() > 7) ? args[7] : "./data";

    info("Txs per inner: ", txs_per_inner);
    info("Inners per root: ", inners_per_root);
    info("Mock proofs: ", mock_proofs);
    info("Lazy init: ", lazy_init);
    info("Persist: ", persist);
    info("Data path: ", data_path);

    if (mock_proofs) {
        info("Running in mock proof mode. Mock proofs will be generated!");
    }

    info("Loading crs...");
    crs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(srs_path);

    account_cd = account::get_circuit_data(crs, mock_proofs);
    js_cd = join_split::get_circuit_data(crs, mock_proofs);
    claim_cd = claim::get_circuit_data(crs, mock_proofs);

    // Lazy init mode conserves memory by purging and recomputing tx/root proving keys.
    // If the halloumi instance is targeted to produce a specific type of proof, use lazy init as it will only
    // need to hold the pk of the specific proof it creates in memory.
    //
    // Eager mode can be useful to create all the circuits up front at load time, which is fine if they are not
    // too big. It can be useful for determining to total memory footprint of the process for certain circuit sizes.
    if (!lazy_init) {
        info("Running in eager init mode, all proving keys will be created once up front.");
        init_tx_rollup(txs_per_inner);
        init_root_rollup(inners_per_root);
        init_root_verifier();
    } else {
        info("Running in lazy init mode, tx rollup and root rollup proving keys will be swapped in and out.");
    }

    info("Reading rollups from standard input...");
    while (true) {
        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        uint32_t proof_id;
        read(std::cin, proof_id);

        switch (proof_id) {
        case 0: {
            create_tx_rollup();
            break;
        }
        case 1: {
            create_root_rollup();
            break;
        }
        case 2: {
            create_claim();
            break;
        }
        case 3: {
            create_root_verifier();
            break;
        }
        case 4: {
            std::cerr << "Serving request to create account proof..." << std::endl;
            create_account_proof();
            break;
        }
        case 100: {
            // Convert to buffer first, so when we call write we prefix the buffer length.
            std::cerr << "Serving join split vk..." << std::endl;
            write(std::cout, to_buffer(*js_cd.verification_key));
            break;
        }
        case 101: {
            std::cerr << "Serving account vk..." << std::endl;
            write(std::cout, to_buffer(*account_cd.verification_key));
            break;
        }
        case 666: {
            // Ping... Pong... Used for learning when rollup_cli is responsive.
            std::cerr << "Ping... Pong..." << std::endl;
            serialize::write(std::cout, true);
            break;
        }
        default: {
            std::cerr << "Unknown command: " << proof_id << std::endl;
            break;
        }
        }
    }

    return 0;
}
