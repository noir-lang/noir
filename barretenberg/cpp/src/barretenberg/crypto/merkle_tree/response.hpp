#pragma once

#include "barretenberg/crypto/merkle_tree/hash_path.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_leaf.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <exception>
#include <functional>
#include <memory>
#include <optional>
#include <string>

namespace bb::crypto::merkle_tree {
struct TreeMetaResponse {
    std::string name;
    uint32_t depth;
    index_t size;
    fr root;
};

struct AddDataResponse {
    index_t size;
    fr root;
};

struct GetSiblingPathResponse {
    fr_sibling_path path;
};

template <typename LeafType> struct LowLeafWitnessData {
    IndexedLeaf<LeafType> leaf;
    index_t index;
    fr_sibling_path path;

    MSGPACK_FIELDS(leaf, index, path);
};

template <typename LeafValueType> struct AddIndexedDataResponse {
    AddDataResponse add_data_result;
    fr_sibling_path subtree_path;
    std::shared_ptr<std::vector<std::pair<LeafValueType, size_t>>> sorted_leaves;
    std::shared_ptr<std::vector<LowLeafWitnessData<LeafValueType>>> low_leaf_witness_data;
};

struct FindLeafIndexResponse {
    index_t leaf_index;
};

struct GetLeafResponse {
    std::optional<bb::fr> leaf;
};

template <typename LeafValueType> struct GetIndexedLeafResponse {
    std::optional<IndexedLeaf<LeafValueType>> indexed_leaf;
};

template <typename ResponseType> struct TypedResponse {
    ResponseType inner;
    bool success{ true };
    std::string message;
};

struct Response {
    bool success;
    std::string message;
};

template <typename ResponseType>
void execute_and_report(const std::function<void(TypedResponse<ResponseType>&)>& f,
                        const std::function<void(const TypedResponse<ResponseType>&)>& on_completion)
{
    TypedResponse<ResponseType> response;
    try {
        f(response);
    } catch (std::exception& e) {
        response.success = false;
        response.message = e.what();
    }
    try {
        on_completion(response);
    } catch (std::exception&) {
    }
}

inline void execute_and_report(const std::function<void()>& f,
                               const std::function<void(const Response&)>& on_completion)
{
    Response response{ true, "" };
    try {
        f();
    } catch (std::exception& e) {
        response.success = false;
        response.message = e.what();
    }
    try {
        on_completion(response);
    } catch (std::exception&) {
    }
}
} // namespace bb::crypto::merkle_tree
