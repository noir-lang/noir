#include "../prover/create.hpp"
#include "../prover/destroy.hpp"
#include "../prover/join.hpp"
#include "../prover/join_split.hpp"
#include "../prover/rollup_context.hpp"
#include "../prover/split.hpp"
#include "../prover/timer.hpp"
#include "../tx/batch_tx.hpp"
#include "../tx/user_context.hpp"

char const* DATA_DB_PATH = "/tmp/rollup_prover";
char const* NULLIFIER_DB_PATH = "/tmp/rollup_prover_nullifier";

typedef std::tuple<std::shared_ptr<waffle::proving_key>, std::shared_ptr<waffle::verification_key>, size_t>
    circuit_keys;

using namespace rollup::prover;

rollup_context create_rollup_context(std::string const& id, Composer& composer)
{
    // TODO: We can't have distinct databases due to requiring atomicity. Change to use a single db with multiple trees.
    leveldb_store data_db("/tmp/" + id, 32);
    leveldb_store nullifier_db("/tmp/" + id + "_nullifier", 128);

    return {
        composer,
        std::move(data_db),
        std::move(nullifier_db),
        public_witness_ct(&composer, data_db.size()),
        public_witness_ct(&composer, data_db.root()),
        public_witness_ct(&composer, nullifier_db.root()),
    };
}

void reset_db(std::string const& id)
{
    std::string data_db_path = "/tmp/" + id;
    std::string nullifier_db_path = "/tmp/" + id + "_nullifier";
    leveldb_store::destroy(data_db_path);
    leveldb_store::destroy(nullifier_db_path);
}

circuit_keys create_circuit_keys(size_t batch_size)
{
    std::cout << "Generating circuit keys..." << std::endl;

    std::string id = "gen_circuit_keys";
    reset_db(id);

    Composer composer = Composer("../srs_db/ignition");
    rollup_context ctx = create_rollup_context(id, composer);
    user_context user = create_user_context();

    auto tx = create_join_split_tx({ "0", "0", "-", "-", "50", "50", "100", "0" }, user);
    join_split(ctx, tx);

    for (size_t i = 0; i < batch_size - 1; ++i) {
        auto index1 = std::to_string(i * 2);
        auto index2 = std::to_string(i * 2 + 1);
        tx = create_join_split_tx({ index1, index2, "50", "50", "50", "50", "0", "0" }, user);
        join_split(ctx, tx);
    }
    std::cout << "Circuit size: " << composer.get_num_gates() << std::endl;
    auto keys = std::make_tuple(
        ctx.composer.compute_proving_key(), ctx.composer.compute_verification_key(), composer.get_num_gates());
    std::cout << "Done." << std::endl;

    return keys;
}

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);
    std::string db_id = "rollup_prover";

    if (args.size() > 1 && args[1] == "reset") {
        reset_db(db_id);
        std::cout << "Erased db." << std::endl;
        return 0;
    }

    size_t batch_size = (args.size() > 1) ? (size_t)atoi(args[1].c_str()) : 1;

    auto circuit_keys = create_circuit_keys(batch_size);

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
        rollup_context ctx = create_rollup_context(db_id, composer);

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
