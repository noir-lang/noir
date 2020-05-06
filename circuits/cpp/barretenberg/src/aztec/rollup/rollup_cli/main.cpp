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

    std::cerr << "Reading rollups from standard input..." << std::endl;

    // Read transactions from stdin.
    while (true) {
        rollup_tx rollup;

        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        read(std::cin, rollup);
        std::cout << "Received rollup " << rollup.rollup_id << " with " << rollup.num_txs << " txs." << std::endl;

        if (rollup.num_txs > batch_size) {
            std::cerr << "Receieved rollup size too large: " << rollup.txs.size() << std::endl;
            continue;
        }

        // TODO:
        // if (rollup.proof_lengths != circuit_data.proof_lengths) {
        //     std::cerr << "Proof lengths incorrect: " << rollup.proof_lengths << std::endl;
        //     continue;
        // }

        // Pad the rollup with gibberish proofs.
        for (size_t i = 0; i < batch_size - rollup.num_txs; ++i) {
            rollup.txs.push_back(std::vector<uint8_t>(rollup.proof_lengths));
        }

        // Do annoying transform from raw bytes to waffle::plonk_proof. Should we be serializing plonk_proof struct?
        std::vector<waffle::plonk_proof> proofs(batch_size);
        std::transform(rollup.txs.begin(), rollup.txs.end(), proofs.begin(), [](auto const& p) {
            return waffle::plonk_proof{ p };
        });

        Timer circuit_timer;
        Composer composer = Composer(circuit_data.proving_key, circuit_data.verification_key, circuit_data.num_gates);
        rollup_circuit(composer, batch_size, proofs, circuit_data.inner_verification_key);

        std::cerr << "Time taken to create circuit: " << circuit_timer.toString() << std::endl;
        std::cerr << "composer gates = " << composer.get_num_gates() << std::endl;

        std::cerr << "Computing witness..." << std::endl;
        Timer witness_timer;
        composer.compute_witness();
        std::cerr << "Time taken to compute witness: " << witness_timer.toString() << std::endl;

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
    }

    return 0;
}
