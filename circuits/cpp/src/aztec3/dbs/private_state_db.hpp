// // TODO: move the leveldb_store to a more neutral module.
// #include "../constants.hpp"

// #include "aztec3/utils/types/native_types.hpp"

// #include <barretenberg/barretenberg.hpp>

// namespace aztec3::db {

// using aztec3::GeneratorIndex;
// using crypto::pedersen::hash;

// char const* PRIVATE_STATE_DB_PATH = "./private_state_var.db";

// struct PrivateStateVar {
//     bool is_partitioned = false;
//     std::vector<PrivateStatePreimage*> active_private_state_preimages;
// };

// // contract->storageSlots->[current->historic]

// /**
//  * Hmmm... There are multiple active leaves for partitioned states.
//  *
//  * [contract_Address, storage_slot] -> {
//  *     is_partitioned,
//  *     earliest_active_commitment,
//  *     earliest_commitment,
//  * }
//  *
//  * commitment -> {
//  *     PrivateStateVar,
//  *     next_active_commitment,
//  *     next_commitment,
//  * }
//  */

// template <typename Store> class PrivateStateDb {
//   public:
//     PrivateStateDb(Store& store, size_t max_state_var_id)
//         : store_(store)
//         , max_state_var_id_(max_state_var_id)
//     {}

//     // we need a linked list through all active commitments, and another linked list through all (active & inactive)
//     // commitments.

//     PrivateStateCommitment get_earliest_active_commitment(fr const& contract_address, fr const& state_var_id)
//     {
//         const fr& storage_slot = state_var_id;
//         const fr db_key =
//             commit_native(std::vector<fr>{ contract_address, storage_slot }, GeneratorIndex::UNIVERSAL_STORAGE_SLOT);
//     }

//     PrivateStateCommitment get_earliest_active_commitment(fr const& contract_address,
//                                                           fr const& state_var_id,
//                                                           fr const& mapping_key)
//     {
//         const fr storage_slot =
//             commit_native(std::vector<fr>{ state_var_id, mapping_key }, GeneratorIndex::MAPPING_STORAGE_SLOT);
//         const fr db_key =
//             commit_native(std::vector<fr>{ contract_address, storage_slot }, GeneratorIndex::UNIVERSAL_STORAGE_SLOT);

//         std::vector<uint8_t> data;

//         bool success = store_.get(db_key, data);

//         return data;
//     }

//     fr get_current_private_state_value(fr const& contract_address, fr const& state_var_id)
//     {
//         PrivateStateCommitment earliest_active_commitment =
//             get_earliest_active_commitment(contract_address, state_var_id);
//     }

//     fr get_current_private_state_value(fr const& contract_address, fr const& state_var_id, fr const& mapping_key) {}

//     void write_metadata(std::ostream& os)
//     {
//         write(os, data_tree_.root());
//         write(os, nullifier_tree_.root());
//         write(os, root_tree_.root());
//         write(os, defi_tree_.root());
//         write(os, data_tree_.size());
//         write(os, nullifier_tree_.size());
//         write(os, root_tree_.size());
//         write(os, defi_tree_.size());
//     }

//     void get(std::istream& is, std::ostream& os)
//     {
//         GetRequest get_request;
//         read(is, get_request);
//         // std::cerr << get_request << std::endl;
//         auto tree = trees_[get_request.tree_id];
//         auto path = tree->get_hash_path(get_request.index);
//         auto leaf = get_request.index & 0x1 ? path[0].second : path[0].first;
//         write(os, leaf == fr::neg_one() ? fr(0) : leaf);
//     }

//     void get_path(std::istream& is, std::ostream& os)
//     {
//         GetRequest get_request;
//         read(is, get_request);
//         // std::cerr << get_request << std::endl;
//         auto tree = trees_[get_request.tree_id];
//         auto path = tree->get_hash_path(get_request.index);
//         write(os, path);
//     }

//     void put(std::istream& is, std::ostream& os)
//     {
//         PutRequest put_request;
//         read(is, put_request);
//         // std::cerr << put_request << std::endl;
//         PutResponse put_response;
//         put_response.root = trees_[put_request.tree_id]->update_element(put_request.index, put_request.value);
//         write(os, put_response);
//     }

//     void batch_put(std::istream& is, std::ostream& os)
//     {
//         std::vector<PutRequest> put_requests;
//         read(is, put_requests);
//         for (auto& put_request : put_requests) {
//             trees_[put_request.tree_id]->update_element(put_request.index, put_request.value);
//         }
//         write_metadata(os);
//     }

//     void commit(std::ostream& os)
//     {
//         // std::cerr << "COMMIT" << std::endl;
//         store_.commit();
//         write_metadata(os);
//     }

//     void rollback(std::ostream& os)
//     {
//         // std::cerr << "ROLLBACK" << std::endl;
//         store_.rollback();
//         write_metadata(os);
//     }

//   private:
//     Store& store_;
//     size_t max_state_var_id_;
// };

// } // namespace aztec3::db