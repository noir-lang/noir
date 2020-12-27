#include "../proofs/join_split/join_split.hpp"
#include "../proofs/join_split/join_split_tx.hpp"
#include "../proofs/join_split/create_noop_join_split_proof.hpp"
#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/rollup/create_rollup.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/root_rollup/create_root_rollup_tx.hpp"
#include "../constants.hpp"
#include <common/streams.hpp>
#include <iostream>
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include <stdlib/types/turbo.hpp>

using namespace ::rollup::proofs;
using namespace plonk::stdlib::merkle_tree;
using namespace plonk::stdlib::types::turbo;
namespace tx_rollup = ::rollup::proofs::rollup;
using Tree = MerkleTree<MemoryStore>;

auto prefix = "tx_factory: ";

void create_inner_rollup(uint32_t num_txs,
                         uint32_t rollup_size,
                         join_split::circuit_data const& join_split_circuit_data,
                         barretenberg::fr const& data_tree_root,
                         Tree& data_tree,
                         Tree& null_tree,
                         Tree& root_tree)
{
    std::cerr << prefix << "Generating a " << rollup_size << " rollup with " << num_txs << " txs..." << std::endl;
    auto proofs = std::vector<std::vector<uint8_t>>(num_txs);
    for (size_t i = 0; i < num_txs; ++i) {
        proofs[i] = create_noop_join_split_proof(join_split_circuit_data, data_tree_root);
    }
    tx_rollup::rollup_tx rollup = tx_rollup::create_rollup(proofs, data_tree, null_tree, root_tree, rollup_size);

    write(std::cout, (uint32_t)0);
    write(std::cout, rollup);
    std::cerr << prefix << "Sent." << std::endl;
}

int main(int argc, char** argv)
{
    MemoryStore store;
    Tree data_tree(store, ::rollup::DATA_TREE_DEPTH, 0);
    Tree null_tree(store, ::rollup::NULL_TREE_DEPTH, 1);
    Tree root_tree(store, ::rollup::ROOT_TREE_DEPTH, 2);

    std::vector<std::string> args(argv, argv + argc);

    if (args.size() < 3) {
        std::cerr << "usage:\n" << args[0] << " <num_txs> <inner_rollup_size> [output_file]" << std::endl;
        return -1;
    }

    bool initialized;
    read(std::cin, initialized);

    auto join_split_circuit_data = join_split::compute_circuit_data("../srs_db/ignition");
    auto data_root = data_tree.root();
    root_tree.update_element(0, to_buffer(data_root));

    uint32_t num_rollups = 0;
    uint32_t num_txs = static_cast<uint32_t>(std::stoul(args[1]));
    const uint32_t inner_rollup_size = static_cast<uint32_t>(std::stoul(args[2]));

    while (num_txs > 0) {
        auto n = std::min(num_txs, inner_rollup_size);
        create_inner_rollup(n, inner_rollup_size, join_split_circuit_data, data_root, data_tree, null_tree, root_tree);
        num_txs -= n;
        num_rollups++;
    }

    std::cerr << prefix << "Reading inner rollups..." << std::endl;

    std::vector<std::vector<uint8_t>> rollups_data;
    for (uint32_t i = 0; i < num_rollups; ++i) {
        std::vector<uint8_t> proof_data;
        bool verified;
        read(std::cin, proof_data);
        read(std::cin, verified);
        if (!verified) {
            std::cerr << prefix << "Received an unverified proof." << std::endl;
            return -1;
        }
        rollups_data.push_back(proof_data);
    }

    auto root_rollup = root_rollup::create_root_rollup_tx(0, rollups_data, data_tree, root_tree);
    write(std::cout, (uint32_t)1);
    write(std::cout, root_rollup);

    if (args.size() > 3) {
        std::vector<uint8_t> proof_data;
        bool verified;
        read(std::cin, proof_data);
        read(std::cin, verified);
        std::ofstream of(args[3]);
        write(of, proof_data);
        write(of, verified);
    }

    return 0;
}
