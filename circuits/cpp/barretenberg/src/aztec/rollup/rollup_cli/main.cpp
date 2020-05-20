#include "../rollup_proofs/compute_join_split_circuit_data.hpp"
#include "../rollup_proofs/compute_rollup_circuit_data.hpp"
#include "../rollup_proofs/create_noop_join_split_proof.hpp"
#include "../rollup_proofs/rollup_tx.hpp"
#include "../rollup_proofs/verify_rollup.hpp"
#include <common/timer.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <stdlib/types/turbo.hpp>

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    size_t rollup_size = (args.size() > 1) ? (size_t)atoi(args[1].c_str()) : 1;
    std::string srs_path = (args.size() > 2) ? args[2] : "../srs_db/ignition";

    auto inner_circuit_data = compute_join_split_circuit_data(srs_path);
    auto circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data, true, srs_path);
    auto noop_proof = create_noop_join_split_proof(inner_circuit_data);
    auto gibberish_data_roots_path = fr_hash_path(28, std::make_pair(fr::random_element(), fr::random_element()));

    std::cerr << "Reading rollups from standard input..." << std::endl;

    while (true) {
        rollup_tx rollup;

        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        read(std::cin, rollup);

        std::cerr << "Received rollup " << rollup.rollup_id << " with " << rollup.num_txs << " txs." << std::endl;

        if (rollup.num_txs > rollup_size) {
            std::cerr << "Receieved rollup size too large: " << rollup.txs.size() << std::endl;
            continue;
        }

        // Pad the rollup with noop proofs.
        auto padding = rollup_size - rollup.num_txs;
        std::cerr << "Padding required: " << padding << std::endl;
        for (size_t i = 0; i < padding; ++i) {
            rollup.txs.push_back(noop_proof);
            rollup.new_null_roots.resize(rollup_size * 2, rollup.new_null_roots.back());
            rollup.old_null_paths.resize(rollup_size * 2, rollup.new_null_paths.back());
            rollup.new_null_paths.resize(rollup_size * 2, rollup.new_null_paths.back());
            rollup.data_roots_paths.resize(rollup_size, gibberish_data_roots_path);
            rollup.data_roots_indicies.resize(rollup_size, 0);
        }

        Timer timer;
        circuit_data.proving_key->reset();

        std::cerr << "Verifying..." << std::endl;
        auto result = verify_rollup(rollup, circuit_data);

        std::cerr << "Time taken: " << timer.toString() << std::endl;
        std::cerr << "Verified: " << result.verified << std::endl;

        write(std::cout, result.proof_data);
        write(std::cout, (uint8_t)result.verified);
    }

    return 0;
}
