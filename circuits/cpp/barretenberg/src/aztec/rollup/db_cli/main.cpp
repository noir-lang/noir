#include "../prover/create.hpp"
#include "../prover/destroy.hpp"
#include "../prover/join.hpp"
#include "../prover/join_split.hpp"
#include "../prover/rollup_context.hpp"
#include "../prover/split.hpp"
#include "../prover/timer.hpp"
#include "../tx/batch_tx.hpp"
#include "../tx/user_context.hpp"

char const* DB_PATH = "./world_state.db";

enum Command {
    GET,
    PUT,
    COMMIT,
    ROLLBACK,
};
struct PutCommand {
    uint128_t index;
    std::array<uint8_t, 64> value;
};

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    if (args.size() > 1 && args[1] == "reset") {
        leveldb_store::destroy(DB_PATH);
        std::cout << "Erased db." << std::endl;
        return 0;
    }

    leveldb_store data_db(, 32);

    // Read commands from stdin.
    while (true) {
        uint32_t command;

        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        read(std::cin, command);

        std::cout << "DB root: " << ctx.data_db.root() << " size: " << ctx.data_db.size() << std::endl;
        std::cout << "Nullifier root: " << ctx.nullifier_db.root() << " size: " << ctx.nullifier_db.size() << std::endl;

        Timer circuit_timer;
        for (auto tx : batch.txs) {
            std::cout << tx << std::endl;
            if (!join_split(ctx, tx)) {
                std::cout << "Failed to generate witness data." << std::endl;
                return -1;
            }
        }

        // Pad the circuit with gibberish notes.
        auto user = create_user_context();
        for (size_t i = 0; i < batch_size - batch.txs.size(); ++i) {
            join_split(ctx, create_join_split_tx({ "0", "0", "-", "-", "0", "0", "0", "0" }, user));
        }

        std::cout << "Time taken to create circuit: " << circuit_timer.toString() << std::endl;
        printf("composer gates = %zu\n", ctx.composer.get_num_gates());

        std::cout << "Computing witness..." << std::endl;
        Timer witness_ctimer;
        ctx.composer.compute_witness();
        std::cout << "Time taken to compute witness: " << witness_ctimer.toString() << std::endl;

        std::cout << "Creating prover..." << std::endl;
        Timer prover_timer;
        auto prover = ctx.composer.create_prover();
        std::cout << "Time taken to create prover: " << prover_timer.toString() << std::endl;

        std::cout << "Constructing proof..." << std::endl;
        Timer proof_timer;
        waffle::plonk_proof proof = prover.construct_proof();
        std::cout << "Time taken to construct proof: " << proof_timer.toString() << std::endl;

        auto verifier = ctx.composer.create_verifier();
        bool verified = verifier.verify_proof(proof);
        std::cout << "Verified: " << verified << std::endl;

        if (verified) {
            ctx.data_db.commit();
            ctx.nullifier_db.commit();
        } else {
            ctx.data_db.rollback();
            ctx.nullifier_db.rollback();
        }
    }

    return 0;
}
