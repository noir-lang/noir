#include "../proofs/join_split/join_split.hpp"
#include "../proofs/join_split/join_split_tx.hpp"
#include "../proofs/join_split/create_noop_join_split_proof.hpp"
#include "../proofs/rollup/compute_circuit_data.hpp"
#include "../proofs/rollup/create_rollup.hpp"
#include "../proofs/rollup/rollup_tx.hpp"
#include "../proofs/root_rollup/create_root_rollup_tx.hpp"
#include "../proofs/root_rollup/compute_or_load_fixture.hpp"
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
auto data_path = "./data/tx_factory";

tx_rollup::rollup_tx create_inner_rollup(size_t rollup_num,
                                         uint32_t num_txs,
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
        auto name = format("js", rollup_num * rollup_size + i);
        proofs[i] = root_rollup::compute_or_load_fixture(
            data_path, name, [&]() { return create_noop_join_split_proof(join_split_circuit_data, data_tree_root); });
    }
    return tx_rollup::create_rollup(proofs, data_tree, null_tree, root_tree, rollup_size);
}

int main(int argc, char** argv)
{
    MemoryStore store;
    Tree data_tree(store, ::rollup::DATA_TREE_DEPTH, 0);
    Tree null_tree(store, ::rollup::NULL_TREE_DEPTH, 1);
    Tree root_tree(store, ::rollup::ROOT_TREE_DEPTH, 2);
    Tree defi_tree(store, ::rollup::DEFI_TREE_DEPTH, 3);

    std::vector<std::string> args(argv, argv + argc);

    if (args.size() < 3) {
        std::cerr << "usage:\n"
                  << args[0] << " <num_txs> <inner_rollup_size> <outer_rollup_size> [output_file]" << std::endl;
        return -1;
    }

    mkdir("./data", 0700);
    mkdir(data_path, 0700);

    bool initialized;
    read(std::cin, initialized);

    auto crs = std::make_shared<waffle::DynamicFileReferenceStringFactory>("../srs_db/ignition");
    auto join_split_circuit_data = join_split::compute_circuit_data(crs);
    auto data_root = data_tree.root();
    root_tree.update_element(0, to_buffer(data_root));

    uint32_t num_txs = static_cast<uint32_t>(std::stoul(args[1]));
    const uint32_t inner_rollup_size = static_cast<uint32_t>(std::stoul(args[2]));
    const uint32_t outer_rollup_size = static_cast<uint32_t>(std::stoul(args[3]));

    std::vector<std::vector<uint8_t>> rollups_data;
    while (num_txs > 0) {
        auto rollup_num = rollups_data.size();
        auto n = std::min(num_txs, inner_rollup_size);
        auto name = format("rollup_", rollup_num, "_", n, "txs.dat");
        num_txs -= n;

        auto rollup = create_inner_rollup(rollups_data.size(),
                                          n,
                                          inner_rollup_size,
                                          join_split_circuit_data,
                                          data_root,
                                          data_tree,
                                          null_tree,
                                          root_tree);

        auto proof_data = root_rollup::compute_or_load_fixture(data_path, name, [&]() {
            std::cerr << prefix << "Sending..." << std::endl;
            write(std::cout, (uint32_t)0);
            write(std::cout, (uint32_t)inner_rollup_size);
            write(std::cout, rollup);
            std::cerr << prefix << "Sent." << std::endl;

            std::vector<uint8_t> proof_data;
            bool verified;
            read(std::cin, proof_data);
            read(std::cin, verified);
            if (!verified) {
                throw std::runtime_error("Received an unverified proof.");
            }
            return proof_data;
        });

        rollups_data.push_back(proof_data);
    }

    auto root_rollup = root_rollup::create_root_rollup_tx(0, rollups_data, data_tree, root_tree, defi_tree);
    write(std::cout, (uint32_t)1);
    write(std::cout, (uint32_t)inner_rollup_size);
    write(std::cout, (uint32_t)outer_rollup_size);
    write(std::cout, root_rollup);

    if (args.size() > 4) {
        std::vector<uint8_t> proof_data;
        bool verified;
        read(std::cin, proof_data);
        read(std::cin, verified);
        std::ofstream of(args[4]);
        write(of, proof_data);
        write(of, verified);
    }

    return 0;
}
