#include "../prover/batch_tx.hpp"
#include "../prover/join_split_tx.hpp"
#include <barretenberg/io/streams.hpp>
#include <iostream>

using namespace rollup;

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);
    user_context user = create_user_context();

    if (args.size() < 2) {
        std::cout << "usage: " << args[0] << " [join-split] [join-split-auto ...>]" << std::endl;
        return -1;
    }

    if (args[1] == "join-split") {
        if (args.size() < 8) {
            std::cout << "usage: " << argv[0]
                      << " join-split <first note index to join> <second note index to join> <first input note value>"
                         " <second input note value> <first output note value> <second output note value>"
                         " [public input] [public output] [json | binary]"
                      << std::endl;
            return -1;
        }

        auto tx = create_join_split_tx({ args.begin() + 2, args.end() }, user);
        if (args.size() < 11 || args[10] == "binary") {
            write(std::cout, hton(tx));
        } else {
            write_json(std::cout, tx);
            std::cout << std::endl;
        }
    } else if (args[1] == "join-split-single") {
        if (args.size() < 8) {
            std::cout << "usage: " << argv[0]
                      << " join-split <first note index to join> <second note index to join> <first input note value>"
                         " <second input note value> <first output note value> <second output note value>"
                         " [public input] [public output]"
                      << std::endl;
            return -1;
        }

        batch_tx batch;
        batch.batch_num = 0;
        batch.txs.push_back(create_join_split_tx({ args.begin() + 2, args.end() }, user));
        std::cerr << batch.txs[0] << std::endl;
        write(std::cout, batch);
    } else if (args.size() > 1 && args[1] == "join-split-auto") {
        bool valid_args = args.size() == 3;
        valid_args |= args.size() == 4 && (args[3] == "binary" || args[3] == "json");
        if (!valid_args) {
            std::cout << "usage: " << argv[0] << " join-split-auto <num transactions> [json | binary]" << std::endl;
            return -1;
        }

        size_t num_txs = (size_t)atoi(args[2].c_str());
        batch_tx batch;
        batch.batch_num = 0;
        batch.txs.reserve(num_txs);
        batch.txs.push_back(create_join_split_tx({ "0", "0", "-", "-", "50", "50", "100", "0" }, user));
        for (size_t i = 0; i < num_txs - 1; ++i) {
            auto index1 = std::to_string(i * 2);
            auto index2 = std::to_string(i * 2 + 1);
            batch.txs.push_back(create_join_split_tx({ index1, index2, "50", "50", "50", "50", "0", "0" }, user));
        }

        auto format = args.size() == 4 ? args[3] : "binary";

        if (format == "binary") {
            write(std::cout, hton(batch));
        } else {
            write_json(std::cout, batch);
        }
    } else {
        std::cout << "usage: " << args[0] << " [join-split] [join-split-auto ...>]" << std::endl;
        return -1;
    }

    return 0;
}
