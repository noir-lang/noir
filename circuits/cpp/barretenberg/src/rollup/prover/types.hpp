#pragma once
#include <barretenberg/misc_crypto/commitment/pedersen_note.hpp>
#include <barretenberg/waffle/composer/turbo_composer.hpp>
#include <barretenberg/waffle/stdlib/byte_array/byte_array.hpp>
#include <barretenberg/waffle/stdlib/merkle_tree/merkle_tree.hpp>
#include <barretenberg/waffle/stdlib/uint32/uint32.hpp>

namespace rollup {

using namespace barretenberg;
using namespace plonk;
using namespace int_utils;

typedef waffle::TurboComposer Composer;
typedef stdlib::uint32<Composer> uint32;
typedef stdlib::field_t<Composer> field_t;
typedef stdlib::bool_t<Composer> bool_t;
typedef stdlib::byte_array<Composer> byte_array;
typedef stdlib::merkle_tree::fr_hash_path fr_hash_path;
typedef stdlib::merkle_tree::hash_path<Composer> hash_path;
typedef stdlib::merkle_tree::LevelDbStore leveldb_store;
typedef stdlib::witness_t<Composer> witness_t;
typedef stdlib::public_witness_t<Composer> public_witness_t;
typedef crypto::pedersen_note::private_note tx_note;

struct rollup_context {
    Composer& composer;
    leveldb_store data_db;
    leveldb_store nullifier_db;
    field_t data_size;
    field_t data_root;
    field_t nullifier_root;
};

} // namespace rollup