#include "leveldb_store.hpp"
#include "leveldb_tx.hpp"
#include "hash.hpp"
#include <common/net.hpp>
#include <iostream>
#include <numeric/bitop/count_leading_zeros.hpp>
#include <numeric/bitop/keep_n_lsb.hpp>
#include <sstream>
#include <leveldb/db.h>
#include <leveldb/write_batch.h>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

namespace {
barretenberg::fr from_string(std::string const& data, size_t offset = 0)
{
    barretenberg::fr result;
    std::copy(data.data() + offset, data.data() + offset + sizeof(barretenberg::fr), (char*)result.data);
    return result;
}
} // namespace

LevelDbStore::LevelDbStore(std::string const& db_path, size_t depth)
    : depth_(depth)
{
    ASSERT(depth_ >= 1 && depth <= 128);
    zero_hashes_.resize(depth);

    leveldb::DB* db;
    leveldb::Options options;
    options.create_if_missing = true;
    options.compression = leveldb::kNoCompression;
    leveldb::Status status = leveldb::DB::Open(options, db_path, &db);
    ASSERT(status.ok());
    db_.reset(db);
    tx_.reset(new leveldb_tx(*db_));

    // Compute the zero values at each layer.
    auto current = hash_value_native(std::string(LEAF_BYTES, 0));
    for (size_t i = 0; i < depth; ++i) {
        zero_hashes_[i] = current;
        // std::cout << "zero hash level " << i << ": " << current << std::endl;
        current = compress_native({ current, current });
    }

    std::string root;
    status = db->Get(leveldb::ReadOptions(), "root", &root);
    root_ = status.ok() ? from_string(root) : current;

    std::string size;
    status = db->Get(leveldb::ReadOptions(), "size", &size);
    size_ = status.ok() ? ntohll(*reinterpret_cast<uint64_t*>(size.data())) : 0ULL;
}

LevelDbStore::LevelDbStore(LevelDbStore&& other)
    : db_(std::move(other.db_))
    , tx_(std::move(other.tx_))
    , zero_hashes_(std::move(other.zero_hashes_))
    , depth_(other.depth_)
    , size_(other.size_)
    , root_(other.root_)
{}

LevelDbStore::~LevelDbStore() {
}

void LevelDbStore::destroy(std::string path) {
    leveldb::DestroyDB(path, leveldb::Options());
}

barretenberg::fr LevelDbStore::root() const
{
    return root_;
}

size_t LevelDbStore::size() const
{
    return size_;
}

fr_hash_path LevelDbStore::get_hash_path(index_t index)
{
    fr_hash_path path(depth_);

    std::string data;
    auto status = tx_->Get(leveldb::Slice((char*)&root_, 32), &data);

    for (size_t i = depth_ - 1; i < depth_; --i) {
        if (!status.ok()) {
            // This is an empty subtree. Fill in zero value.
            path[i] = std::make_pair(zero_hashes_[i], zero_hashes_[i]);
            continue;
        }

        if (data.size() == 64) {
            // This is a regular node with left and right trees. Descend according to index path.
            auto left = from_string(data, 0);
            auto right = from_string(data, 32);
            path[i] = std::make_pair(left, right);
            bool is_right = (index >> i) & 0x1;
            status = tx_->Get(data.substr(is_right ? 32 : 0, 32), &data);
        } else {
            // This is a stump. The hash path can be fully restored from this node.
            index_t element_index = *(index_t*)(data.data() + 32);
            fr current = from_string(data, 0);
            index_t diff = element_index ^ numeric::keep_n_lsb(index, i + 1);

            // std::cout << "ghp hit stump height:" << i << " element_index:" << (uint64_t)element_index
            //           << " index:" << (uint64_t)index << " diff:" << (uint64_t)diff << std::endl;

            if (diff < 2) {
                for (size_t j = 0; j <= i; ++j) {
                    bool is_right = (element_index >> j) & 0x1;
                    if (is_right) {
                        path[j] = std::make_pair(zero_hashes_[j], current);
                    } else {
                        path[j] = std::make_pair(current, zero_hashes_[j]);
                    }
                    current = compress_native({ path[j].first, path[j].second });
                }
            } else {
                size_t common_bits = numeric::count_leading_zeros(diff);
                size_t ignored_bits = sizeof(index_t) * 8 - i;
                size_t common_height = i - (common_bits - ignored_bits) - 1;

                // std::cout << "ghp diff:" << (uint64_t)diff << " ch:" << common_height << std::endl;

                for (size_t j = 0; j < common_height; ++j) {
                    path[j] = std::make_pair(zero_hashes_[j], zero_hashes_[j]);
                }
                current = compute_zero_path_hash(common_height, element_index, current);
                for (size_t j = common_height; j <= i; ++j) {
                    bool is_right = (element_index >> j) & 0x1;
                    if (is_right) {
                        path[j] = std::make_pair(zero_hashes_[j], current);
                    } else {
                        path[j] = std::make_pair(current, zero_hashes_[j]);
                    }
                    current = compress_native({ path[j].first, path[j].second });
                }
            }
            break;
        }
    }

    return path;
}

LevelDbStore::value_t LevelDbStore::get_element(index_t index)
{
    fr leaf = get_element(root_, index, depth_);
    std::string data;
    auto status = tx_->Get(leveldb::Slice((char*)&leaf, 32), &data);
    return status.ok() ? data : std::string(64, 0);
}

fr LevelDbStore::get_element(fr const& root, index_t index, size_t height)
{
    // std::cout << "get_element root:" << root << " index:" << (uint64_t)index << " height:" << height << std::endl;
    if (height == 0) {
        return root;
    }

    std::string data;
    auto status = tx_->Get(leveldb::Slice((char*)&root, 32), &data);

    if (!status.ok()) {
        return zero_hashes_[0];
    }

    if (data.size() != 64) {
        index_t existing_index = *(index_t*)(data.data() + 32);
        fr existing_value = from_string(data, 0);
        // std::cout << "get_element stump existing_index:" << (uint64_t)existing_index << " index:" << (uint64_t)index
        //           << std::endl;
        return (existing_index == index) ? existing_value : zero_hashes_[0];
    } else {
        bool is_right = (index >> (height - 1)) & 0x1;
        // std::cout << "get_element reg is_right:" << is_right << std::endl;
        fr subtree_root = from_string(data, is_right ? 32 : 0);
        return get_element(subtree_root, numeric::keep_n_lsb(index, height - 1), height - 1);
    }
}

void LevelDbStore::update_element(index_t index, value_t const& value)
{
    fr sha_leaf = hash_value_native(value);

    auto new_root = update_element(root_, sha_leaf, index, depth_);

    root_ = new_root;
    tx_->Put(leveldb::Slice((char*)&sha_leaf, 32), value);
    tx_->Put("root", leveldb::Slice((char*)&root_, 32));
    uint64_t new_size = htonll(size_);
    tx_->Put("size", leveldb::Slice((char*)&new_size, sizeof(new_size)));
}

void LevelDbStore::commit()
{
    leveldb::WriteBatch batch;
    tx_->populate_write_batch(batch);
    db_->Write(leveldb::WriteOptions(), &batch);
    tx_.reset(new leveldb_tx(*db_));
}

void LevelDbStore::rollback()
{
    tx_.reset(new leveldb_tx(*db_));

    std::string root;
    leveldb::Status status = db_->Get(leveldb::ReadOptions(), "root", &root);
    root_ = status.ok() ? from_string(root) : compress_native({ zero_hashes_.back(), zero_hashes_.back() });

    std::string size;
    status = db_->Get(leveldb::ReadOptions(), "size", &size);
    size_ = status.ok() ? ntohll(*reinterpret_cast<uint64_t*>(size.data())) : 0ULL;
}

fr LevelDbStore::binary_put(index_t a_index, fr const& a, fr const& b, size_t height)
{
    bool a_is_right = (a_index >> (height - 1)) & 0x1;
    auto left = a_is_right ? b : a;
    auto right = a_is_right ? a : b;
    auto key = compress_native({ left, right });
    put(key, left, right);
    // std::cout << "BINARY PUT height: " << height << " key:" << key << " left:" << left << " right:" << right
    //<< std::endl;
    return key;
}

fr LevelDbStore::fork_stump(
    fr const& value1, index_t index1, fr const& value2, index_t index2, size_t height, size_t common_height)
{
    if (height == common_height) {
        if (height == 1) {
            // std::cout << "Stump forked into leaves." << std::endl;
            index1 = numeric::keep_n_lsb(index1, 1);
            index2 = numeric::keep_n_lsb(index2, 1);
            return binary_put(index1, value1, value2, height);
        } else {
            size_t stump_height = height - 1;
            index_t stump1_index = numeric::keep_n_lsb(index1, stump_height);
            index_t stump2_index = numeric::keep_n_lsb(index2, stump_height);
            // std::cout << "Stump forked into two at height " << stump_height << " index1 " << (uint64_t)index1
            //           << " index2 " << (uint64_t)index2 << std::endl;
            fr stump1_hash = compute_zero_path_hash(stump_height, stump1_index, value1);
            fr stump2_hash = compute_zero_path_hash(stump_height, stump2_index, value2);
            put_stump(stump1_hash, stump1_index, value1);
            put_stump(stump2_hash, stump2_index, value2);
            return binary_put(index1, stump1_hash, stump2_hash, height);
        }
    } else {
        auto new_root = fork_stump(value1, index1, value2, index2, height - 1, common_height);
        // std::cout << "Stump branch hash at " << height << " " << new_root << " " << zero_hashes_[height] <<
        // std::endl;
        return binary_put(index1, new_root, zero_hashes_[height - 1], height);
    }
}

fr LevelDbStore::update_element(fr const& root, fr const& value, index_t index, size_t height)
{
    // std::cout << "update_element root:" << root << " value:" << value << " index:" << (uint64_t)index
    //           << " height:" << height << std::endl;
    if (height == 0) {
        return value;
    }

    std::string data;
    auto status = tx_->Get(leveldb::Slice((char*)&root, 32), &data);

    if (!status.ok()) {
        // std::cout << "Adding new stump at height " << height << std::endl;
        fr key = compute_zero_path_hash(height, index, value);
        put_stump(key, index, value);
        size_ += 1;
        return key;
    }

    // std::cout << "got data of size " << data.size() << std::endl;
    if (data.size() < 64) {
        // We've come across a stump.
        index_t existing_index = *(index_t*)(data.data() + 32);

        if (existing_index == index) {
            // We are updating the stumps element. Easy update.
            // std::cout << "Updating existing stump element at index " << (uint64_t)index << std::endl;
            fr new_hash = compute_zero_path_hash(height, index, value);
            put_stump(new_hash, existing_index, value);
            return new_hash;
        }

        fr existing_value = from_string(data, 0);
        size_t common_bits = numeric::count_leading_zeros(existing_index ^ index);
        size_t ignored_bits = sizeof(index_t) * 8 - height;
        size_t common_height = height - (common_bits - ignored_bits);
        // std::cout << height << " common_bits:" << common_bits << " ignored_bits:" << ignored_bits
        //           << " existing_index:" << (uint64_t)existing_index << " index:" << (uint64_t)index
        //           << " common_height:" << common_height << std::endl;

        size_ += 1;
        return fork_stump(existing_value, existing_index, value, index, height, common_height);
    } else {
        bool is_right = (index >> (height - 1)) & 0x1;
        // std::cout << "Normal node is_right:" << is_right << std::endl;
        fr subtree_root = from_string(data, is_right ? 32 : 0);
        subtree_root = update_element(subtree_root, value, numeric::keep_n_lsb(index, height - 1), height - 1);
        auto left = from_string(data, 0);
        auto right = from_string(data, 32);
        if (is_right) {
            right = subtree_root;
        } else {
            left = subtree_root;
        }
        auto new_root = compress_native({ left, right });
        put(new_root, left, right);
        // TODO: Perhaps delete old node?
        return new_root;
    }
}

fr LevelDbStore::compute_zero_path_hash(size_t height, index_t index, fr const& value)
{
    fr current = value;
    for (size_t i = 0; i < height; ++i) {
        bool is_right = (index >> i) & 0x1;
        fr left, right;
        if (is_right) {
            left = zero_hashes_[i];
            right = current;
        } else {
            right = zero_hashes_[i];
            left = current;
        }
        current = compress_native({ is_right ? zero_hashes_[i] : current, is_right ? current : zero_hashes_[i] });
    }
    return current;
}

void LevelDbStore::put(fr const& key, fr const& left, fr const& right)
{
    std::ostringstream os;
    os.write((char*)left.data, 32);
    os.write((char*)right.data, 32);
    tx_->Put(leveldb::Slice((char*)key.data, 32), os.str());
    // std::cout << "PUT key:" << key << " left:" << left << " right:" << right << std::endl;
}

void LevelDbStore::put_stump(fr const& key, index_t index, fr const& value)
{
    std::ostringstream os;
    os.write((char*)value.data, 32);
    os.write((char*)&index, sizeof(index_t));
    tx_->Put(leveldb::Slice((char*)key.data, 32), os.str());
    // std::cout << "PUT STUMP key:" << key << " index:" << (uint64_t)index << " value:" << value << std::endl;
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk