#include "escape_hatch_tx.hpp"
#include <common/streams.hpp>
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace escape_hatch {

using namespace barretenberg;

void write(std::vector<uint8_t>& buf, escape_hatch_tx const& tx)
{
    using serialize::write;
    write(buf, tx.js_tx);

    write(buf, tx.rollup_id);
    write(buf, tx.data_start_index);
    write(buf, tx.new_data_root);
    write(buf, tx.old_data_path);
    write(buf, tx.new_data_path);

    write(buf, tx.old_null_root);
    write(buf, tx.new_null_roots);
    write(buf, tx.old_null_paths);
    write(buf, tx.new_null_paths);
    write(buf, tx.account_null_path);

    write(buf, tx.old_data_roots_root);
    write(buf, tx.new_data_roots_root);
    write(buf, tx.old_data_roots_path);
    write(buf, tx.new_data_roots_path);
}

void read(uint8_t const*& buf, escape_hatch_tx& tx)
{
    using serialize::read;
    read(buf, tx.js_tx);

    read(buf, tx.rollup_id);
    read(buf, tx.data_start_index);
    read(buf, tx.new_data_root);
    read(buf, tx.old_data_path);
    read(buf, tx.new_data_path);

    read(buf, tx.old_null_root);
    read(buf, tx.new_null_roots);
    read(buf, tx.old_null_paths);
    read(buf, tx.new_null_paths);
    read(buf, tx.account_null_path);

    read(buf, tx.old_data_roots_root);
    read(buf, tx.new_data_roots_root);
    read(buf, tx.old_data_roots_path);
    read(buf, tx.new_data_roots_path);
}

bool operator==(escape_hatch_tx const& lhs, escape_hatch_tx const& rhs)
{
    // clang-format off
    return lhs.js_tx == rhs.js_tx
        && lhs.rollup_id == rhs.rollup_id
        && lhs.data_start_index == rhs.data_start_index
        && lhs.new_data_root == rhs.new_data_root
        && lhs.old_data_path == rhs.old_data_path
        && lhs.new_data_path == rhs.new_data_path
        && lhs.old_null_root == rhs.old_null_root
        && lhs.new_null_roots == rhs.new_null_roots
        && lhs.old_null_paths == rhs.old_null_paths
        && lhs.new_null_paths == rhs.new_null_paths
        && lhs.account_null_path == rhs.account_null_path
        && lhs.old_data_roots_root == rhs.old_data_roots_root
        && lhs.new_data_roots_root == rhs.new_data_roots_root
        && lhs.old_data_roots_path == rhs.old_data_roots_path
        && lhs.new_data_roots_path == rhs.new_data_roots_path;
    // clang-format on
}

std::ostream& operator<<(std::ostream& os, escape_hatch_tx const& tx)
{
    os << "join_split: " << tx.js_tx << "\n";
    os << "rollup_id: " << tx.rollup_id << "\n";
    os << "data_start_index: " << tx.data_start_index << "\n";

    os << "\nDATA TREE UPDATE CONTEXT:\n";
    os << "new_data_root: " << tx.new_data_root << "\n";
    os << "old_data_path: " << tx.old_data_path << "\n";
    os << "new_data_path: " << tx.new_data_path << "\n";

    os << "\nNULL TREE UPDATE CONTEXT:\n";
    os << "old_null_root: " << tx.old_null_root << "\n";
    os << "new_null_roots:\n";
    for (auto e : tx.new_null_roots) {
        os << e << "\n";
    }
    os << "old_null_paths:\n";
    for (auto e : tx.old_null_paths) {
        os << e << "\n";
    }
    os << "new_null_paths:\n";
    for (auto e : tx.new_null_paths) {
        os << e << "\n";
    }
    os << "account_null_path: " << tx.account_null_path << "\n";

    os << "old_data_roots_root: " << tx.old_data_roots_root << "\n";
    os << "new_data_roots_root: " << tx.new_data_roots_root << "\n";
    os << "old_data_roots_path: " << tx.old_data_roots_path << "\n";
    os << "new_data_roots_path: " << tx.new_data_roots_path << "\n";

    return os;
}

} // namespace escape_hatch
} // namespace proofs
} // namespace rollup
