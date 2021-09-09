#include "../proofs/account/compute_circuit_data.hpp"
#include "../proofs/join_split/compute_circuit_data.hpp"
#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/rollup/verify.hpp"
#include "../proofs/root_rollup/compute_circuit_data.hpp"
#include "../proofs/root_rollup/root_rollup_tx.hpp"
#include "../proofs/root_rollup/verify.hpp"
#include "../proofs/claim/get_circuit_data.hpp"
#include "../proofs/claim/verify.hpp"
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

namespace {
std::string data_path;
bool persist;
std::shared_ptr<waffle::DynamicFileReferenceStringFactory> crs;
join_split::circuit_data js_cd;
account::circuit_data account_cd;
tx_rollup::circuit_data tx_rollup_cd;
root_rollup::circuit_data root_rollup_cd;
claim::circuit_data claim_cd;
} // namespace

bool create_tx_rollup()
{
    uint32_t num_txs;
    read(std::cin, num_txs);

    if (!tx_rollup_cd.proving_key || tx_rollup_cd.num_txs != num_txs) {
        tx_rollup_cd.proving_key.reset();
        tx_rollup_cd =
            tx_rollup::get_circuit_data(num_txs, js_cd, account_cd, claim_cd, crs, data_path, true, persist, persist);
    }

    tx_rollup::rollup_tx rollup;
    std::cerr << "Reading tx rollup..." << std::endl;
    read(std::cin, rollup);

    std::cerr << "Received tx rollup with " << rollup.num_txs << " txs." << std::endl;

    if (rollup.num_txs > tx_rollup_cd.num_txs) {
        std::cerr << "Receieved rollup size too large: " << rollup.txs.size() << std::endl;
        return false;
    }

    Timer timer;
    tx_rollup_cd.proving_key->reset();

    std::cerr << "Creating tx rollup proof..." << std::endl;
    auto result = verify(rollup, tx_rollup_cd);

    std::cerr << "Time taken: " << timer.toString() << std::endl;
    std::cerr << "Verified: " << result.verified << std::endl;

    write(std::cout, result.proof_data);
    write(std::cout, (uint8_t)result.verified);
    std::cout << std::flush;

    return result.verified;
}

bool create_root_rollup()
{
    uint32_t num_txs;
    uint32_t num_proofs;
    read(std::cin, num_txs);
    read(std::cin, num_proofs);

    if (!tx_rollup_cd.proving_key || tx_rollup_cd.num_txs != num_txs) {
        tx_rollup_cd.proving_key.reset();
        tx_rollup_cd =
            tx_rollup::get_circuit_data(num_txs, js_cd, account_cd, claim_cd, crs, data_path, true, persist, persist);
    }

    if (!root_rollup_cd.proving_key || root_rollup_cd.num_inner_rollups != num_proofs) {
        root_rollup_cd.proving_key.reset();
        root_rollup_cd =
            root_rollup::get_circuit_data(num_proofs, tx_rollup_cd, crs, data_path, true, persist, persist);
    }

    root_rollup::root_rollup_tx root_rollup;
    std::cerr << "Reading root rollup..." << std::endl;
    read(std::cin, root_rollup);

    std::cerr << "Received root rollup with " << root_rollup.rollups.size() << " rollups." << std::endl;

    if (root_rollup.rollups.size() > root_rollup_cd.num_inner_rollups) {
        std::cerr << "Receieved rollup size too large: " << root_rollup.rollups.size() << std::endl;
        return false;
    }

    auto gibberish_data_roots_path = fr_hash_path(28, std::make_pair(fr::random_element(), fr::random_element()));

    Timer timer;
    root_rollup_cd.proving_key->reset();

    std::cerr << "Creating root rollup proof..." << std::endl;
    auto result = verify(root_rollup, root_rollup_cd);

    std::cerr << "Time taken: " << timer.toString() << std::endl;
    std::cerr << "Verified: " << result.verified << std::endl;

    auto encoded_inputs = result.root_data.encode_proof_data();

    write(std::cout, encoded_inputs);
    write(std::cout, result.proof_data);
    write(std::cout, (uint8_t)result.verified);
    std::cout << std::flush;

    return result.verified;
}

bool create_claim()
{
    claim::claim_tx claim_tx;
    std::cerr << "Reading claim tx..." << std::endl;
    read(std::cin, claim_tx);

    Timer timer;
    claim_cd.proving_key->reset();

    std::cerr << "Creating claim proof..." << std::endl;
    auto result = verify(claim_tx, claim_cd);

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

    const std::string srs_path = (args.size() > 1) ? args[1] : "../srs_db/ignition";
    data_path = (args.size() > 2) ? args[2] : "./data";
    persist = data_path != "-";

    std::cerr << "Loading crs..." << std::endl;
    crs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(srs_path);

    account_cd = persist ? account::compute_or_load_circuit_data(crs, data_path) : account::compute_circuit_data(crs);
    js_cd = persist ? join_split::compute_or_load_circuit_data(crs, data_path) : join_split::compute_circuit_data(crs);
    claim_cd = claim::get_circuit_data(crs, data_path, true, persist, persist);

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
        }
    }

    return 0;
}
