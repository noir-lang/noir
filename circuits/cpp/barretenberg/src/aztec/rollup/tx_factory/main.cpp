#include "../client_proofs/join_split/join_split.hpp"
#include "../client_proofs/join_split/join_split_tx.hpp"
#include "../client_proofs/join_split/sign_notes.hpp"
#include "../rollup_proofs/create_noop_join_split_proof.hpp"
#include "../rollup_proofs/create_rollup.hpp"
#include "../rollup_proofs/rollup_tx.hpp"
#include "../tx/user_context.hpp"
#include <common/streams.hpp>
#include <iostream>
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/types/turbo.hpp>

using namespace rollup::rollup_proofs;
using namespace rollup::client_proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

int main(int argc, char** argv)
{
    MemoryStore store;
    MerkleTree<MemoryStore> data_tree(store, 32, 0);
    MerkleTree<MemoryStore> null_tree(store, 128, 1);
    MerkleTree<MemoryStore> root_tree(store, 28, 2);

    std::vector<std::string> args(argv, argv + argc);

    if (args.size() < 3) {
        std::cout << "usage: " << args[0] << " <num_txs> <rollup_size>" << std::endl;
        return -1;
    }

    auto data_root = data_tree.root();
    auto rootBuf = data_root.to_buffer();
    auto index = from_buffer<uint128_t>(rootBuf, 16);
    auto non_empty_value = std::vector<uint8_t>(64, 0);
    non_empty_value[63] = 1;
    root_tree.update_element(index, non_empty_value);

    const uint32_t num_txs = static_cast<uint32_t>(std::stoul(args[1]));
    const uint32_t rollup_size = static_cast<uint32_t>(std::stoul(args[2]));

    std::cerr << "Generating a " << rollup_size << " rollup with " << num_txs << " txs..." << std::endl;

    auto join_split_circuit_data = compute_join_split_circuit_data("../srs_db/ignition");

    auto proofs = std::vector<std::vector<uint8_t>>(num_txs);
    for (size_t i = 0; i < num_txs; ++i) {
        proofs[i] = create_noop_join_split_proof(join_split_circuit_data, data_tree.root());
    }
    auto noop_proof = create_noop_join_split_proof(join_split_circuit_data);
    rollup_tx rollup = create_rollup(0, proofs, data_tree, null_tree, root_tree, rollup_size, noop_proof);

    write(std::cout, rollup);

    return 0;
}
