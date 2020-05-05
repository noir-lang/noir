#include <common/timer.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/types/turbo.hpp>
#include "../rollup_proofs/rollup_tx.hpp"
#include "../rollup_proofs/compute_rollup_circuit_data.hpp"

using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::rollup_proofs;


int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    size_t batch_size = (args.size() > 1) ? (size_t)atoi(args[1].c_str()) : 1;

    auto circuit_data = compute_rollup_circuit_data(batch_size);

/*
    // Read transactions from stdin.
    while (true) {
        batch_tx batch;

        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        read(std::cin, batch);

        if (batch.txs.size() > batch_size) {
            std::cerr << "Receieved batch size too large: " << batch.txs.size() << std::endl;
            continue;
        }

        std::get<0>(circuit_keys)->reset();
        // Composer get's corrupted if we use move ctors.
        // Have to create at top level (as opposed to in create_rollup_context).
        Composer composer = Composer(std::get<0>(circuit_keys), std::get<1>(circuit_keys), std::get<2>(circuit_keys));

        Timer circuit_timer;
        for (auto tx : batch.txs) {
            std::cerr << tx << std::endl;
            if (!verify_tx(composer, tx, true)) {
                std::cerr << "Failed to generate witness data." << std::endl;
                return -1;
            }
        }

        // Pad the circuit with gibberish notes.
        for (size_t i = 0; i < batch_size - batch.txs.size(); ++i) {
            join_split_tx tx;
            tx.input_path[0].resize(32);
            tx.input_path[1].resize(32);
            verify_tx(composer, tx, false);
        }

        std::cerr << "Time taken to create circuit: " << circuit_timer.toString() << std::endl;
        std::cerr << "composer gates = " << composer.get_num_gates() << std::endl;
        ;

        std::cerr << "Computing witness..." << std::endl;
        Timer witness_ctimer;
        composer.compute_witness();
        std::cerr << "Time taken to compute witness: " << witness_ctimer.toString() << std::endl;

        std::cerr << "Creating prover..." << std::endl;
        Timer prover_timer;
        auto prover = composer.create_prover();
        std::cerr << "Time taken to create prover: " << prover_timer.toString() << std::endl;

        std::cerr << "Constructing proof..." << std::endl;
        Timer proof_timer;
        waffle::plonk_proof proof = prover.construct_proof();
        std::cerr << "Time taken to construct proof: " << proof_timer.toString() << std::endl;

        auto verifier = composer.create_verifier();
        bool verified = verifier.verify_proof(proof);
        std::cerr << "Verified: " << verified << std::endl;

        write(std::cout, proof.proof_data);
        write(std::cout, (uint8_t)verified);
    }
    */

    return 0;
}
