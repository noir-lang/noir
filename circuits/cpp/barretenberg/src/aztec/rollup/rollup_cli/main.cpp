#include <common/timer.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/types/turbo.hpp>
#include "../rollup_proofs/rollup_tx.hpp"
#include "../rollup_proofs/compute_join_split_circuit_data.hpp"
#include "../rollup_proofs/compute_rollup_circuit_data.hpp"
#include "../rollup_proofs/create_noop_join_split_proof.hpp"
#include "../rollup_proofs/verify_rollup.hpp"

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    size_t rollup_size = (args.size() > 1) ? (size_t)atoi(args[1].c_str()) : 1;

    auto inner_circuit_data = compute_join_split_circuit_data();
    auto circuit_data = compute_rollup_circuit_data(rollup_size, inner_circuit_data);
    auto noop_proof = create_noop_join_split_proof(inner_circuit_data);

    std::cerr << "Reading rollups from standard input..." << std::endl;

    // Read transactions from stdin.
    while (true) {
        rollup_tx rollup;

        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        read(std::cin, rollup);
        std::cerr << rollup << std::endl;

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
            rollup.new_null_roots.resize(rollup_size * 2, *(rollup.new_null_roots.end() - 1));
            rollup.old_null_paths.resize(rollup_size * 2, *(rollup.old_null_paths.end() - 1));
            rollup.new_null_paths.resize(rollup_size * 2, *(rollup.new_null_paths.end() - 1));
        }

        Timer timer;
        circuit_data.proving_key->reset();

        std::cerr << "Verifying..." << std::endl;
        auto verified = verify_rollup(rollup, circuit_data);

        std::cerr << "Time taken: " << timer.toString() << std::endl;
        std::cerr << "Verified: " << verified << std::endl;
    }

    return 0;
}
